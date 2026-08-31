use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::CommandSpec;

#[derive(Serialize, Clone)]
pub struct ProcessOutputPayload {
    pub process_id: String,
    pub data: String,
    pub is_stderr: bool,
}

#[derive(Serialize, Clone)]
pub struct ProcessExitPayload {
    pub id: String,
    pub code: i32,
}

/// Where process events go. The real app emits Tauri events; tests collect
/// them in memory.
pub trait EventSink: Send + Sync + 'static {
    fn output(&self, payload: ProcessOutputPayload);
    fn exit(&self, payload: ProcessExitPayload);
}

impl EventSink for AppHandle {
    fn output(&self, payload: ProcessOutputPayload) {
        let _ = self.emit("process-output", payload);
    }

    fn exit(&self, payload: ProcessExitPayload) {
        let _ = self.emit("process-exit", payload);
    }
}

/// One-shot slot a worker thread fills with the final exit code. Waiting on
/// it never holds the process-manager lock, so Stop stays responsive while a
/// build runs.
pub struct ExitSlot {
    state: Mutex<Option<i32>>,
    cv: Condvar,
}

impl ExitSlot {
    fn new() -> Self {
        Self {
            state: Mutex::new(None),
            cv: Condvar::new(),
        }
    }

    fn set(&self, code: i32) {
        if let Ok(mut state) = self.state.lock() {
            *state = Some(code);
        }
        self.cv.notify_all();
    }

    pub fn get(&self) -> Option<i32> {
        self.state.lock().ok().and_then(|s| *s)
    }

    /// Block until the process (or sequence) finishes; returns the exit code.
    pub fn wait(&self) -> i32 {
        let mut state = match self.state.lock() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        loop {
            if let Some(code) = *state {
                return code;
            }
            state = match self.cv.wait(state) {
                Ok(s) => s,
                Err(_) => return -1,
            };
        }
    }

    /// Wait up to `timeout`; None when still running.
    pub fn wait_timeout(&self, timeout: Duration) -> Option<i32> {
        let mut state = self.state.lock().ok()?;
        let deadline = std::time::Instant::now() + timeout;
        loop {
            if let Some(code) = *state {
                return Some(code);
            }
            let remaining = deadline.checked_duration_since(std::time::Instant::now())?;
            let (next, timed_out) = self.cv.wait_timeout(state, remaining).ok()?;
            state = next;
            if timed_out.timed_out() {
                return (*state).or(None);
            }
        }
    }
}

struct ManagedProcess {
    /// Pid of the currently running step, if one is running right now.
    current_pid: Arc<Mutex<Option<u32>>>,
    exit: Arc<ExitSlot>,
    cancel: Arc<AtomicBool>,
}

#[derive(Default)]
pub struct ProcessManager {
    processes: HashMap<String, ManagedProcess>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// Spawn a single command (a one-step sequence).
    pub fn spawn_command(
        &mut self,
        id: &str,
        working_dir: &std::path::Path,
        command: &str,
        args: &[&str],
        sink: Arc<dyn EventSink>,
    ) -> Result<(), ForgeError> {
        let step = CommandSpec::new(
            command,
            args.iter().map(|a| a.to_string()).collect(),
            working_dir,
        );
        self.spawn_sequence(id, vec![step], sink)
    }

    /// Run `steps` in order on a worker thread, streaming each line of output
    /// through `sink` and stopping at the first failing step. A `$ command`
    /// header line is emitted before each step so the terminal shows what is
    /// running. One `process-exit` event fires when the sequence ends.
    pub fn spawn_sequence(
        &mut self,
        id: &str,
        steps: Vec<CommandSpec>,
        sink: Arc<dyn EventSink>,
    ) -> Result<(), ForgeError> {
        if steps.is_empty() {
            return Err(ForgeError::ProcessError(
                "nothing to run for this action".to_string(),
            ));
        }

        // A finished entry under the same id is stale — reap it so re-running
        // (e.g. building the same target twice) works.
        if let Some(existing) = self.processes.get(id) {
            if existing.exit.get().is_none() {
                return Err(ForgeError::ProcessError(format!(
                    "process already running for id: {id}"
                )));
            }
            self.processes.remove(id);
        }

        let current_pid = Arc::new(Mutex::new(None::<u32>));
        let exit = Arc::new(ExitSlot::new());
        let cancel = Arc::new(AtomicBool::new(false));

        self.processes.insert(
            id.to_string(),
            ManagedProcess {
                current_pid: Arc::clone(&current_pid),
                exit: Arc::clone(&exit),
                cancel: Arc::clone(&cancel),
            },
        );

        let worker_id = id.to_string();
        std::thread::spawn(move || {
            let mut final_code = 0i32;

            for step in &steps {
                if cancel.load(Ordering::Relaxed) {
                    final_code = -1;
                    break;
                }

                sink.output(ProcessOutputPayload {
                    process_id: worker_id.clone(),
                    data: format!("$ {} {}", step.program, step.args.join(" ")),
                    is_stderr: false,
                });

                let mut child = match Command::new(&step.program)
                    .args(&step.args)
                    .current_dir(&step.cwd)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        sink.output(ProcessOutputPayload {
                            process_id: worker_id.clone(),
                            data: format!("failed to start {}: {e}", step.program),
                            is_stderr: true,
                        });
                        final_code = -1;
                        break;
                    }
                };

                if let Ok(mut slot) = current_pid.lock() {
                    *slot = Some(child.id());
                }

                let mut readers = Vec::new();
                if let Some(stdout) = child.stdout.take() {
                    let sink = Arc::clone(&sink);
                    let pid_id = worker_id.clone();
                    readers.push(std::thread::spawn(move || {
                        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                            sink.output(ProcessOutputPayload {
                                process_id: pid_id.clone(),
                                data: line,
                                is_stderr: false,
                            });
                        }
                    }));
                }
                if let Some(stderr) = child.stderr.take() {
                    let sink = Arc::clone(&sink);
                    let pid_id = worker_id.clone();
                    readers.push(std::thread::spawn(move || {
                        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                            sink.output(ProcessOutputPayload {
                                process_id: pid_id.clone(),
                                data: line,
                                is_stderr: true,
                            });
                        }
                    }));
                }

                let status = child.wait();
                for reader in readers {
                    let _ = reader.join();
                }
                if let Ok(mut slot) = current_pid.lock() {
                    *slot = None;
                }

                let code = status.ok().and_then(|s| s.code()).unwrap_or(-1);
                if code != 0 || cancel.load(Ordering::Relaxed) {
                    final_code = if cancel.load(Ordering::Relaxed) && code == 0 {
                        -1
                    } else {
                        code
                    };
                    break;
                }
            }

            exit.set(final_code);
            sink.exit(ProcessExitPayload {
                id: worker_id,
                code: final_code,
            });
        });

        Ok(())
    }

    /// Handle for waiting on a process without holding the manager lock:
    /// grab the handle under the lock, drop the lock, then `wait()`.
    pub fn wait_handle(&self, id: &str) -> Result<Arc<ExitSlot>, ForgeError> {
        self.processes
            .get(id)
            .map(|p| Arc::clone(&p.exit))
            .ok_or_else(|| ForgeError::ProjectNotFound(id.to_string()))
    }

    /// Stop a managed process (or sequence).
    ///
    /// Cancels any pending steps, asks the current child to exit gracefully
    /// (`SIGTERM` on Unix, `taskkill` elsewhere), and escalates to a hard kill
    /// if it ignores the request. Removes the entry, so the id can be reused.
    pub fn kill(&mut self, id: &str) -> Result<(), ForgeError> {
        let managed = self
            .processes
            .remove(id)
            .ok_or_else(|| ForgeError::ProjectNotFound(id.to_string()))?;

        managed.cancel.store(true, Ordering::Relaxed);

        let pid = managed.current_pid.lock().ok().and_then(|p| *p);
        let Some(pid) = pid else {
            // Between steps or already finished; the cancel flag stops any
            // remaining steps.
            return Ok(());
        };

        terminate_pid(pid, false);

        // Give the process up to ~2s to exit gracefully, then force it.
        if managed
            .exit
            .wait_timeout(Duration::from_millis(2000))
            .is_none()
        {
            terminate_pid(pid, true);
        }

        Ok(())
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.processes
            .get(id)
            .map(|p| p.exit.get().is_none())
            .unwrap_or(false)
    }

    pub fn pid(&self, id: &str) -> Option<u32> {
        self.processes
            .get(id)
            .and_then(|p| p.current_pid.lock().ok().and_then(|slot| *slot))
    }
}

/// Ask (or force) a process to stop by pid.
#[cfg(unix)]
fn terminate_pid(pid: u32, force: bool) {
    let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
    // SAFETY: sending a signal to a pid is a simple libc call; an
    // invalid/exited pid just returns an error we ignore.
    unsafe {
        libc::kill(pid as libc::pid_t, signal);
    }
}

#[cfg(not(unix))]
fn terminate_pid(pid: u32, force: bool) {
    let mut cmd = Command::new("taskkill");
    cmd.args(["/PID", &pid.to_string(), "/T"]);
    if force {
        cmd.arg("/F");
    }
    let _ = cmd.output();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[derive(Default)]
    struct TestSink {
        lines: Mutex<Vec<(String, bool)>>,
        exits: Mutex<Vec<(String, i32)>>,
    }

    impl EventSink for TestSink {
        fn output(&self, payload: ProcessOutputPayload) {
            self.lines
                .lock()
                .unwrap()
                .push((payload.data, payload.is_stderr));
        }

        fn exit(&self, payload: ProcessExitPayload) {
            self.exits.lock().unwrap().push((payload.id, payload.code));
        }
    }

    fn sh(script: &str, cwd: &Path) -> CommandSpec {
        CommandSpec::new("sh", vec!["-c".to_string(), script.to_string()], cwd)
    }

    #[test]
    fn sequence_runs_steps_in_order_and_reports_exit() {
        let dir = tempfile::tempdir().unwrap();
        let sink = Arc::new(TestSink::default());
        let mut pm = ProcessManager::new();

        pm.spawn_sequence(
            "seq",
            vec![sh("echo one", dir.path()), sh("echo two", dir.path())],
            sink.clone(),
        )
        .unwrap();

        let handle = pm.wait_handle("seq").unwrap();
        assert_eq!(handle.wait(), 0);

        let lines = sink.lines.lock().unwrap();
        let data: Vec<&str> = lines.iter().map(|(l, _)| l.as_str()).collect();
        assert!(data.contains(&"one"));
        assert!(data.contains(&"two"));
        assert_eq!(sink.exits.lock().unwrap().as_slice(), &[("seq".into(), 0)]);
    }

    #[test]
    fn sequence_stops_at_first_failure() {
        let dir = tempfile::tempdir().unwrap();
        let sink = Arc::new(TestSink::default());
        let mut pm = ProcessManager::new();

        pm.spawn_sequence(
            "fail",
            vec![sh("exit 3", dir.path()), sh("echo never", dir.path())],
            sink.clone(),
        )
        .unwrap();

        let handle = pm.wait_handle("fail").unwrap();
        assert_eq!(handle.wait(), 3);

        let lines = sink.lines.lock().unwrap();
        assert!(!lines.iter().any(|(l, _)| l == "never"));
    }

    #[test]
    fn finished_id_can_be_reused() {
        let dir = tempfile::tempdir().unwrap();
        let sink = Arc::new(TestSink::default());
        let mut pm = ProcessManager::new();

        pm.spawn_command("job", dir.path(), "sh", &["-c", "true"], sink.clone())
            .unwrap();
        pm.wait_handle("job").unwrap().wait();

        // The first run has exited; the same id must be reusable.
        pm.spawn_command("job", dir.path(), "sh", &["-c", "true"], sink.clone())
            .unwrap();
        assert_eq!(pm.wait_handle("job").unwrap().wait(), 0);
    }

    #[test]
    fn running_id_cannot_be_reused() {
        let dir = tempfile::tempdir().unwrap();
        let sink = Arc::new(TestSink::default());
        let mut pm = ProcessManager::new();

        pm.spawn_command("busy", dir.path(), "sh", &["-c", "sleep 5"], sink.clone())
            .unwrap();
        let second = pm.spawn_command("busy", dir.path(), "sh", &["-c", "true"], sink.clone());
        assert!(second.is_err());
        pm.kill("busy").unwrap();
    }

    #[test]
    fn kill_stops_a_running_process_and_frees_the_id() {
        let dir = tempfile::tempdir().unwrap();
        let sink = Arc::new(TestSink::default());
        let mut pm = ProcessManager::new();

        pm.spawn_command("dev", dir.path(), "sh", &["-c", "sleep 30"], sink.clone())
            .unwrap();
        let handle = pm.wait_handle("dev").unwrap();

        pm.kill("dev").unwrap();
        // The worker observes the death promptly.
        assert!(handle.wait_timeout(Duration::from_secs(5)).is_some());
        assert!(!pm.is_running("dev"));

        // And the id is immediately reusable.
        pm.spawn_command("dev", dir.path(), "sh", &["-c", "true"], sink.clone())
            .unwrap();
        assert_eq!(pm.wait_handle("dev").unwrap().wait(), 0);
    }

    #[test]
    fn kill_cancels_pending_steps() {
        let dir = tempfile::tempdir().unwrap();
        let sink = Arc::new(TestSink::default());
        let mut pm = ProcessManager::new();

        pm.spawn_sequence(
            "chain",
            vec![sh("sleep 30", dir.path()), sh("echo never", dir.path())],
            sink.clone(),
        )
        .unwrap();
        let handle = pm.wait_handle("chain").unwrap();
        pm.kill("chain").unwrap();
        assert!(handle.wait_timeout(Duration::from_secs(5)).is_some());

        let lines = sink.lines.lock().unwrap();
        assert!(!lines.iter().any(|(l, _)| l == "never"));
    }
}

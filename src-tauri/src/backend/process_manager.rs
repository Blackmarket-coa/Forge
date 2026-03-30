use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::backend::errors::ForgeError;

#[derive(Default)]
pub struct ProcessManager {
    processes: HashMap<String, ManagedProcess>,
}

struct ManagedProcess {
    child: Arc<Mutex<Child>>,
    pid: u32,
}

#[derive(Serialize, Clone)]
struct ProcessOutputPayload {
    process_id: String,
    data: String,
    is_stderr: bool,
}

#[derive(Serialize, Clone)]
struct ProcessExitPayload {
    id: String,
    code: i32,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    pub fn spawn_command(
        &mut self,
        id: &str,
        working_dir: &Path,
        command: &str,
        args: &[&str],
        app_handle: &AppHandle,
    ) -> Result<u32, ForgeError> {
        if self.is_running(id) {
            return Err(ForgeError::ProcessError(format!(
                "process already running for id: {id}"
            )));
        }

        let mut child = Command::new(command)
            .args(args)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ForgeError::ProcessError(format!("failed to spawn process: {e}")))?;

        let pid = child.id();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let child_arc = Arc::new(Mutex::new(child));
        self.processes.insert(
            id.to_string(),
            ManagedProcess {
                child: Arc::clone(&child_arc),
                pid,
            },
        );

        if let Some(stdout) = stdout {
            let id_out = id.to_string();
            let app_out = app_handle.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = app_out.emit(
                        "process-output",
                        ProcessOutputPayload {
                            process_id: id_out.clone(),
                            data: line,
                            is_stderr: false,
                        },
                    );
                }
            });
        }

        if let Some(stderr) = stderr {
            let id_err = id.to_string();
            let app_err = app_handle.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = app_err.emit(
                        "process-output",
                        ProcessOutputPayload {
                            process_id: id_err.clone(),
                            data: line,
                            is_stderr: true,
                        },
                    );
                }
            });
        }

        {
            let id_exit = id.to_string();
            let app_exit = app_handle.clone();
            let child_wait = Arc::clone(&child_arc);
            std::thread::spawn(move || {
                let code = child_wait
                    .lock()
                    .ok()
                    .and_then(|mut c| c.wait().ok())
                    .and_then(|status| status.code())
                    .unwrap_or(-1);

                let _ = app_exit.emit("process-exit", ProcessExitPayload { id: id_exit, code });
            });
        }

        Ok(pid)
    }

    pub fn wait_for_exit(&self, id: &str) -> Result<i32, ForgeError> {
        let managed = self
            .processes
            .get(id)
            .ok_or_else(|| ForgeError::ProjectNotFound(id.to_string()))?;

        let mut lock = managed
            .child
            .lock()
            .map_err(|_| ForgeError::ProcessError("failed to lock process for wait".to_string()))?;

        let status = lock
            .wait()
            .map_err(|e| ForgeError::ProcessError(format!("failed to wait process: {e}")))?;

        Ok(status.code().unwrap_or(-1))
    }

    pub fn kill(&mut self, id: &str) -> Result<(), ForgeError> {
        let managed = self
            .processes
            .remove(id)
            .ok_or_else(|| ForgeError::ProjectNotFound(id.to_string()))?;

        let mut lock = managed
            .child
            .lock()
            .map_err(|_| ForgeError::ProcessError("failed to lock process for kill".to_string()))?;

        lock.kill()
            .map_err(|e| ForgeError::ProcessError(format!("failed to kill process: {e}")))?;

        Ok(())
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.processes.contains_key(id)
    }

    pub fn pid(&self, id: &str) -> Option<u32> {
        self.processes.get(id).map(|p| p.pid)
    }
}

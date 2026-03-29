FROM node:20-bullseye-slim

WORKDIR /app
ADD package.json .
RUN yarn install
ADD . .
RUN yarn build

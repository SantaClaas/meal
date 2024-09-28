# Build the app
FROM node:21 AS build-app
WORKDIR /app-build

# Installs pnpm as it is set as package manager in package.json
RUN corepack enable

# Copy over manifests
# Workspace root
COPY ./package.json ./package.json
COPY ./pnpm-workspace.yaml ./pnpm-workspace.yaml
COPY ./pnpm-lock.yaml ./pnpm-lock.yaml

# App specific
COPY ./app/package.json ./app/package.json

# Install dependencies
RUN pnpm install

# Copy over the source to build the application
COPY ./app ./app

# Build and cache
RUN pnpm run build

# Build the delivery service containing the API and hosting the app
FROM rust:1.81 AS build-delivery-service

# Create a new empty shell project
RUN USER=root cargo new --bin delivery-service
WORKDIR /delivery-service

# Copy over manifests
# The lock file is in the workspace root. Should probably use the workspace Cargo.toml too but this works so far
COPY ./Cargo.lock ./Cargo.lock
COPY ./delivery-service/Cargo.toml ./Cargo.toml

# Install serialport crate dependencies
RUN apt-get update && apt-get install -y libudev-dev

# Build and cache the dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy over the source to build the application
COPY ./delivery-service/src ./src

# Build the application
RUN rm ./target/release/deps/delivery-service*
RUN cargo build --release

# Final base image
FROM debian:bookworm-slim AS final

# Copy the build artifacts from the build stage
COPY --from=build-delivery-service /delivery-service/target/release/delivery-service .
# The ./app directory is where the delivery service looks for when app static files are requested
COPY --from=build-app /app/dist ./app

EXPOSE 3000
# Set the startup command to run the application
CMD ["./delivery-service"]
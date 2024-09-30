ARG RUST_VERSION=1.81

FROM rust:${RUST_VERSION} AS builder
# Create a new empty shell project to enable downloading dependencies before building for caching
RUN USER=root cargo new --bin delivery-service
RUN USER=root cargo new --name meal-core --lib core
RUN USER=root cargo new --name pumpe --bin pumpe
RUN mkdir tugboat
# RUN USER=root cargo new --name pumpe --bin tugboat

# Copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./core/Cargo.toml ./core/Cargo.toml
COPY ./delivery-service/Cargo.toml ./delivery-service/Cargo.toml
COPY ./pumpe/Cargo.toml ./pumpe/Cargo.toml
# COPY ./tugboat/Cargo.toml ./tugboat/Cargo.toml

# Build and cache the dependencies
RUN cargo build --release --exclude tugboat

# Remove build artifacts that are not needed in next steps
RUN rm ./target/release/deps/delivery_service*
RUN rm ./target/release/deps/meal_core*
RUN rm ./target/release/deps/pumpe*

# Don't remove source code as that breaks building from cargo workspace
# TODO test if true


# Build core rust wasm
FROM builder AS build-core

# Install wasm-pack
#TODO improve this step as it builds from source and is slow but downloading built binary would need hash check and might change
RUN cargo install wasm-pack

# Copy over the source code to build the library
RUN rm core/src/*.rs
COPY ./core/src ./core/src

# Build the application
RUN wasm-pack build --release ./core




# Build tool to compress files
FROM builder AS build-pumpe

# Copy over the source code to build the application
RUN rm ./pumpe/src/*.rs
COPY ./pumpe/src ./pumpe/src

# Build the application
RUN cargo build --package pumpe --release




# Build the app
FROM node:lts AS build-app
WORKDIR /app-build
# Copy build artifacts from core rust wasm build
COPY --from=build-core /core/pkg ./core/pkg
# Copy tool to compress files
COPY --from=build-pumpe /target/release/pumpe ./pumpe

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

# Compress files
RUN ./pumpe ./app/dist




# Build the delivery service containing the API and hosting the app
FROM builder AS build-delivery-service

# Copy over the source code to build the application
RUN rm ./delivery-service/src/*.rs
COPY ./delivery-service/src ./delivery-service/src

# Build the application
RUN cargo build --package delivery-service --release




# Final base image
FROM debian:bookworm-slim AS final

# Copy the build artifacts from the build stage
COPY --from=build-delivery-service /target/release/delivery-service .
# The ./app directory is where the delivery service looks for when app static files are requested
COPY --from=build-app /app-build/app/dist ./app

EXPOSE 3000
# Set the startup command to run the application
CMD ["./delivery-service"]
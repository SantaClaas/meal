ARG RUST_VERSION=1.81
#TODO test out creating a base image to build from to remove duplicated steps
# Build core rust wasm
FROM rust:${RUST_VERSION} AS build-core
WORKDIR /core-build
# Create a new empty shell project to enable downloading dependencies before building for caching
RUN USER=root cargo new --bin delivery-service
RUN USER=root cargo new --name meal-core --lib core

# Install wasm-pack
#TODO improve this step as it builds from source and is slow but downloading built binary would need hash check and might change
RUN cargo install wasm-pack

# Copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./core/Cargo.toml ./core/Cargo.toml
# Need to copy both or cargo workspace will not find it and fail
COPY ./delivery-service/Cargo.toml ./delivery-service/Cargo.toml

# Build and cache the dependencies
RUN cargo build --package meal-core --release
RUN rm core/src/*.rs

# Copy over the source to build the library
COPY ./core/src ./core/src

# Build the application
RUN rm ./target/release/deps/meal_core*
RUN wasm-pack build --release ./core


# Build the app
FROM node:21 AS build-app
WORKDIR /app-build

# Copy build artifacts from core rust wasm build
COPY --from=build-core /core-build/core/pkg ./core/pkg
RUN ls -la ./core/pkg

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
FROM rust:${RUST_VERSION} AS build-delivery-service

WORKDIR /delivery-service-build
# Create a new empty shell project to enable downloading dependencies before building for caching
RUN USER=root cargo new --bin delivery-service
RUN USER=root cargo new --name meal-core --lib core

# Copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./delivery-service/Cargo.toml ./delivery-service/Cargo.toml
# Need to copy both or cargo workspace will not find it and fail
COPY ./core/Cargo.toml ./core/Cargo.toml

# Build and cache the dependencies
RUN cargo build --package delivery-service --release
RUN rm ./delivery-service/src/*.rs

# Copy over the source to build the application
COPY ./delivery-service/src ./delivery-service/src

# Build the application
RUN rm ./target/release/deps/delivery_service*
RUN cargo build --package delivery-service --release

# Final base image
FROM debian:bookworm-slim AS final

# Copy the build artifacts from the build stage
COPY --from=build-delivery-service /delivery-service-build/target/release/delivery-service .
# The ./app directory is where the delivery service looks for when app static files are requested
COPY --from=build-app /app-build/app/dist ./app

EXPOSE 3000
# Set the startup command to run the application
CMD ["./delivery-service"]
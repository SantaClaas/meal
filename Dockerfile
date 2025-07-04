ARG RUST_VERSION=1.87

FROM rust:${RUST_VERSION} AS builder

# Install cmake to build libsql
# Run apt-get in one step for caching https://docs.docker.com/build/building/best-practices/#apt-get
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project to enable downloading dependencies before building for caching
RUN USER=root cargo new --bin delivery-service
RUN USER=root cargo new --name meal-core --lib core
RUN USER=root cargo new --name pumpe --bin pumpe
RUN USER=root cargo new --name tugboat --bin tugboat

# Copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./core/Cargo.toml ./core/Cargo.toml
COPY ./delivery-service/Cargo.toml ./delivery-service/Cargo.toml
COPY ./pumpe/Cargo.toml ./pumpe/Cargo.toml
COPY ./tugboat/Cargo.toml ./tugboat/Cargo.toml


# Build and cache the dependencies
RUN cargo build --release

# Remove build artifacts that are not needed in next steps
RUN rm ./target/release/deps/delivery_service*
RUN rm ./target/release/deps/meal_core*
RUN rm ./target/release/deps/pumpe*
RUN rm ./target/release/deps/tugboat*

# Don't remove source code as that breaks building from cargo workspace




# Build core rust wasm
FROM builder AS build-core

# Install wasm-pack
#TODO improve this step as it builds from source and is slow but downloading built binary would need hash check and might change
RUN cargo install wasm-pack

# Copy over the source code to build the library
RUN rm core/src/*.rs
COPY ./core/src ./core/src

# Build the application
RUN wasm-pack build --target web --release ./core




# Build tool to compress files
FROM builder AS build-pumpe

# Copy over the source code to build the application
RUN rm ./pumpe/src/*.rs
COPY ./pumpe/src ./pumpe/src

# Build the application
RUN cargo build --package pumpe --release


FROM node:lts-slim AS node-base
# Updating corepack to not have signing keys out of date
RUN npm install --global corepack@latest
# Installs pnpm(?)
RUN corepack enable
#TODO use pnpm fetch to cache dependencies and only use pnpm install --offline to not refetch




# Build the app
FROM node-base AS build-app
WORKDIR /app-build
# Copy build artifacts from core rust wasm build
COPY --from=build-core /core/pkg ./core/pkg
# Copy tool to compress files
COPY --from=build-pumpe /target/release/pumpe ./pumpe

# Copy over manifests
# Workspace root
COPY ./package.json ./package.json
COPY ./pnpm-workspace.yaml ./pnpm-workspace.yaml
COPY ./pnpm-lock.yaml ./pnpm-lock.yaml

# App specific
COPY ./app/package.json ./app/package.json

# Install dependencies
RUN pnpm install --frozen-lockfile

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




# The final image can be specified with the target when building the docker image
# Final base image
FROM debian:bookworm-slim AS final-delivery-service

# Copy the build artifacts from the build stage
COPY --from=build-delivery-service /target/release/delivery-service .
# The ./app directory is where the delivery service looks for when app static files are requested
COPY --from=build-app /app-build/app/dist ./app

EXPOSE 3000
# Set the startup command to run the application
CMD ["./delivery-service"]


# Build tugboat styles
FROM node-base AS build-tugboat-styles
WORKDIR /tugboat-styles

# Copy tool to compress files
COPY --from=build-pumpe /target/release/pumpe ./pumpe

#TODO this should be the lock file not the workspace file?
COPY ./pnpm-workspace.yaml ./pnpm-workspace.yaml
RUN pnpm fetch


# Copy over manifests
# Workspace root
COPY ./package.json ./package.json
COPY ./pnpm-lock.yaml ./pnpm-lock.yaml

# App specific
COPY ./tugboat/package.json ./tugboat/package.json

# Install dependencies
RUN pnpm install --frozen-lockfile

# Copy over the source to build the styles
COPY ./tugboat ./tugboat

# Build and cache
RUN cd tugboat \
    && pnpx @tailwindcss/cli --input ./app.css --output ./public/app.css --minify \
    && cd ..

# Compress files
RUN ./pumpe ./tugboat/public


FROM builder AS build-tugboat

# Copy over the source code to build the application
RUN rm ./tugboat/src/*.rs
COPY ./tugboat/src ./tugboat/src
COPY ./tugboat/templates ./tugboat/templates


# Build the application
RUN cargo build --package tugboat --release




# The final image can be specified with the target when building the docker image
# Final base image
FROM debian:bookworm-slim AS final-tugboat

RUN apt-get update && apt-get install -y --no-install-recommends \
    # OpenSSL dependency for reqwest used by bitwarden
    libssl-dev \
    # Certificates needed to make HTTPS requests
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the build artifacts from the build stage
COPY --from=build-tugboat-styles /tugboat-styles/tugboat/public ./public
COPY --from=build-tugboat /target/release/tugboat .

EXPOSE 3001
# Set the startup command to run the application
CMD ["./tugboat"]

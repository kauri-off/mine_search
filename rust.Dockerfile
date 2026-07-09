# Single build for both Rust binaries: the workspace dependency layer is cooked
# once and shared. CI builds each image by target (--target worker / backend).
# Prebuilt cargo-chef image avoids compiling cargo-chef from source.
FROM lukemathwalker/cargo-chef:latest-rust-1.93.1-trixie AS chef
WORKDIR /app

FROM chef AS planner
# Whole workspace is needed so the recipe captures every member's dependencies.
COPY Cargo.toml Cargo.lock ./
COPY worker/Cargo.toml worker/
COPY worker/src worker/src
COPY backend/Cargo.toml backend/
COPY backend/src backend/src
COPY proto/Cargo.toml proto/build.rs proto/
COPY proto/src proto/src
COPY proto/proto proto/proto
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# libpq is needed by the backend's diesel/postgres native linkage.
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq-dev libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# cargo-chef reconstructs the workspace skeleton from the recipe and builds the
# cached dependency layer. This layer is only invalidated when dependencies change.
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Bring in the real sources and build both binaries in one pass so they share
# the compiled workspace dependencies.
COPY Cargo.toml Cargo.lock ./
COPY worker/Cargo.toml worker/
COPY worker/src worker/src
COPY backend/Cargo.toml backend/
COPY backend/src backend/src
COPY backend/migrations backend/migrations
COPY proto/Cargo.toml proto/build.rs proto/
COPY proto/src proto/src
COPY proto/proto proto/proto

RUN cargo build --release -p worker -p backend

# Runtime base must not be older than the build image's distro (trixie): a
# binary linked against trixie's glibc can fail on bookworm's older glibc.
FROM debian:trixie-slim AS worker
WORKDIR /app

COPY --from=builder /app/target/release/worker .

CMD ["./worker"]

FROM debian:trixie-slim AS backend
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend .

EXPOSE 3000

CMD ["./backend"]

FROM rust:1.93.1-trixie AS builder
WORKDIR /app

# Workspace root manifest + single lockfile.
COPY Cargo.toml Cargo.lock ./

# All workspace members must be present so Cargo can parse the workspace graph.
COPY worker/Cargo.toml worker/
COPY worker/src worker/src

COPY backend/Cargo.toml backend/
COPY backend/src backend/src

COPY proto/Cargo.toml proto/build.rs proto/
COPY proto/src proto/src
COPY proto/proto proto/proto

# `-p worker` compiles only the worker crate, not backend.
RUN cargo build --release -p worker

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/worker .

CMD ["./worker"]

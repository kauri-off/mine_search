FROM rust:1.93.1-trixie AS builder
RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config
WORKDIR /app

# Workspace root manifest + single lockfile.
COPY Cargo.toml Cargo.lock ./

# All workspace members must be present so Cargo can parse the workspace graph.
COPY db_schema/Cargo.toml db_schema/
COPY db_schema/src db_schema/src
COPY db_schema/migrations db_schema/migrations

COPY worker/Cargo.toml worker/
COPY worker/src worker/src

COPY backend/Cargo.toml backend/
COPY backend/src backend/src

COPY proto/Cargo.toml proto/build.rs proto/
COPY proto/src proto/src
COPY proto/proto proto/proto

# `-p worker` compiles only worker + db_schema, not backend.
RUN cargo build --release -p worker

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/worker .

CMD ["./worker"]

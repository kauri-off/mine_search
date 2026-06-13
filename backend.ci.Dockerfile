FROM rust:1.93.1-trixie AS builder
RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config
WORKDIR /app

# Workspace root manifest + single lockfile.
COPY Cargo.toml Cargo.lock ./

# All workspace members must be present so Cargo can parse the workspace graph.
COPY db_schema/Cargo.toml db_schema/
COPY db_schema/src db_schema/src
COPY db_schema/migrations db_schema/migrations

COPY backend/Cargo.toml backend/
COPY backend/src backend/src

COPY worker/Cargo.toml worker/
COPY worker/src worker/src

# `-p backend` compiles only backend + db_schema, not worker.
RUN cargo build --release -p backend

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/backend .

EXPOSE 3000
CMD ["./backend"]

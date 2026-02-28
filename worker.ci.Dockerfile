FROM rust:1.93.1-trixie AS builder
RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config
WORKDIR /app

COPY db_schema/src db_schema/src
COPY db_schema/Cargo.toml db_schema/

COPY worker/src worker/src
COPY worker/Cargo.toml worker/Cargo.toml

WORKDIR /app/worker
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/worker/target/release/worker .

CMD ["./worker"]

FROM rust:1.93.1-trixie AS builder
RUN apt-get update && apt-get install -y cmake
WORKDIR /app

COPY db_schema/src db_schema/src
COPY db_schema/Cargo.toml db_schema/

COPY worker/src worker/src
COPY worker/Cargo.toml worker/Cargo.toml

WORKDIR /app/worker
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/worker/target/release/worker .

CMD ["./worker"]

FROM rust:1.93.1-trixie AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner

COPY db_schema/src db_schema/src
COPY db_schema/Cargo.toml db_schema/
COPY worker/src worker/src
COPY worker/Cargo.toml worker/

WORKDIR /app/worker
RUN cargo generate-lockfile
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y cmake

WORKDIR /app/worker
COPY --from=planner /app/worker/recipe.json recipe.json
COPY --from=planner /app/worker/Cargo.lock Cargo.lock

COPY db_schema/src /app/db_schema/src
COPY db_schema/Cargo.toml /app/db_schema/

RUN cargo chef cook --release --recipe-path recipe.json

WORKDIR /app
COPY worker/src worker/src
COPY worker/Cargo.toml worker/

WORKDIR /app/worker
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/worker/target/release/worker .

CMD ["./worker"]
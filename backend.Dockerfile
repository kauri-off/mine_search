FROM rust:1.93.1-trixie AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner

COPY db_schema/src db_schema/src
COPY db_schema/Cargo.toml db_schema/
COPY backend/src backend/src
COPY backend/Cargo.toml backend/

WORKDIR /app/backend
RUN cargo generate-lockfile
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config

WORKDIR /app/backend
COPY --from=planner /app/backend/recipe.json recipe.json
COPY --from=planner /app/backend/Cargo.lock Cargo.lock

COPY db_schema/src /app/db_schema/src
COPY db_schema/Cargo.toml /app/db_schema/

RUN cargo chef cook --release --recipe-path recipe.json

WORKDIR /app
COPY backend/src backend/src
COPY backend/Cargo.toml backend/Cargo.toml

WORKDIR /app/backend
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/backend/target/release/backend .

EXPOSE 3000

CMD ["./backend"]
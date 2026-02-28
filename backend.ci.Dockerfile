FROM rust:1.93.1-trixie AS builder
RUN apt-get update && apt-get install -y cmake
WORKDIR /app

COPY db_schema/src db_schema/src
COPY db_schema/Cargo.toml db_schema/

COPY backend/src backend/src
COPY backend/Cargo.toml backend/Cargo.toml

WORKDIR /app/backend
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/backend/target/release/backend .

EXPOSE 3000
CMD ["./backend"]

FROM rust AS builder

WORKDIR /app

RUN apt-get update -y && apt-get install cmake -y

COPY worker worker
COPY db_schema db_schema

WORKDIR /app/worker
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/worker/target/release/mine_search .

CMD ["./mine_search"]
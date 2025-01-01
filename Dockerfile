FROM rust AS builder

WORKDIR /app

RUN apt-get update -y && apt-get install cmake -y

RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml .

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY src src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/mine_search .

CMD ["./mine_search"]
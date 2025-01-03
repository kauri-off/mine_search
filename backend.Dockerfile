FROM rust AS builder

WORKDIR /app

RUN apt-get update -y && apt-get install cmake -y

COPY backend backend
COPY db_schema db_schema

WORKDIR /app/backend
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/backend/target/release/backend .

EXPOSE 3000

CMD ["./backend"]
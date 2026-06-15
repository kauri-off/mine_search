FROM rust:1.93.1-trixie AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
# Whole workspace is needed so the recipe captures every member's dependencies.
COPY Cargo.toml Cargo.lock ./
COPY db_schema/Cargo.toml db_schema/
COPY db_schema/src db_schema/src
COPY worker/Cargo.toml worker/
COPY worker/src worker/src
COPY backend/Cargo.toml backend/
COPY backend/src backend/src
COPY proto/Cargo.toml proto/build.rs proto/
COPY proto/src proto/src
COPY proto/proto proto/proto
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt-get update && apt-get install -y libpq-dev libssl-dev pkg-config

WORKDIR /app
# cargo-chef reconstructs the workspace skeleton from the recipe and builds the
# cached dependency layer. This layer is only invalidated when dependencies change.
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Bring in the real sources and build the binary. `-p worker` builds only
# worker + db_schema.
COPY Cargo.toml Cargo.lock ./
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

RUN cargo build --release -p worker

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/worker .

CMD ["./worker"]

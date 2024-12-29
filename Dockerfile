FROM rust

WORKDIR /app

RUN echo "fn main() {}" > dummy.rs
COPY Cargo.toml .

RUN sed -i 's#src/main.rs#dummy.rs#' Cargo.toml
RUN cargo build --release
RUN sed -i 's#dummy.rs#src/main.rs#' Cargo.toml

COPY src src

RUN cargo build --release
RUN echo "DATABASE_URL=/data/db/database.db" >> .env

CMD ["target/release/mc_lookup"]
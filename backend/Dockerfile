FROM rust:1.83-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/urban-mobility-api /usr/local/bin/urban-mobility-api

EXPOSE 8000
CMD ["urban-mobility-api"]


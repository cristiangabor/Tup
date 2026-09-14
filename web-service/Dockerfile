FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

COPY --from=builder /app/target/release/web-service /usr/local/bin/web-service
COPY static /app/static

EXPOSE 3001

CMD ["web-service"]
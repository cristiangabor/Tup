FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

COPY --from=builder /app/target/release/Tup /usr/local/bin/tup-service

ENV PORT=3000

EXPOSE 3000

CMD ["tup-service"]
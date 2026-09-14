# Build a small Linux binary suitable for a TrueNAS SCALE container.
FROM rust:1.98-slim-bookworm AS builder

WORKDIR /app

# Some Rust TLS dependencies compile small native libraries on Linux.
RUN apt-get update \
    && apt-get install --no-install-recommends -y build-essential cmake perl pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim

# TLS certificates are needed for the backend's HTTPS requests to FoxESS.
RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 foxess

COPY --from=builder /app/target/release/foxess-api /usr/local/bin/foxess-api

USER foxess
EXPOSE 3000

CMD ["foxess-api"]

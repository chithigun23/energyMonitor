# Build a small Linux binary suitable for a TrueNAS SCALE container.
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Build dependencies are cached until Cargo.toml or Cargo.lock changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && printf 'fn main() {}\n' > src/main.rs && cargo build --release --locked

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

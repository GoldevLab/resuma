# Multi-stage build for the Resuma docs site (example-website).
# Build from repo root: docker build -t resuma-docs .

FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY examples ./examples
COPY runtime ./runtime

RUN cargo build --release -p example-website

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home resuma

WORKDIR /app
COPY --from=builder /app/target/release/website /app/website
COPY --from=builder /app/examples/website/src/pages /app/pages
RUN chown -R resuma:resuma /app

USER resuma

ENV HOST=0.0.0.0
ENV PORT=3000
ENV RESUMA_PAGES_ROOT=/app/pages
ENV RESUMA_ENV=production
ENV RESUMA_TRUST_PROXY=1

EXPOSE 3000
CMD ["/app/website"]

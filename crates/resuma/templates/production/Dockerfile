FROM rust:1.91-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/%NAME% /app/server
ENV RESUMA_ENV=production
ENV RESUMA_ADDR=0.0.0.0:8080
ENV RESUMA_TRUST_PROXY=1
EXPOSE 8080
USER 65532:65532
CMD ["/app/server"]

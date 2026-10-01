# syntax=docker/dockerfile:1.6
FROM rust:1.78-slim AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev protobuf-compiler cmake && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
COPY engine engine
COPY proto proto
ENV SQLX_OFFLINE=true
RUN cargo build --release -p integration-gateway

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=builder /build/target/release/integration-gateway /app/integration-gateway
USER nonroot:nonroot
EXPOSE 8080 50051 8443
ENTRYPOINT ["/app/integration-gateway"]

# syntax=docker/dockerfile:1.6
# Multi-stage build for compliance-governor (Rust Core Engine)
# Final image is distroless for minimal attack surface.

FROM rust:1.78-slim AS builder
WORKDIR /build

# System deps for sqlx + tonic
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev protobuf-compiler cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
COPY engine engine
COPY proto proto

ENV SQLX_OFFLINE=true
RUN cargo build --release -p compliance-governor

# ---- Runtime ----
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app

COPY --from=builder /build/target/release/compliance-governor /app/compliance-governor

USER nonroot:nonroot
EXPOSE 8080 50051

ENTRYPOINT ["/app/compliance-governor"]

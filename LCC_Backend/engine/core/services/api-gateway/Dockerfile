FROM rust:1.81-slim AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . /build
RUN cargo build --release --bin lcc-api-gateway

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home --shell /bin/bash lcc
COPY --from=builder /build/target/release/lcc-api-gateway /usr/local/bin/lcc-api-gateway
USER lcc:lcc
EXPOSE 8080
CMD ["lcc-api-gateway"]

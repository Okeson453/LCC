FROM rust:1.81-slim AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . /build
RUN cargo build --release --bin lcc-opportunity-discovery-worker

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates && rm -rf /var/lib/apt/lists/* && useradd --uid 10001 --create-home --shell /bin/bash lcc
COPY --from=builder /build/target/release/lcc-opportunity-discovery-worker /usr/local/bin/lcc-opportunity-discovery-worker
USER lcc:lcc
CMD ["lcc-opportunity-discovery-worker"]

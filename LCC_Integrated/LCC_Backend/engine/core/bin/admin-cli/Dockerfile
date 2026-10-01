FROM rust:1.81-slim AS builder
WORKDIR /build
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . /build
RUN cargo build --release --bin lcc-admin-cli
FROM debian:bookworm-slim
RUN useradd --uid 10001 --create-home --shell /bin/bash lcc
COPY --from=builder /build/target/release/lcc-admin-cli /usr/local/bin/lcc-admin-cli
USER lcc:lcc
ENTRYPOINT ["lcc-admin-cli"]

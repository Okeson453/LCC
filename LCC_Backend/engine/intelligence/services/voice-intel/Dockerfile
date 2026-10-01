# syntax=docker/dockerfile:1.6
FROM python:3.12-slim AS builder
WORKDIR /build
ENV PIP_DISABLE_PIP_VERSION_CHECK=1 PIP_NO_CACHE_DIR=1 UV_LINK_MODE=copy
RUN pip install uv==0.4.18
COPY packages packages
COPY engine/intelligence engine/intelligence
WORKDIR /build/engine/intelligence/services/voice-intel
RUN uv sync --frozen

FROM python:3.12-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 libssl3 curl ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home --shell /bin/bash lcc
COPY --from=builder /build /build
USER lcc:lcc
WORKDIR /build/engine/intelligence/services/voice-intel
ENV PYTHONUNBUFFERED=1 PATH=/build/.venv/bin:$PATH
EXPOSE 8092
ENV HTTP_PORT=8092
ENV SERVICE_PACKAGE=voice_intel
CMD ["bash", "-c", "uv run --frozen python -m voice_intel"]

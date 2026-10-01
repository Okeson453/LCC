# syntax=docker/dockerfile:1.6
FROM python:3.12-slim AS builder
WORKDIR /build

ENV PIP_DISABLE_PIP_VERSION_CHECK=1 \
    PIP_NO_CACHE_DIR=1 \
    UV_LINK_MODE=copy

# Install uv for fast lockfile-driven install.
RUN pip install uv==0.4.18

# Layer-copy pyproject + uv.lock
COPY packages packages
COPY engine/intelligence engine/intelligence

RUN uv sync --frozen --no-install-project
RUN uv sync --frozen

# ---- Runtime ----
FROM python:3.12-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 libssl3 curl ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home --shell /bin/bash lcc

COPY --from=builder /build /build
WORKDIR /build/engine/intelligence

USER lcc:lcc
ENV PYTHONUNBUFFERED=1 \
    PATH=/build/.venv/bin:$PATH

EXPOSE 8080 50051
CMD ["bash", "-c", "uv run --frozen uvicorn $SERVICE_PACKAGE.app:app --host 0.0.0.0 --port ${HTTP_PORT:-8080}"]

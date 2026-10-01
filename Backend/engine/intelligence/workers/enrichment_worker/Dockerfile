# syntax=docker/dockerfile:1.6
FROM python:3.12-slim
WORKDIR /build
RUN pip install uv==0.4.18
COPY packages packages
COPY engine/intelligence engine/intelligence
WORKDIR /build/engine/intelligence/workers/enrichment_worker
RUN uv sync --frozen
ENV PYTHONUNBUFFERED=1 PATH=/build/.venv/bin:$PATH
# Create the non-root user the services also use. Without this the
# image fails at start with "unable to find user specified".
RUN useradd --uid 10001 --create-home --shell /bin/bash lcc
USER lcc:lcc
CMD ["bash", "-c", "uv run --frozen python -m src.__main__"]

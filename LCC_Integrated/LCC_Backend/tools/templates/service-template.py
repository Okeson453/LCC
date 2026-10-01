"""LCC service template.

Replace `MY_SERVICE` with the actual service name. Provides:
- FastAPI app
- Settings via pydantic-settings
- /healthz and /readyz
- audit emission hook
"""

from __future__ import annotations

import os

import uvicorn
from fastapi import FastAPI
from pydantic_settings import BaseSettings, SettingsConfigDict

from intelligence_common.logging import configure_logging, get_logger


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="MY_SERVICE_", extra="ignore")
    service_name: str = "my-service"
    log_level: str = "info"
    http_port: int = 8080


settings = Settings()
configure_logging(settings.service_name, settings.log_level)
log = get_logger(__name__)

app = FastAPI(title=settings.service_name)


@app.get("/healthz")
async def healthz() -> dict[str, str]:
    return {"status": "ok"}


@app.get("/readyz")
async def readyz() -> dict[str, str]:
    return {"status": "ready"}


if __name__ == "__main__":
    uvicorn.run("MY_SERVICE.__main__:app", host="0.0.0.0", port=settings.http_port)

"""KB types + enums + exceptions."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class KBCategory(str, Enum):
    ACHIEVEMENTS = "achievements"
    SKILLS = "skills"
    PROJECTS = "projects"
    EXPERIENCE = "experience"
    VOICE_SAMPLES = "voice_samples"
    CASE_STUDIES = "case_studies"
    TESTIMONIALS = "testimonials"
    PREFERENCES = "preferences"


@dataclass
class KBRecord:
    """A single KB chunk returned by retrieval."""

    id: str
    member_id: str
    category: KBCategory
    title: str
    fact: str
    chunk_index: int = 0
    chunk_count: int = 1
    score: float = 0.0
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class KBCreateRequest:
    """A request to create a new KB record (chunked + embedded)."""

    member_id: str
    category: KBCategory
    title: str
    body: str
    source_kind: str = "manual"
    source_url: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


class KBClientError(Exception):
    """Base KB client error."""


class KBRetrievalError(KBClientError):
    """Retrieval failed (vector store down, etc.)."""


class KBDuplicateError(KBClientError):
    """Record is a duplicate of an existing one."""

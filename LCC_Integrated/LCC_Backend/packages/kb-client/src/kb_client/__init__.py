"""kb-client — Knowledge Base repository + retriever interfaces."""

from kb_client.types import (
    KBCategory,
    KBRecord,
    KBClientError,
    KBRetrievalError,
    KBDuplicateError,
    KBCreateRequest,
)
from kb_client.repository import KBRepository, PgKBRepository, InMemoryKBRepository
from kb_client.retriever import KBRetriever

__all__ = [
    "KBCategory",
    "KBRecord",
    "KBClientError",
    "KBRetrievalError",
    "KBDuplicateError",
    "KBCreateRequest",
    "KBRepository",
    "PgKBRepository",
    "InMemoryKBRepository",
    "KBRetriever",
]

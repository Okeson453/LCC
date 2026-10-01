"""vector-db — Vector DB abstraction over Qdrant + Pinecone."""

from vector_db.base import (
    SearchHit,
    VectorDB,
    VectorDBError,
    VectorSearchResult,
    VectorStoreUnavailable,
)
from vector_db.qdrant import QdrantClientWrapper

__all__ = [
    "SearchHit",
    "VectorDB",
    "VectorDBError",
    "VectorSearchResult",
    "VectorStoreUnavailable",
    "QdrantClientWrapper",
]

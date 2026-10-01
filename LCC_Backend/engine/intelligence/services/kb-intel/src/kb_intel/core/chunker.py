"""KB chunker — splits long KB facts into retrieval-sized chunks.

Each chunk is small enough to embed efficiently but large enough to
preserve context. We use a 1200-char window with a 200-char overlap.
"""

from __future__ import annotations

import re


def chunk(text: str, max_chars: int = 1200, overlap: int = 200) -> list[str]:
    """Split text into overlapping chunks by sentence boundary first."""
    text = text.strip()
    if len(text) <= max_chars:
        return [text] if text else []

    # Split on sentence boundaries.
    sentences = re.split(r"(?<=[.!?])\s+", text)
    chunks: list[str] = []
    current: list[str] = []
    current_len = 0

    for s in sentences:
        s_len = len(s)
        if s_len > max_chars:
            # Hard split — single sentence longer than max_chars.
            for i in range(0, s_len, max_chars - overlap):
                chunks.append(s[i : i + max_chars])
            continue

        if current_len + s_len > max_chars and current:
            chunks.append(" ".join(current))
            # Overlap: keep last `overlap` chars worth of content.
            tail = " ".join(current)
            if len(tail) > overlap:
                # Keep last few sentences that fit in overlap.
                tail_text = tail[-overlap:]
                current = [tail_text]
                current_len = len(tail_text)
            else:
                current = list(current)
                current_len = sum(len(x) for x in current)
            current.append(s)
            current_len += s_len
        else:
            current.append(s)
            current_len += s_len

    if current:
        chunks.append(" ".join(current))

    return chunks


__all__ = ["chunk"]

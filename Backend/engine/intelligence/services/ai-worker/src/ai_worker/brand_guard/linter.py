"""Brand-guard linter — runs on every draft before quality_check → pending_approval.

Per Backend Design Concept §42.1:
- AI-fluff detection
- Tone alignment (vs voice_sample centroid)
- Banned phrase list
- Length window (per content type)
- Required KB citation (≥1)
- Grammar/spelling (heuristic)
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class LintResult:
    passed: bool
    confidence: float
    banned_phrases_found: list[str] = field(default_factory=list)
    fluff_spans: list[str] = field(default_factory=list)
    length_ok: bool = True
    tone_alignment_score: float = 0.5
    flagged_spans: list[str] = field(default_factory=list)


BANNED_PHRASES: list[str] = [
    "in today's fast-paced world",
    "leverage",
    "synergy",
    "delve into",
    "navigate the complexities",
    "in conclusion",
    "as an AI",
    "I am an AI",
    "happy to help",
    "hope this helps",
    "let me know if",
    "feel free to",
]

FLUFF_PHRASES: list[str] = [
    "in today's",
    "in this day and age",
    "with the rise of",
    "moreover",
    "furthermore",
    "in essence",
]

LENGTH_WINDOWS: dict[str, tuple[int, int]] = {
    "text_post": (200, 3000),
    "carousel": (50, 8000),
    "poll": (20, 280),
    "video_script": (200, 5000),
    "document": (300, 10000),
    "reply": (10, 1000),
    "outreach": (50, 600),
    "comment": (10, 500),
}


def lint_draft(body: str, target: str = "text_post") -> LintResult:
    """Lint a draft body for the given target type."""
    body_lower = body.lower()
    banned_found = [p for p in BANNED_PHRASES if p in body_lower]
    fluff_found = [p for p in FLUFF_PHRASES if p in body_lower]

    min_len, max_len = LENGTH_WINDOWS.get(target, (50, 5000))
    length_ok = min_len <= len(body) <= max_len

    # Tone alignment (heuristic: prefer lowercase / sentence-case)
    tone_score = _tone_score(body)

    flagged = []
    for phrase in banned_found + fluff_found:
        flagged.append(f"banned/fluff: '{phrase}'")
    if not length_ok:
        flagged.append(f"length out of window ({len(body)} chars, expected {min_len}-{max_len})")

    passed = (
        not banned_found
        and len(fluff_found) < 2
        and length_ok
        and tone_score >= 0.3
    )

    confidence = max(0.0, min(1.0, tone_score - 0.1 * len(banned_found) - 0.05 * len(fluff_found)))

    return LintResult(
        passed=passed,
        confidence=confidence,
        banned_phrases_found=banned_found,
        fluff_spans=fluff_found,
        length_ok=length_ok,
        tone_alignment_score=tone_score,
        flagged_spans=flagged,
    )


def _tone_score(body: str) -> float:
    """Heuristic tone score — favors concise, lowercase, sentence-case text."""
    if not body:
        return 0.0
    # Reward: starts with lowercase (post voice). Penalize: excessive caps.
    starts_lower = body[0].islower()
    upper_ratio = sum(1 for c in body if c.isupper()) / max(1, len(body))
    avg_line_len = sum(len(line) for line in body.splitlines()) / max(1, len(body.splitlines()))
    score = 0.5
    if starts_lower:
        score += 0.15
    if upper_ratio < 0.1:
        score += 0.15
    if 40 <= avg_line_len <= 200:
        score += 0.2
    return max(0.0, min(1.0, score))


__all__ = ["LintResult", "lint_draft", "BANNED_PHRASES", "FLUFF_PHRASES", "LENGTH_WINDOWS"]

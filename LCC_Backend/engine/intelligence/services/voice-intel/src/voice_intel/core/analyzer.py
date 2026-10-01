"""Voice analyzer — extracts a style fingerprint from text samples.

The fingerprint is a dict of:
- avg_sentence_length (chars)
- avg_word_length (chars)
- lowercase_ratio (0..1)
- emoji_density (per 100 chars)
- hashtag_density (per 100 chars)
- line_break_style ("dense" | "airy")
- tone_descriptor (string)
- top_unigrams (list[str])

This is a deterministic, no-LLM fingerprint. The voice_train_worker uses
it to compute centroid vectors for similarity scoring.
"""

from __future__ import annotations

import re
from collections import Counter
from dataclasses import dataclass


@dataclass
class VoiceFingerprint:
    avg_sentence_length: float
    avg_word_length: float
    lowercase_ratio: float
    emoji_density: float
    hashtag_density: float
    line_break_style: str
    tone_descriptor: str
    top_unigrams: list[str]

    def to_dict(self) -> dict:
        return {
            "avg_sentence_length": self.avg_sentence_length,
            "avg_word_length": self.avg_word_length,
            "lowercase_ratio": self.lowercase_ratio,
            "emoji_density": self.emoji_density,
            "hashtag_density": self.hashtag_density,
            "line_break_style": self.line_break_style,
            "tone_descriptor": self.tone_descriptor,
            "top_unigrams": self.top_unigrams,
        }


_EMOJI_RE = re.compile(r"[\U0001F300-\U0001FAFF\U00002700-\U000027BF]")
_HASHTAG_RE = re.compile(r"#\w+")
_WORD_RE = re.compile(r"\w+")
_SENTENCE_RE = re.compile(r"(?<=[.!?])\s+|\n+")


def _stopwords() -> set[str]:
    return {
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "should",
        "can", "could", "may", "might", "must", "shall", "to", "of", "in",
        "for", "on", "with", "at", "by", "from", "as", "into", "through",
        "and", "but", "or", "nor", "so", "yet", "both", "either", "neither",
        "i", "you", "he", "she", "it", "we", "they", "this", "that", "these",
        "those", "my", "your", "his", "her", "its", "our", "their",
        "what", "which", "who", "whom", "whose", "why", "how", "all", "any",
        "about", "than", "after", "before", "between", "during", "without",
        "within", "along", "across", "behind", "beyond", "near",
        "if", "when", "where", "while", "because", "since", "until",
        "no", "yes", "not", "also", "just", "very", "much", "more", "less",
        "one", "two", "three", "four", "five", "first", "second", "third",
        "new", "good", "great", "big", "small", "long", "short",
    }


def fingerprint(text: str) -> VoiceFingerprint:
    """Compute a voice fingerprint from a text (or sample-concatenated string)."""
    if not text.strip():
        return VoiceFingerprint(
            avg_sentence_length=0.0, avg_word_length=0.0, lowercase_ratio=1.0,
            emoji_density=0.0, hashtag_density=0.0, line_break_style="dense",
            tone_descriptor="empty", top_unigrams=[],
        )

    chars = len(text)
    words = _WORD_RE.findall(text)
    sentences = [s for s in _SENTENCE_RE.split(text) if s.strip()]

    avg_sent = sum(len(s) for s in sentences) / max(1, len(sentences))
    avg_word = sum(len(w) for w in words) / max(1, len(words))
    lower_ratio = sum(1 for c in text if c.islower()) / max(1, sum(1 for c in text if c.isalpha()))

    n_emojis = len(_EMOJI_RE.findall(text))
    n_hashtags = len(_HASHTAG_RE.findall(text))

    emoji_density = (n_emojis / chars) * 100 if chars else 0
    hashtag_density = (n_hashtags / chars) * 100 if chars else 0

    avg_line_len = sum(len(line) for line in text.splitlines()) / max(1, len(text.splitlines()))
    line_break_style = "airy" if avg_line_len < 80 else "dense"

    # Tone descriptor: derived from a small ruleset.
    if emoji_density > 0.5:
        tone = "playful"
    elif hashtag_density > 0.5:
        tone = "promotional"
    elif avg_sent > 200:
        tone = "essayistic"
    elif avg_sent < 50:
        tone = "punchy"
    else:
        tone = "neutral"

    sw = _stopwords()
    unigrams = [w.lower() for w in words if w.lower() not in sw and len(w) > 3]
    top_unigrams = [w for w, _ in Counter(unigrams).most_common(10)]

    return VoiceFingerprint(
        avg_sentence_length=avg_sent,
        avg_word_length=avg_word,
        lowercase_ratio=lower_ratio,
        emoji_density=emoji_density,
        hashtag_density=hashtag_density,
        line_break_style=line_break_style,
        tone_descriptor=tone,
        top_unigrams=top_unigrams,
    )


__all__ = ["fingerprint", "VoiceFingerprint"]

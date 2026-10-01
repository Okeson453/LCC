# voice-intel

Voice profile extraction, fingerprinting, training-data collection.

## Endpoints

- `POST /internal/intelligence/voice/fingerprint` — compute a fingerprint from samples.
- `POST /internal/intelligence/voice/samples` — add an approved post to the voice_samples collection.
- `GET /healthz`, `/readyz`.

## Fingerprint fields

- `avg_sentence_length` (chars)
- `avg_word_length` (chars)
- `lowercase_ratio`
- `emoji_density` (per 100 chars)
- `hashtag_density` (per 100 chars)
- `line_break_style` ("airy" | "dense")
- `tone_descriptor` ("playful" | "punchy" | "essayistic" | "promotional" | "neutral")
- `top_unigrams` (list of 10)

The `voice_train_worker` periodically recomputes the centroid vector for the
member and publishes `voice.profile.updated`.

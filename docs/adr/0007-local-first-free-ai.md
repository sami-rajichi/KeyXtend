# 7. AI features are local-first and free

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
There is no budget. Users need good Arabic, French and English speech-to-text, word prediction and OCR. Privacy matters.

## Decision
- **Speech-to-text:**
  - Default: whisper.cpp (MIT code and weights) through whisper-rs, with models downloaded on demand:
    - `large-v3-turbo` q5_0, 574 MB;
    - `small` q5_1, 190 MB for weak PCs.
  - Optional cloud, bring your own key, off by default: Groq free tier, Cloudflare Workers AI, Azure Speech F0.
  - Gemini's free tier only with a data-use warning.
  - Rejected: Parakeet and Canary (no Arabic); Moonshine Arabic (licence).
- **Prediction:** our own trie and n-gram engine, seeded from wordfreq data (CC-BY-SA, separate files) and learning on the device. No neural model in v1.
- **OCR:** Windows OCR first, then Tesseract `tessdata_fast` (Apache-2.0), then PaddleOCR ONNX for Arabic.
- **Camera (future):** `nokhwa` and `ort` with face-landmark ONNX models. Do not use the MediaPipe Tasks runtime (it sends metrics to Google).

## Consequences
- Never ship an API key. Keys are stored with the OS secret store.
- A model benchmark button tells the user which model their PC runs fast enough.

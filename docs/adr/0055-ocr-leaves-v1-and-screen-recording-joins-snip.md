# 55. OCR leaves v1, and screen recording joins Snip

- **Date:** 2026-11-23
- **Status:** Accepted. Supersedes ADR-0007's OCR line and ADR-0004's `ocr` job; their other lines and jobs stand. Replaces spec §8.4 and the roadmap's Phase 12 with [`2026-11-23-snip-design.md`](../superpowers/specs/2026-11-23-snip-design.md).

## Context

- P12 was "Snip + OCR": Region, Window, Full screen, Scrolling and a timer, an editor with copy text (OCR), and pin on screen (spec §8.4, roadmap Phase 12).
- ADR-0007 planned three OCR engines: Windows OCR, then Tesseract `tessdata_fast`, then PaddleOCR through ONNX for Arabic. ADR-0004 gave them a worker job, `ocr`.
- With one finger and no keyboard, Print Screen, Win+Shift+S and Win+Alt+R are out of reach, and Windows' Snipping Tool needs a drag for a region.
- On 2026-11-23 the owner asked for "a Snipping Tool, not all of it": pictures, short videos, a light editor, and nothing kept unless asked.

## Decision

Decided by the owner on 2026-11-23, with the P12 design and plan.

1. **No OCR in v1.** No engine, no `ocr` job and no `OcrEngine` port. OCR may come back later as an idea, with its own ADR.
2. **Recording joins Snip.** Video region and Video full screen record at `fps` into an MP4, with the microphone and the computer's sound each off by default. A video can be copied, saved, played and sped up; nothing else edits it.
3. **Pictures as a Snipping Tool takes them.** Region, Window and Full screen, a strip of Copy, Edit, Save and Close, and an editor of eleven tools with undo and redo.
4. **Out of P12.** Pin on screen, scrolling capture, a timer for pictures, freeform shapes, drawing on, trimming or pausing a video, GIF, auto-save, sharing and uploading.
5. **A new name.** P12 is "Snip and screen recording" in `docs/FEATURES.md` and the roadmap, still v0.8.

## Consequences

- Tesseract, PaddleOCR and an ONNX runtime never enter the download or the worker; `THIRD_PARTY.md` marks Tesseract as dropped.
- Nothing in P12 turns a picture into text, so a snip stays pixels.
- The worker loses the `ocr` job and gains `record` and `pick` (ADR-0056).
- The kernel's port list drops `OcrEngine`; `ARCHITECTURE.md`, `README.md`, `docs/security/keyxtend-worker.md` and `docs/items/P12.md` lose their OCR lines with P12's docs.

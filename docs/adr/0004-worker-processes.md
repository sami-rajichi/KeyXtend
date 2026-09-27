# 4. Untrusted data and network access live in worker processes

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
The uiAccess keyboard process is privileged (ADR-0003). Several features touch outside data:
- voice: microphone, cloud APIs, model downloads;
- OCR;
- clipboard images and files from other apps;
- update checks.

A bug in an image decoder or HTTP stack inside the uiAccess process would give an attacker UIPI bypass.

## Decision
- One binary, `keyxtend-worker.exe`, with subcommands `voice`, `ocr`, `media` and `update`. Each job runs in its own process with **normal user rights**.
- The main process spawns workers and passes an inherited pipe handle. Messages are versioned enums (`kx-ipc`) with size limits, validation and rate limits, and the decoding is fuzzed.
- Workers are started on demand, stopped when idle, and restarted with backoff by the health monitor.
- Only the voice and update workers may open network connections.

## Consequences
- Some latency for OCR and thumbnails, which is acceptable.
- `cargo xtask tidy` forbids HTTP/TLS and media-decoding crates in the `keyxtend` app's dependency tree.

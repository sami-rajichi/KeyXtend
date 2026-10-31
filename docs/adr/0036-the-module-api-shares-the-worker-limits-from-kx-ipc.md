# 36. The module API shares the worker's limits from kx-ipc

- **Date:** 2026-10-31
- **Status:** Accepted.

## Context

- The clipboard's settings check sizes `worker_mb` from two facts the worker's protocol owns: how much a payload read holds as it grows (`READ_TIMES`), and what the PNG decoder may allocate (`DECODE_BYTES`).
- A `kx-mod-*` crate may depend only on `kx-module-api`, so the check could not read them, and copies of them could drift from the worker.
- `kx-ipc` is a leaf crate: it depends only on `thiserror` and holds no `unsafe`, network or decoding code.

## Decision

1. `kx-module-api` depends on `kx-ipc`, and its pictures port re-exports `READ_TIMES` and `DECODE_BYTES` as `WORKER_READ_TIMES` and `WORKER_DECODE_BYTES`.
2. `kx-module-api` takes only these limits from `kx-ipc`; frames, readers and command words stay out of the module API.

## Consequences

- A change to the worker's limits reaches the clipboard's settings check without a second edit, and tests on both sides tie them.
- `kx-ipc` must stay a leaf crate; a dependency added to it reaches every module.

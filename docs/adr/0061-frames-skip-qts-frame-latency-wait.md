# 61. Frames skip Qt's frame-latency wait

- **Date:** 2026-11-27
- **Status:** Accepted (owner, 2026-11-27). Supersedes the last clause of ADR-0025 decision 2, "The other frame knobs keep Qt's own values", for `frame_latency` only; ADR-0025's other lines and ADR-0026 stand.

## Context

- With `[ui] frame_latency = 2` (Qt's own), each frame waits at its start on Direct3D's frame-latency object, for up to 1 s. On a window shown again whose last frame was still pending when it hid, that wait stalls the Qt thread for up to 1 s a frame.
- The owner's live debug logs on the installed build, 2026-11-27:
  - With 2, the countdown after a Cancel blocked the Qt thread for about 4 s, showing only its first second, and re-shown Snip windows were ready in about 1225 ms (229 ms for the first one after start).
  - With 0, every countdown second drew on time (15–18 frames each), and re-shown Snip windows were ready in about 211 ms.
- The owner chose 0, and typing felt normal.

## Decision

- `[ui] frame_latency` defaults to 0, so Qt makes no frame-latency object and skips that wait.
- `present_vsync` (ADR-0026) and `vblank_updates` stand: each Present still waits for a refresh, and frames still start on it.

## Consequences

- A window shown again draws at once, and the countdown shows every second.
- Direct3D's own queue of 3 frames bounds Present in place of Qt's cap of 2.
- G8 lit is re-run on the next live run, since ADR-0025's key light figures were measured with Qt's 2.
- The knob stays, with its range 0 to 3, so 2 can be measured again.

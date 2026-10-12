# 26. Frames keep the refresh wait; a press still draws first

- **Date:** 2026-10-12
- **Status:** Accepted. Supersedes ADR-0025's `present_vsync = false`; keeps its `press_frame = "now"` and its key light target.

## Context

- ADR-0025 made frames present without vsync (`[ui] present_vsync = false`) and planned a fall-back to `true` if the full G8 showed dropped frames or idle CPU over budget.
- The full G8 on the installed build at 144 Hz and default settings counted 142 dropped frames (201 lost of 944 presented), with PresentMon showing Composed: Flip at sync interval 0: the keyboard handed Windows more frames than the screen showed.
- The key light still met its target in that run: a median of 14.2 ms and a slowest press of 21.8 ms, against 21.1 and 28.1 ms.

## Decision

- `present_vsync` defaults to `true` again, so each frame waits for a refresh; `press_frame` stays `"now"`.
- The knob stays, off by default, for measuring.

## Consequences

- Animations drop no frames, as spec §10 asks.
- About a quarter of presses light one refresh later than without vsync (15 of 60 against 6 in ADR-0025's runs), still within the target: a median of about 14.5 ms and a slowest of about 22 ms.

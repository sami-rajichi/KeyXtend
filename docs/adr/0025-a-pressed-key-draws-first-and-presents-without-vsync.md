# 25. A pressed key draws first and presents without vsync

- **Date:** 2026-10-12
- **Status:** Accepted. Changes spec §10's key light target; keeps ADR-0021's drawing and ADR-0024's press.

## Context

- Spec §10 asked a press to light its key on the next screen frame, and G8 held the slowest of its presses to 2 frames plus its screen copy (bug H7, 2026-10-12).
- By default a pressed key acted first and drew after (`[ui] press_frame = "after"`), and each frame waited for the screen's refresh to show (`present_vsync = true`).
- Measured on the installed uiAccess build at 144 Hz and 125 %, in the owner's look, 3 runs of 20 presses each, with `kx-gates g8 lit` and `g8 osk`:
  - The old defaults: a median of 28.1 ms. The key's action ended 6.2 ms after the click (typing sent at 1.8 ms, then the full state redraw, about 4.4 ms); its frame then waited for the vblank to 10.6 ms and swapped at 13.2 ms.
  - `press_frame = "now"`: a median of 14.5 ms and a worst of 22.3 ms, with 15 of 60 presses one refresh late. The press frame swapped 2.5 ms after the click, and every press typed exactly once.
  - `"now"` with `present_vsync = false`: medians of 14.3–15.1 ms and worsts of 21.0–21.8 ms, with 6 of 60 late.
  - Windows' On-Screen Keyboard, timed the same way: medians of 14.2–14.5 ms and worsts of 21.6–25.7 ms, with 6 of 60 late. It failed the old "worst within 2 frames" rule in 3 of 3 runs.

## Decision

Decided by the owner on 2026-10-12.

1. **The press frame draws first.** `[ui] press_frame` defaults to `"now"`: the frame that shows the key pressed is drawn at once, before the key acts. The key still types at the press (ADR-0024).
2. **Presents do not wait for vsync.** `[ui] present_vsync` defaults to `false`, so each window asks for swap interval 0. The composed flip still shows each frame at DWM's refresh. The other frame knobs keep Qt's own values.
3. **The target, as Windows' own keyboard.** The median press lights within the next frame, and the slowest within one refresh more.
4. **G8 judges both,** each as its own part and each plus the median screen copy: the median press against `[g8] highlight_frames` (2) frames as `highlight_p50_ms`, and the slowest against `highlight_worst_frames` (3) as `highlight_worst_ms`. `g8 lit` and `g8 osk` print the same two parts.

## Consequences

- Each press draws one frame more: the press frame, then the redraw after the key acted.
- Typing now waits one event-loop turn for the press frame, about 2.5 ms after the click, still far inside spec §10's 30 ms from click to character.
- With no wait at Present, animations may draw more frames than the screen shows.
- The fall-back: if the full G8 shows dropped frames, or idle CPU over its budget, with `present_vsync = false`, that default goes back to `true`: its one line in `defaults.toml`, with the tests and docs that state it (`docs/items/P3.md` lists them); `press_frame` stays `"now"`.
- ADR-0021's one device and basic render loop stay: the press frame is drawn on the GUI thread through `Keyboard.drawNow`, which refuses inside another frame.
- G8's result has `highlight_p50_ms` and `highlight_worst_ms` in place of the one `highlight_ms`.
- Each press still re-reads the whole keyboard state (`restate`, about 4.4 ms on the development PC); a partial update is a P13 note.

## Alternatives

- **Keep `"after"` with vsync:** a median of 28.1 ms, about twice Windows' own keyboard; refused.
- **`"first"`:** the key types only once its press frame is out, so typing waits for a frame; refused, as keys type at the press (ADR-0024).
- **`"now"` with vsync:** 15 of 60 presses one refresh late, against Windows' 6; it stays the fall-back.
- **Keep "the slowest within 2 frames":** Windows' own keyboard fails it in every run, so it is not a fair target on this compositor; refused.

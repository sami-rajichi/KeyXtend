# 45. The dictation bar takes clicks but never focus

- **Date:** 2026-11-14
- **Status:** Accepted. Supersedes ADR-0013's window rule "click-through windows (shadows, ring, caption bar) add `Qt.WindowTransparentForInput`" for the caption bar only; shadows and the ring stay click-through.

## Context

- ADR-0013 made the caption bar click-through, like the shadows and the pointer ring. The spike's caption only showed text.
- The new bar has buttons in every state: Cancel, the gear and Stop while listening, and Copy, Undo insert and Close with the text ([`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md), The bar).
- The bar must stay when the keyboard hides, so Stop still works. An owned window hides with its owner.
- The notice in `crates/kx-ui/qml/App.qml` already has no owner (`transientParent: null`), so it shows while the keyboard is hidden.
- Every window we own except Settings must never take focus from the app (ADR-0013).

## Decision

Decided with the owner on 2026-11-10; the design is [`2026-11-10-voice-typing-design.md`](../superpowers/specs/2026-11-10-voice-typing-design.md) (The bar).

1. **Its own window.** The dictation bar is a window of its own with the no-focus flags of ADR-0013 and without `Qt.WindowTransparentForInput`. It takes clicks and never takes focus; nothing calls `requestActivate()` on it.
2. **No owner.** Like the notice in `App.qml`, it has no owner (`transientParent: null`), so it stays when the keyboard hides or is minimised.
3. **Guarded like a pop-up.** The window guard covers it as it covers a pop-up: it keeps `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`, answers `MA_NOACTIVATE` and lifts the bar back into the topmost band. A guard run names it as `top` when it shows.

## Consequences

- The guard refuses click-through on a window that took clicks when first guarded, so the bar keeps its clicks.
- The bar adds a guarded window, so the guard's open z-order finding is looked at first (`docs/items/P11.md`, plan task 4). If it stays open, the hands-on check on the signed build covers the bar.
- Qt Quick tests check that the bar takes clicks and never focus, and G22 checks that focus stays in the app.
- The caption leaves the owned windows in [`docs/architecture/qt-windows.md`](../architecture/qt-windows.md) when the bar is built.

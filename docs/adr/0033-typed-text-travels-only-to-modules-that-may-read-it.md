# 33. Typed text travels only to modules that may read it

- **Date:** 2026-10-24
- **Status:** Accepted.

## Context

- Word prediction must hear what the keyboard types and must type a picked word. Until P7, typed text never went on the bus (ARCHITECTURE.md, the kernel contract).
- The bus's capability gate is opt-in: an event without `Event::CAPABILITY` reaches any module (P3).
- `KeyboardFaceKey` names no capability, yet its actions type keys, so any module that requires the face could type (from P3, 2026-10-01).
- `AssistNote` and `PointerMoved` carry click points and name no capability, so any module could hear them (from P4, 2026-10-14).
- `FaceAction::FlipCaps` exists only in debug builds, so the public enum's shape depends on the build kind (from P3, 2026-10-11).
- The architecture's example of an intercept event was "the focused field is a password field", which would stop learning.

## Decision

Decided with the approval of P7's plan on 2026-10-24.

1. **`ReadFocusedText` gates typed text.** `Typed` (what the keyboard typed), `TypeWord` (a request to erase and type a word), `Suggested` (the chips), `ChipPicked` and `WordsAsk` all name it. The keyboard, the ui module and the prediction module declare it.
2. **The keyboard stays the only typist.** Prediction never holds `InjectInput`: it sends `TypeWord`, and the keyboard types it through the app's route, its latches, its dead-key rules and its `text_cap`.
3. **`KeyboardUi` gates the face and the pointer notes.** A new capability, held only by the ui module, gates `KeyboardFaceKey`, `AssistNote` and `PointerMoved`.
4. **A field service, not an intercept chain.** `FieldWatch`, gated by `ReadFocusedText`, tells the focused box's kind (text, password, unknown or none) and reads at most N words before the cursor. Each module that needs it asks it, so no ordering between modules matters.
5. **`FlipCaps` goes.** The memory check's churn turns Caps over through a debug wrapper of the layout source in the composition root and redraws with `relabel()`.

## Consequences

- A module without `ReadFocusedText` cannot hear, ask for or suggest typed text; contract tests check every gated event.
- The vault (P9) and the selection helper (P10) reuse `FieldWatch` and `TypeWord`.
- ARCHITECTURE.md's intercept example changes to a real need when one comes; no module intercepts today.

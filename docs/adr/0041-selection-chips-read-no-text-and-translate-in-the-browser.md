# 41. Selection chips read no text, and Translate opens the browser

- **Date:** 2026-11-07
- **Status:** Accepted. Replaces spec §8.3 with [`2026-11-07-selection-chips-design.md`](../superpowers/specs/2026-11-07-selection-chips-design.md), and the roadmap's "gesture fallback" for Phase 10.

## Context

- The owner wants Copy, Cut and Paste next to selected text, Paste at a caret just clicked into, and a Translate chip (`docs/items/P10.md`, 2026-11-07).
- The spike's pill polled every 300 ms and read one character to tell a selection from a caret. Polling breaks the idle CPU budget (spec §10), and a read is more than the chips need.
- Free translation without a key, as extensions do it, goes through unpublished Google or Microsoft endpoints, which a sold app must not rely on. Free official services need an account and a key per user. Zen and Firefox translate on the PC, with an engine and model downloads that P11 builds for voice.
- The keyboard is the only typist (ADR-0032, ADR-0038), and the clipboard's paste request lets every latch go, which a chip must not do.

## Decision

1. **No text before a click.** The selection port tells the box's kind, whether its selection holds text, its line boxes and its caret box, from events (focus and text-selection changes, our presses and releases), never a timer. Only Translate reads the selection, once, at its click.
2. **No gesture fallback.** An app that does not tell its selection gets no chips (owner).
3. **Capabilities.** The selection module holds `ReadFocusedText` (the port, Translate's read and `AimedLine`), a new `Selection` (the gate of the chips' events, held by the module, the keyboard and the ui) and a new `OpenLink` (hand a link to the user's browser, held by the module alone). It holds no `InjectInput`.
4. **Edit keys through the keyboard.** The module asks the keyboard for Copy, Cut or Paste; the keyboard sends exactly that shortcut with no latch added and every latch left as it was. In a terminal it sends the table's terminal ids, which copy and paste but never stop a program.
5. **Translate opens the browser's translate page.** The link comes from a compiled-in site table, holds at most `translate_chars` characters, and is handed to the desktop shell, so the browser never inherits our rights. The first click asks once, because the text leaves the PC; the module acts only on an answer to its own question, within `consent_s`.
6. **Presses per asker.** `PressesWanted` names its asker, and the mouse module tells presses while any asker wants them; it also tells `PointerReleased`, with no point, so a selection made with the mouse is looked at when the button comes up.

## Consequences

- Selected text never reaches KeyXtend unless the user clicks Translate and says yes.
- Apps that hide their selection from accessibility tools get no chips; the Shortcuts layer still works there.
- The in-app translation box waits for F8, on this PC, reusing P11's engine and download parts (`docs/items/F8.md`).
- `ReadFocusedText` gains a holder, so the app's capability pins change with this record.

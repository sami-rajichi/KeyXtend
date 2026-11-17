# 48. Voice pastes through the clipboard history, which puts the clipboard back

- **Date:** 2026-11-17
- **Status:** Accepted. Adds the events `PasteText` and `TextPasted` to ADR-0034's paste route; ADR-0034 decision 2 stands, so the keyboard stays the only typist.

## Context

- The owner asked on 2026-11-17 for a choice between typing a dictated text and pasting it at once ([`2026-11-10-phase-11-voice-cloud.md`](../superpowers/plans/2026-11-10-phase-11-voice-cloud.md), task 4b). A paste uses the clipboard, which belongs to the user.
- The clipboard history already sets the clipboard as ours and asks the keyboard for one paste (ADR-0034 decision 2). It alone watches the clipboard, so it alone knows what to put back.
- A copy a password manager marks private must never lose its marks. Our writer sets only our own marker, so a private copy written back would become an ordinary one.

## Decision

1. **The voice module asks; the clipboard history pastes.** The voice module sends `PasteText` with its round, the text as a wiped `Secret` and its wait, `paste_ms`. Both events need `WriteClipboard`, so the asker never gains a way to read the clipboard.
2. **One round counter.** The clipboard history keeps what the clipboard holds, writes the text as ours, and asks the keyboard for one paste (`PasteAsk`) on the counter History's Insert uses. So a lend and an insert never run together, and the keyboard's answer always names one of them.
3. **The answer comes at once.** `TextPasted` follows the keyboard's `Pasted` right away; false means not all of the paste went. A paste may start only in the first half of the asker's wait, so the answer comes before the asker gives up.
4. **The clipboard goes back.** After `paste_gap_ms`, even after a paste not all sent, the history writes back what the clipboard held, as ours: an entry whole, or a text alone up to `back_chars`. A copy, clear or private mark another app made meanwhile stays.
5. **A private copy is never touched.** A mark another app set, a `skip_apps` copy, or a clipboard not seen since the start makes the history answer no before it reads or writes anything. A start-up clipboard may hold a private copy the watch never told.
6. **Voice types on any no.** A no, or no answer within `paste_ms`, types the text once, and a late answer is dropped. A password box is always typed.

## Other platforms

- The events and the round are neutral. Each platform's watch must tell a private mark as `Marked`, as Windows' does (P15).

## Consequences

- No module but the clipboard history reads or writes back the clipboard for a paste, and neither clipboard history keeps the dictated text.
- Until the first copy or clear after the start, Paste at once types; so does every dictation while the clipboard history is off, after `paste_ms`.
- The history tells our writes from private marks by counting echoes; a note that said whose mark it is would need the platform's watch to change.

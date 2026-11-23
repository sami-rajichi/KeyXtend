# 48. Voice pastes through the clipboard history, which puts the clipboard back

- **Date:** 2026-11-17
- **Status:** Accepted. Adds the events `PasteText` and `TextPasted` to ADR-0034's paste route; ADR-0034 decision 2 stands, so the keyboard stays the only typist.

## Context

- The owner asked on 2026-11-17 for a choice between typing a dictated text and pasting it at once ([`2026-11-10-phase-11-voice-cloud.md`](../superpowers/plans/2026-11-10-phase-11-voice-cloud.md), task 4b). A paste uses the clipboard, which belongs to the user.
- The clipboard history already sets the clipboard as ours and asks the keyboard for one paste (ADR-0034 decision 2). It alone watches the clipboard, so it alone knows what to put back.
- A copy a password manager marks private must never lose its marks. Our writer sets only the number markers, so a copy with a presence marker written back would become an ordinary one.

## Decision

1. **The voice module asks; the clipboard history pastes.** The voice module sends `PasteText` with its round, the text as a wiped `Secret` and its wait, `paste_ms`. Both events need `WriteClipboard`, so the asker never gains a way to read the clipboard.
2. **One round counter.** The clipboard history keeps what the clipboard holds, writes the text as ours, and asks the keyboard for one paste (`PasteAsk`) on the counter History's Insert uses. So a lend and an insert never run together, and the keyboard's answer always names one of them.
3. **The answer comes at once.** `TextPasted` follows the keyboard's `Pasted` right away; false means not all of the paste went. A paste may start only in the first half of the asker's wait, so the answer comes before the asker gives up.
4. **The clipboard goes back.** After `paste_gap_ms`, even after a paste not all sent, the history writes back what the clipboard held, as ours: an entry whole, or a text the history cannot name alone, up to `back_chars`. A kept entry it cannot read whole answers no instead, so its other formats are never dropped, and a copy, clear or private mark another app made meanwhile stays.
5. **A private copy is never touched.** The history reads the clipboard only through `Clipboard::unmarked_text`, which checks the presence markers (`ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore`) in the same open as the read. With one there it answers no before reading or writing anything. Our writes set only `CanIncludeInClipboardHistory` and `CanUploadToCloudClipboard` to 0, so an unmarked text written back stays as private as it was; this holds from the first dictation, at any board state. A skipped app's copy is never put back either: after a note from a `skip_apps` app, or a change the watch could not read, the history answers no until another app's copy or a clear, so a password manager's clear timer firing in the gap never leaves its password there.
6. **Voice types on any no.** A no, or no answer within `paste_ms`, types the text once, and a late answer is dropped. A password box is always typed.

## Other platforms

- The events and the round are neutral. Each platform's watch must tell a private mark as `Marked`, and its `unmarked_text` must check that platform's private markers in the same read, as Windows' does (P15).

## Consequences

- No module but the clipboard history reads or writes back the clipboard for a paste, and neither clipboard history keeps the dictated text.
- Paste at once works from the first dictation, though a start-up clipboard with no text, which may be a picture, still types. It also types while a private or skipped app's copy is on the clipboard, and after `paste_ms` while the clipboard history is off.
- The history counts our writes' echoes only to put an entry back whole. A stale count can put back a stale entry's other formats with the clipboard's text, or write "empty" over a picture whose note it swallowed; nothing private is read, but the current clipboard is lost.

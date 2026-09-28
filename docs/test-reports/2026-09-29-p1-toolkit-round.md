# Test report: P1 toolkit test round (Qt face)

- **Dates:** 2026-09-27 to 2026-09-29
- **Windows:** Windows 11 Home Single Language 25H2, build 26200
- **Display:** 125 % scale, 1920×1080, about 165 Hz
- **Machine:** the development PC (Ryzen 7 7735HS, 24 GB, RTX 4060)
- **Scope:** the owner's hands-on checks. The automatic gate runs and their numbers are in ADR-0013.

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Switch Native, ET66 and Dolch, light and dark, beside the mock-up | Each matches the mock-up | Pass | Native passed after its colour fixes (2026-09-28) |
| 2 | Use the top bar: move, size −/+, S/M/L, resize corner, minimise to bubble, tooltips | Each works with single left clicks, no dragging | Pass | |
| 3 | Type a dead key, then a letter | The accented letter arrives | Pass | ê arrived |
| 4 | Right-click and Grab with the real mouse for 90 s each | A still hold right-clicks; Grab drops on the next click | Pass | Hook max 0.22 ms |
| 5 | Dictate with the Mic key | The caption shows without focus; the words arrive | Pass | Checkpoint accepted 2026-09-28 |
| 6 | Turn on the ring test and watch the pointer | The ring waves over the hold, bursts, and follows the pointer | Pass | 87 s at 165 fps, p99 7.4 ms |
| 7 | Open the Arabic panel from the Settings key | Right to left, paged, caret correct | Pass (function) | The look was rejected as plain and unthemed; rebuilt in P13 |
| 8 | Select text, then use the Copy pill | The pill shows without focus and copies | Pass | Its icon was added on 2026-09-29 |

**Follow-ups:** the Settings window and list panel follow the theme and the keyboard's language (P13). Ring colours in ET66 and Dolch light were darkened to reach 3:1.

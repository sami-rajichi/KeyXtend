# 40. A panel's line takes the letters only while it is aimed at

- **Date:** 2026-11-06
- **Status:** Accepted. Replaces "while a panel is open its keys go to it" in spec §3.4, §7 and §8.1, "←/→ switch tabs" in §3.3, and "panel keys tell prediction nothing" for the search lines only.

## Context

- A panel stays open until closed by hand, and our windows never take focus (ADR-0013).
- Every key went to the open panel, so a box clicked in another app showed its caret but received nothing. The owner found this in the hands-on check of P9.
- The search lines offered no word suggestions, unlike any other text box, because prediction only follows the app's box.

## Decision

1. **Aim.** The keyboard alone knows whether the open panel's line is aimed at. While it is not, every key, arrows, Enter and Esc too, goes to the app as if no panel were open. While it is, a key with Ctrl, Alt or Win latched still goes to the app as a shortcut.
2. **What aims.** A panel opening aims at its search. A press anywhere on the panel (a line, a row, a button or blank space) aims at it, so the last place clicked takes the keys, and a page with no line still takes arrows, Enter and Esc. A press inside the double-click time of the one before asks nothing, so a shaky tap after a fill never gives the aim back. A page with a form or password line aims at it when it appears, and leaving that page puts the aim back as it was, unless a later rule took it away.
3. **What takes the aim away.** A click outside our windows, the front window changing, and a fill or paste from the panel. On a form or password line only the click does, so a window that pops up never gets the rest of a secret.
4. **Seeing it.** Only an aimed line draws the caret and the focus ring. Windows still draws the app's own caret, which we cannot hide while focus stays there (ADR-0013).
5. **Presses.** The keyboard asks the mouse module for presses while a panel is open. Without that module, the front window check alone takes the aim away.
6. **Search lines tell prediction their text.** Each panel module tells the keyboard which kind of line shows and a search line's text, and the keyboard passes that text on only while the line is aimed. Prediction reads it as it reads a box, so the lane shows chips, a picked chip goes to the line, and nothing is ever learned.
7. **Other lines tell nothing.** Name, label, value and password lines send no text to prediction, and its lane stays empty while one is aimed. Panel keys still never publish `Typed`.
8. **A pick names its target.** A picked word says whether it was offered for a line. The keyboard drops one whose target no longer matches the aim, so a late pick never erases text in the app.
9. **A line has a caret.** With text in the line, Left and Right move the caret by one character, in the line's reading direction, and never switch filters; an empty search line still switches them. On a hidden line one press moves one dot. Letters, Space, Backspace, Delete and a chip's word act at the caret; only on the clipboard's empty search does Space still tick the highlighted row. A click on the text places the caret. Prediction hears only the text before the caret.
10. **The mouse wheel turns pages.** One notch over a panel turns its page, as the scroll pad does, and does not move the aim. It is silent. It is an extra: nothing needs it (ADR-0008).

## Consequences

- Letters follow the last click, so a panel can stay open during ordinary typing. After a click on a row, letters go to the panel's search until the next click elsewhere or a fill.
- The mouse hook runs while a panel is open, even when prediction does not ask for it.
- Without the mouse module, a click on another box in the same window is not noticed.
- Search text reaches prediction's memory while the line is aimed; it is never stored or logged.

# 49. The Voice panel takes the keys only while its page shows a line

- **Date:** 2026-11-18
- **Status:** Accepted. Narrows ADR-0040 decision 2 for the Voice panel; its other decisions apply as written.

## Context

- A cloud service's key page has three lines: the key, the address and the Other model id (ADR-0047). They took only Paste, so a key or address could not be typed on our keyboard.
- Most Voice panel pages are lists with buttons and no line. ADR-0040 decision 2 aims a panel at a press anywhere on it, so the arrows, Enter and Esc would go to a list page that does nothing with them.
- The keyboard knew two panels, the clipboard history and Quick-fill, and closed one as the other opened.

## Decision

1. **A third panel.** The keyboard counts the Voice panel among its panels. One panel shows at a time: the Voice panel opening closes the clipboard or Quick-fill panel, and either of those opening closes it.
2. **Keys only on a line.** The Voice panel takes the keys only while its page shows a line and that line is aimed at. On every other page every key goes to the app, aimed or not.
3. **Esc goes to the app.** On a line, Esc leaves the text as it is and reaches the app. Enter saves the line.
4. **Private lines.** The key, address and model lines count as private lines (ADR-0040 decision 7): prediction's lane stays empty, and only a click outside our windows takes the aim away.
5. **The key stays in the voice module.** The typed key lives only in the voice module, is wiped on Save, Paste, Remove, a page turn, leaving the page and the panel closing, and reaches the views only as a count of dots and a caret.
6. **Events.** The voice module tells which line its page shows (`VoiceLine`, none while closed), after each view and when asked. The keyboard sends the keys as `VoiceLineKey` and asks the panel to close with `VoiceClose`. All three need `Voice`.

## Consequences

- A key, an address or a model id can be typed with one finger on our keyboard, as on any other line.
- The arrows, Enter and Esc never reach a voice list page, which has no use for them.
- A voice module that stops or fails tells the keyboard its panel is gone, so the keys never go to a module that is not there.

# 57. Snip's windows take the keys only after the user's press, beside one other panel

- **Date:** 2026-11-24
- **Status:** Accepted (owner, 2026-11-25). Supersedes ADR-0040 decision 2 ("a panel opening aims") for our windows that open on their own, and ADR-0049 decision 1's "one panel shows at a time" for Snip's windows.

## Context

- Snip's windows can open with no click: the strip when a recording's time runs out or a fault ends it, and the Snip panel from the tray. Under ADR-0040 decision 2 they would take the keys from the app the user types in.
- Closing a capture so that another panel can open would lose the user's snip, so Snip's windows must be able to show beside the clipboard, Quick-fill or Voice panel.
- With two panels on screen, a press that only says "a panel" and a state that only says "aimed" cannot tell which one takes the keys or draws the caret (ADR-0040 decision 4).

## Decision

1. **Only a press aims.** Snip's windows take the keys only after the user's press on them or on the Snip key. A line that shows on its own waits unaimed, and a text line (`PanelLine::Private`) aims nothing by itself.
2. **Beside one other panel.** Snip's windows may show beside one other panel. Neither closes the other, the Snip key closes nothing, and the last one pressed takes the keys, one at a time.
3. **A press names its window.** A press on Snip's windows sends `AimSnip`, and `AimPanel` still means the open panel. A click on the Snip key before its line shows aims that line when it comes, unless an outside click or the capture's end came first.
4. **The state names the aim.** The face's state says which panel takes the keys (`aim`), so only that window draws its caret. `aimed` stays true only for the open panel's line.
5. **An ask after a restart.** The keyboard tells `SnipAsk` at start, and the snip module answers with its light and line, so a running capture keeps its key lit. All Snip events need the `Snip` gate.

## Consequences

- A recording that ends by itself never takes the letters typed into another app.
- The clipboard panel may stay open while a snip is edited, and each press moves the keys between them.
- The ui sends `AimSnip` on every press on a Snip window, but never from the countdown or the record bar, which have no line.
- The keyboard holds the `Snip` gate for the Snip key and its line, so it can also read `SnipView`'s pixels and send `SnipAct`, as its `Vault` and `Voice` gates already let it hear those modules' views (ADR-0038, ADR-0042). The owner accepted this reach (2026-11-25).

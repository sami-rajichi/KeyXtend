# 24. Controls act on the press they got

- **Date:** 2026-10-11
- **Status:** Accepted.

## Context

- The owner clicks with one finger, and the hand slips a few pixels while the button is down; one click lasts about 400 ms (bug H5, 2026-10-11).
- Qt's `MouseArea` clicks only when the release lands inside the area. A press that slipped off, even into the 3–5 px gap between keys, typed nothing, and a held repeating key stopped.
- A physical keyboard and Windows' On-Screen Keyboard type at the press (owner, 2026-10-11).
- WCAG 2.2 success criterion 2.5.2, Pointer Cancellation, lets a control act on the down-event only where that is essential; its understanding page counts a keyboard emulator as such a case.
- G2 pressed and let go in one `SendInput` batch, which no hand makes, so it never saw these lost clicks.

## Decision

Decided by the owner on 2026-10-11.

1. **Keys act at the press.** A key types, latches or drives the open language list the moment the button goes down. The language key takes its part from where it was pressed.
2. **A held key repeats until let-go**, after `repeat_delay_ms` and then once every `repeat_every_ms`, whatever the pointer does. Nothing slipped onto ever acts.
3. **Every other control acts at let-go, wherever the release lands.** It acts once, for the press it got, and the control under the release never acts.
4. **A Qt cancel acts on nothing.** It ends a burst, as the keyboard hiding or the list opening does.
5. **One shared piece.** `kx-ui`'s `Act.qml` holds the rule; keys set it to act at the press.
6. **Screen readers** keep Invoke and Toggle as before.
7. **G2 clicks like a person.** Each click holds for `[g2] press_ms` and slips `slip_px` inside the key before it lets go.

## Consequences

- A slip never loses or moves a click. Sliding off no longer takes a click back; for keys, as on a physical keyboard, nothing does.
- The keyboard's face types on `Press` (a repeating key) or `Tap` (any other), both sent at the press; `LetGo` only ends a burst, and `Leave` does the same.
- No new product setting. `kx-gates` gains `[g2] press_ms` and `slip_px`, both checked at load, and the hold stays below the default repeat delay.
- G2 takes about `press_ms` longer per click, and G4 and G26 share its clicks. G2's raw click file gains each release time.
- G8 still reads G2's clicks as one batch, so its press windows end too early; that is a P3 note.

## Alternatives

- **Act on a release inside the control** (Qt's default and Windows' push buttons): it loses the owner's slipped clicks, which was the bug.
- **A slip margin around each key:** a longer slip still loses the click, and the margins of neighbours meet in the gaps.
- **Keys at let-go, wherever the release lands:** a key would type late, unlike a physical keyboard, and a held key would need the press anyway to repeat.

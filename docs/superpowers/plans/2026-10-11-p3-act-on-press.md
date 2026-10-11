# P3: keys act at the press, so a slip never cancels them

## Goal
- A key acts the moment the button goes down, as on a real keyboard and Windows' On-Screen Keyboard, so a slip of the hand never cancels or redirects it (owner, 2026-10-11).
- Links: spec §4.6 (key repeat), ADR-0008 (no drag, no scroll), `docs/items/P3.md`.

## Scope
- **In**
  - Every key acts at the press: characters, Space, Enter, Backspace, Delete, Tab, Esc, modifiers, the language key, and a key that drives the open language list.
  - A held repeating key types at the press, then repeats after `repeat_delay_ms`, once every `repeat_every_ms`, until let-go.
    - A slip anywhere while held changes nothing.
    - A key slipped onto never acts.
  - Every other control acts at let-go, wherever the release lands (owner, 2026-10-11): the top-bar buttons and chips, the resize corner, the move/resize second click (`Mover`, `Catch`), the language list's rows and page buttons, the tray menu, the notice buttons, the bubble, and the D-pad's arrows and stop.
  - `kx-gates` G2 clicks like a person: each press is held for a time and slips a little.
  - A new ADR for the rule, and spec §4.6's lines.
- **Out**
  - Dwell clicking and hover typing (later features).
  - The dimmed D-pad (P5).

## Design notes
- **kx-mod-keyboard**
  - `repeat.rs`: `press` makes the key due at once; the first repeat comes after the delay.
  - `face/inner/held.rs` and `face.rs`: `Press` answers with the tap; `LetGo` only ends a burst.
- **kx-ui QML**
  - One shared piece, `Act.qml`: a `MouseArea` that emits `acted` with the press position, at the press (keys) or at the release wherever it lands (other controls), so the rule lives in one place.
  - `Key.qml`, `Hold.qml`, `TopBar.qml`, `Chip.qml`, `Corner.qml`, `Mover.qml`, `Catch.qml`, `LangRow.qml`, `LangList.qml`, `TrayMenu.qml`, `Notice.qml`, `Bubble.qml` and `DPad.qml` use it.
  - `Hold.qml` drops `slide`, `reenter` and `lost`. A hold ends only on let-go, a Qt cancel, the keyboard hiding, or the list opening.
  - The language key takes its back or forward part from where it was pressed.
- **Settings**
  - No new product setting; the existing `[keyboard] repeat`, `repeat_delay_ms` and `repeat_every_ms` stay.
  - kx-gates gets `[g2] press_ms` (near the owner's 400 ms) and `slip_px`, both checked.
- **Docs:** ADR-0024 (controls act on the press they got), spec §4.6, and `docs/items/P3.md`.

## Tasks
1. **Core types at the press**
   - *Files:* `crates/kx-mod-keyboard/src/repeat.rs`, `face.rs`, `face/inner/held.rs`, `tests/keyboard/held.rs`, `stops.rs`.
   - *Test first:* a press types once at once; a hold adds one at the delay and one each beat; let-go types nothing.
   - *Implement:* move the click's typing from let-go to the press, and keep the latches, dead keys and the list as they act now.
   - *Verify:* `cargo test -p kx-mod-keyboard` is green.
2. **Keys act at the press in QML**
   - *Files:* `crates/kx-ui/qml/Act.qml` (new), `Key.qml`, `Hold.qml`, `Plate.qml`, plus Quick tests.
   - *Test first:* a press near an edge that slips into the gap, onto a neighbour, or off the keyboard acts once, and the neighbour never acts.
   - *Implement:* keys use `Act`, and `Hold` keeps a hold until let-go.
   - *Verify:* Quick passes with 0 failures and no QWARN.
3. **Other controls slip-proof**
   - *Files:* the controls listed above, plus their Quick tests.
   - *Test first:* for each control, a slip out before the release still acts once, and acts only once.
   - *Implement:* each control uses `Act` and acts at let-go, wherever the release lands.
   - *Verify:* Quick and pictures (no change).
4. **G2 like a person**
   - *Files:* `tools/kx-gates/src/clicks.rs`, `g2.rs`, `kx-gates.toml`.
   - *Test first:* a click batch holds for `press_ms` and slips by `slip_px` inside the key.
   - *Implement:* the batch shape and the settings check.
   - *Verify:* `cargo test -p kx-gates`.
5. **Docs**
   - *Files:* the new ADR, spec §4.6, `docs/items/P3.md`.
   - *Verify:* tidy is clean.

## Edge cases (each has a test)
1. A press in the gap between keys: nothing acts.
2. Press A, slip onto B, release on B: A acts once, and B never acts.
3. Press A, slip off the keyboard, release outside: A acts once, and the release is ignored.
4. Hold A, slip onto B: A repeats until let-go, and B never acts.
5. A Qt cancel or the keyboard hiding during a hold ends the burst.
6. A double click on a key types twice, one per press.
7. The language key: the part is decided by where it was pressed.
8. Shift latched: the press types a capital, and Shift lets go after the press, or after the burst ends.
9. Ctrl, Alt or Win latched: a held key acts once, at the press.
10. A dead key, held: it acts once.
11. The list is open: a key that drives the list acts at the press, on the list.
12. Repeat off: a press types once.
13. Minimise, close or Quit with a slip: it acts once at let-go, and nothing behind it gets a click.
14. Screen readers: Invoke and Toggle act as before.

## Security
- Same data classes as today. Typed text stays in `Redacted`, nothing is logged, and no new capability is added.

## Manual Windows check
1. In Notepad, quick clicks with natural slips on letters, Space and Backspace: each key types once.
2. A click that slips onto a neighbour: only the pressed key types.
3. Hold Backspace and slip onto another key: it keeps deleting until let-go.
4. Top-bar buttons, list rows, a notice and the tray menu, each with a slip: each acts once.
5. French and Arabic AltGr keys with slips.
6. G2 with person-like clicks passes.

## Done when
- The Definition of Done in `docs/roadmap.md`, every edge case tested, and the owner's manual check passes.

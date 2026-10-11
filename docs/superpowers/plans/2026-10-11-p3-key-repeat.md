# P3 addendum: a held key repeats

## Goal
- Holding a typing key types it again and again until the pointer lets go, as a physical keyboard does. A short click works exactly as today.
- Owner's request (2026-10-11), built in P3 (keyboard core). Spec §4 gains §4.6; `docs/FEATURES.md` P3.

## Scope
- In:
  - repeat on hold for typing keys;
  - three settings with calm defaults;
  - the spec, the board data, the tests and the manual check.
- Out:
  - the settings window's controls and their explaining texts (P13, §8.6);
  - hold-to-fill on @ and P (P7 decides how it meets repeat);
  - Home, End and the Page keys, which are not on the board (spec §4.1; F15 adds them, and they then repeat too);
  - sounds per repeat (P4).

## Owner decisions (2026-10-11)
- Build it now, in P3.
- Calm defaults for pointer use: the first repeat after 0.8 s, then 10 a second. Each user can change both or turn repeat off. P13's settings window explains them in plain words in EN, FR and AR.
- Repeating keys: characters (letters, digits, symbols), Space, Enter, Backspace, Delete and the four arrows.
- Never repeating: Shift, Ctrl, Alt, AltGr, Win, Caps Lock, Tab, Esc, Menu, the language key, accent (dead) keys, dimmed keys and the top-bar buttons.

## Design notes
- **Crates and files:**
  - `crates/kx-mod-keyboard`: `board.toml` (a `repeat` flag on named keys; character runs always repeat), `defaults.toml`, `settings.rs`, `spec.rs` (reads the flag), a new `repeat.rs` (the state machine), and `face.rs` with `face/inner.rs` (press, hold and let-go calls);
  - `crates/kx-ui`: `src/bridges/keyboard.rs`, `qml/Key.qml` (press, release, leave), `qml/KbWindow.qml` (one hold timer for the whole window, never one per key), and the Quick stand-in `FakeKeyboard`;
  - the spec, `docs/items/P3.md`, `P9.md` and `P13.md`.
- **State machine `Repeat`** in `kx-mod-keyboard`:
  - states: idle, held (a key, since when) and repeating (a key, next due time);
  - inputs: press, tick, let go and leave;
  - outputs: "type it now" or nothing;
  - time comes from the injected `Clock` (`Mono`), so no test sleeps.
- **Short click unchanged:**
  - a key that is let go before the delay types once, on release, as today;
  - after the delay it types once, then again every interval;
  - letting go after a repeat types nothing more.
- **Stops:**
  - letting go;
  - the pointer leaving the key;
  - the keyboard hiding;
  - a panel opening;
  - a tap that sends nothing (`Stale`, `Empty`, `Later`, or an injection error).
- **Latched modifiers** stay down for the whole burst and let go when it ends, so Shift + held "a" gives "AAAA".
  - With Ctrl, Alt or Win latched, a held key types once and does not repeat, so a shortcut such as Ctrl+W can never fire many times.
- **Panels:** while one of our panels is open (spec §3.3), keys go to the panel and do not repeat.
- **The hold timer** in `KbWindow.qml` runs only while a key is held, so idle CPU stays at zero. It only asks Rust whether a repeat is due.
- **New settings in `[keyboard]`:**
  - `repeat = true`: held typing keys repeat;
  - `repeat_delay_ms = 800`: the hold before the first repeat;
  - `repeat_every_ms = 100`: the time between repeats.
  - Their allowed ranges live in `settings.rs` beside `RELABEL_MS` (delay 300–3000 ms, interval 33–1000 ms). A hand-edited value outside them is refused like the other keys.
- **Tokens and texts:** none now. P13 adds the settings window's texts.
- **Logging:** never a key id, scan code or character on this path (spec §9.3). A failed repeat logs one warning with no key.

## Tasks
1. **Spec and board data.**
   - *Files:* the spec (new §4.6 "Key repeat" and §8.6's list), `board.toml`, `spec.rs`, and the spec tests.
   - *Test first:* the board marks exactly the owner's repeating keys, every character key repeats, and a named key with no flag does not.
   - *Implement:* the `repeat` flag on Backspace, Delete, Enter, Space and the arrows, read into each key's spec.
   - *Verify:* `cargo test -p kx-mod-keyboard`, all pass.
2. **Settings.**
   - *Files:* `defaults.toml`, `settings.rs`, and the settings tests.
   - *Test first:* the defaults load; values outside the ranges are refused; `repeat = false` turns it off.
   - *Implement:* the three keys and their ranges, with one-line comments in `defaults.toml`.
   - *Verify:* `cargo test -p kx-mod-keyboard`.
3. **The `Repeat` state machine.**
   - *Files:* new `src/repeat.rs`, with tests and a property test.
   - *Test first:*
     - unit tests: release before the delay types once on release; a hold types at the delay and then every interval; letting go after a repeat types nothing; leaving stops it;
     - property test: for any press, tick and let-go sequence, it never types for a non-repeating key, never types twice in one interval, and never types after let-go.
   - *Implement:* the machine on `Mono`, with no OS calls.
   - *Verify:* `cargo test -p kx-mod-keyboard repeat`.
4. **Wire it into the face.**
   - *Files:* `face.rs`, `face/inner.rs`, `run.rs` if latches need it, and their tests.
   - *Test first:*
     - with the fake injector, a held "a" with Shift latched sends "A" several times, and Shift lets go only after the burst;
     - Ctrl latched sends once;
     - a dead key sends once;
     - a panel open sends to the panel without repeat;
     - `Stale` or an injection error stops the burst with one warning.
   - *Implement:* press, tick and let-go calls on the face, which reach `run.rs`'s tap for each repeat.
   - *Verify:* `cargo test -p kx-mod-keyboard`.
5. **The UI.**
   - *Files:* `bridges/keyboard.rs`, `Key.qml`, `KbWindow.qml`, `qmltest/` stand-in and a new `qmltest/tst_repeat.qml`.
   - *Test first:*
     - a press then a release on a key calls press and let go, and a short click still types once;
     - leaving the key calls leave;
     - the hold timer runs only while a key is held, and its ticks are driven by the test, not waited for;
     - the language key and dimmed keys keep their click;
     - `tst_budget`'s item counts do not change.
   - *Implement:* pressed, released and exited handlers on repeating keys. One `Timer` in `KbWindow.qml`, with its interval from the settings, asks the bridge on each tick.
   - *Verify:* `cargo test -p kx-ui --test texts`, then `qmltestrunner -input crates/kx-ui/qmltest` with `QML_DISABLE_DISK_CACHE=1`: 0 failures, no QWARN.
6. **Notes for later items, and the record.**
   - *Files:* `docs/items/P7.md` (hold-to-fill on @ and P meets repeat), `docs/items/P13.md` (settings window: repeat on/off, delay, speed, each with a plain explanation in EN, FR and AR), `docs/items/P3.md`.
   - *Verify:* `cargo xtask tidy`.

## Edge cases (each maps to a test)
1. A click shorter than the delay types once, on release (tasks 3, 5).
2. A hold of exactly the delay types once at the delay, not twice (task 3).
3. The pointer leaves the key while held: the repeat stops, and letting go outside types nothing (tasks 3, 5).
4. Shift latched: every repeat is upper case, and Shift lets go after the burst (task 4).
5. Ctrl, Alt or Win latched: one key, no repeat (task 4).
6. A dead key held: one dead key, no repeat (task 4).
7. A panel open: the key goes to the panel and does not repeat (task 4).
8. `repeat = false`: holding works as a click (tasks 2, 4).
9. The app in front changes layout mid-burst (`Stale`): the burst stops (task 4).
10. An injection error mid-burst: the burst stops with one warning and no key logged (task 4).
11. The keyboard hides or minimises while a key is held: the burst stops (task 5).
12. Arabic and French layouts repeat their own characters, including AltGr ones (task 4).
13. A delay or interval outside its range in a hand-edited file is refused (task 2).

## Security
- No new capability or port. Repeats go through the same injector as clicks, with our signature.
- Data class: typed keys. No key, scan code or character is logged (`Redacted<T>` where needed). Run `kx-security-check` §2–3 on the diff.
- Threat: a stuck repeat typing into an app. Every stop path is tested, and a burst can never outlive the pointer's press.

## Manual Windows check (owner)
1. In Notepad, hold "a": one "a", then a steady stream after about 1 s; let go and it stops.
2. Hold Backspace, then Delete: each deletes steadily and stops on let-go.
3. Click Shift once, hold "a": "AAAA", then Shift is off.
4. A slow single click (about half a second) types one letter.
5. Hold "a" and slide off the key: it stops.
6. Click Ctrl, then hold Z: one undo only.
7. Arabic layout: hold a letter: it repeats correctly.

## Done when
- The Definition of Done in `docs/roadmap.md`.
- Every edge case has its test, and the owner passes the manual check.
- `kx-review` has run on every changed file, and every finding is fixed.

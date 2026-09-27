# P1 stage 1: mouse-helper probes (G5, G17, G18, G6)

## Goal
Prove on this PC that the mouse helpers of spec §4.4, §5.1 and §5.2 work with one short left click at a time. This is the detail of tasks 12–13 of `2026-09-27-phase-1-toolkit-test-round.md`; the pass marks are in `docs/roadmap.md` (Phase 1).

## Scope
- **In:** a hold engine (Right-click and Grab), modifier+click, the three scroll routes, harness runs `g5`, `g17`, `g18`, `g6`, and a hand-try mode for the owner.
- **Out:** the ring, badge and sounds (P4), the scroll pad UI (P5), the health monitor (P4). A failed probe is a design finding, never a toolkit fail.

## Design notes
- **Spike only, light rules:** throwaway code in `spike/`; what we learn feeds P4 and P5.
- `spike/core/src/hold.rs`: the §5.1 state machine as pure logic.
  - Time is passed in (µs), so tests never sleep.
  - Every event returns "pass" or "swallow" plus the input to inject.
- `spike/harness/src/assist.rs`: hosts the engine on its own thread.
  - Low-level mouse and keyboard hooks and a message loop.
  - A timer for the hold, and injection after the hook returns.
  - The hook only queues; it times itself for the p99 check.
- **Whose input:** automated runs act only on input tagged "simulated user", so a hand on the mouse cannot disturb a run. The hand-try mode acts only on real, non-injected input.
- **Our own input:** everything the engine injects carries the existing `inject::TAG` and is skipped.
- `spike/harness/src/uia.rs`: UI Automation.
  - Find an item by name and its box.
  - Line boxes of a text.
  - Is an item selected.
  - Scroll with ScrollPattern and read the scroll percent.
- **Targets:**
  - `target-window` logs mouse buttons and right-clicks. It swallows its context menu, so nothing pops up.
  - `target-window` also runs as administrator through `admin::run_as`.
  - Notepad, Chrome and Word hold a known text; Explorer opens a fresh test folder.
- **New settings:**
  - `spike.toml [assist]`: `hold_ms = 1500`, `still_px = 7`.
  - `harness.toml [g5]`: case timings and `hook_p99_ms = 1.0`.
  - `harness.toml [g17]`, `[g18]`, `[g6]`: the move steps, test file names and scroll steps.

## Tasks
1. **Hold engine.** *Files:* `core/src/hold.rs`, `lib.rs`, `spike.toml`.
   - *Test first:* each §5.1 transition gives the right pass/swallow and injections, including every edge case below.
   - *Implement:* modes Off, Right-click and Grab; states Idle, Pending, AwaitUp and Carrying.
   - *Verify:* `cargo test -p spike-core` is green.
2. **Mouse log.** *Files:* `target-window/src/main.rs`, `core/src/targetlog.rs`, `harness/src/tlog.rs`.
   - *Test first:* mouse lines write and parse with button, kind and position.
   - *Implement:* log left and right down/up and context-menu events.
   - *Verify:* tests green.
3. **Assist host.** *Files:* `harness/src/assist.rs`.
   - *Test first:* hook data turns into engine events, and the source filter keeps only the wanted input.
   - *Implement:* start, stop, set mode, latch a modifier, last point outside the face, callback timings, events seen.
   - *Verify:* tests green.
4. **G5 run.** *Files:* `harness/src/g5.rs`, `main.rs`, `harness.toml`.
   - *Cases:* click, 30 px drag, still hold, 6 px wiggle then hold, and an 8 px move. Each runs on target-window and on target-window as administrator.
   - *Pass:* each case logs exactly the expected buttons, the hook p99 is at most 1 ms, and the hook saw every event.
   - *Verify:* `harness g5` gives `pass: true`.
5. **Hand try.** `harness assist <right|grab> --secs N`: the owner holds still on the desktop and gets the menu, then grabs a file. Their result is taken with a Pass/Fail pop-up.
6. **UIA helper.** *Files:* `harness/src/uia.rs`.
   - *Test first:* the pure parts, such as choosing points inside line boxes.
   - *Verify:* used by tasks 7–9.
7. **G17 Grab.**
   - Explorer: a file is dropped into a folder and moves on disk.
   - Esc during a carry leaves the file in place.
   - Notepad, Chrome and Word: a grab selection copies a non-empty part of the known text.
   - After every case, the left button is up.
8. **G18 Modifier+click.**
   - Shift+click selects from the caret in Notepad and Chrome.
   - Ctrl+click leaves two Explorer files selected.
   - The modifier is released after the click.
9. **G6 Scroll.**
   - Four directions × Notepad, Chrome, Explorer and Word × the three routes.
   - The pointer rests on the face, so the target is the last point outside it.
   - A route passes when the window picture or the UIA percent changes.
   - We record which route works where; "not scrollable that way" is recorded, not failed.
10. **Record.** Review, WORKLOG, commit after tasks 4, 7 and 9; numbers into `docs/NEXT-SESSION.md`.

## Edge cases (each one maps to a hold-engine test)
1. Press, then move 6 px: still a hold. Move 8 px: a normal drag from the first point.
2. Release before T is a normal click at the press point. Release at exactly T counts as a hold.
3. A second button pressed during Pending replays the held press, then lets the button through.
4. A mode switch during Pending replays the held press.
5. A press on our own window is never held.
6. Esc during a carry lets Esc through and releases the button. A drop ends Grab and brings Right-click back if it was on.
7. Our own injected input is always passed and never acted on.
8. A latched modifier is released after the next left click, even if the click is held back.

## Security
- The hooks see all input. They act only on the left button and Esc, log no keys and no positions outside the test, and are removed when the run ends.
- Test folders, files and texts are generated and deleted after each run.
- The admin target is our own `target-window`, started by `admin::run_as`; no settings change.

## Manual Windows check (owner)
The hand try (task 5): Right-click by a still hold on the desktop, then Grab a file into a folder. The owner answers Pass/Fail in a pop-up.

## Done when
`g5`, `g17`, `g18` and `g6` print their numbers and the hand try is answered. Every result is in `docs/NEXT-SESSION.md`, ready for ADR-0013.

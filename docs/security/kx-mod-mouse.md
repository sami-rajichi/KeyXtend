# Security note: kx-mod-mouse

- **Owner:** product owner
- **Last review:** 2026-10-14 (P4 review)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-mouse`: the hold engine, the actor thread that runs it on the pointer hook, and the `[mouse]` settings. It runs inside the uiAccess `keyxtend` process from P4 on, and its engine decides each event on the platform's hook thread (ADR-0027).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `PointerHook` | It gates the system-wide pointer hook the hold engine runs on (ADR-0027). | The module does not start, so Right-click, Grab and modifier + click are off. |
| `InjectInput` | The engine sends clicks and modifier keys. It also gates the bus events the module hears (`AssistToggle`, `LatchesChanged`) and sends (`LatchesUsed`, `LatchesAsk`). | The module does not start, as its subscriptions are refused. |

- Besides reading its settings, the module writes only `rclick`, when the user turns Right-click on or off; Grab's pause writes nothing.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| `[mouse]` settings: Right-click mode, the hold time, the still radius, the ring's delay, and the hook's health and echo times | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| Pointer points: each hook event's point, a held-back press, the last point, and the points in the notes and on the bus (`AssistNote`, `PointerMoved`) | Sensitive (a run of clicks on an on-screen PIN pad shows what was typed) | Memory only: an event lives for one engine call, notes wait in a fixed buffer until `settle`, and the latest move waits in one shared word (`Moves`) | No | Until the next event, note or move; the move word keeps the last point until the module stops |
| Latched modifiers' scan codes: the keyboard's latches (`LatchesChanged`) and a click's own modifiers | Sensitive together with the clicks around them | Memory only, at most `MAX_LATCHES` | No | Until the keyboard changes them, or the click's release |
| The hook's numbers (`HookStats`) | Public (counts and times) | The log, one line per stop (`STOPPED_LINE`) | No | As the log |

No point or scan code is written to disk or logged. Debug output hides them: `Note` and the port's `PointerEvent`, `PointerAct`, `AssistNote` and `PointerMoved` show each one as `Redacted`, `LatchesChanged` shows only a count, and `Msg` and `Cmd` have no `Debug`.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[mouse]` settings | `deny_unknown_fields`; each value within its range in `settings.rs` (`HOLD_MS` is spec §5.1's), and `ring_after_ms` below the shortest hold | Checked at load: a refused value goes back to its default with the `settings-reset` notice. The kernel caps the file (`MAX_FILE_BYTES`) | No; `settings/tests.rs` tries each bound |
| `AssistToggle` from the keyboard | A mode with nothing to check; sending it needs `InjectInput`. It flips a mode, lets a held-back press through first and cancels a carry | One command per toggle; a `settle` applies every queued command within the room `max_acts` reserves | Runtime test of a burst of toggles |
| `LatchesChanged` from the keyboard | Sending it needs `InjectInput`. The engine keeps the first `MAX_LATCHES` in order, and a click in progress keeps its own | `MAX_LATCHES` | Engine tests (`a_click_carries_at_most_max_latches_in_their_order`) |
| `AssistAsk` from any module | No payload; it only publishes the state again | One event per ask | Runtime test of either start order |
| Hook events (`PointerEvent`) from the platform | In the shipped app only the hand's input, plus our keyboard's `TAG` Esc, which only cancels a carry (`kx-platform-windows.md`). Point and time maths saturate, and a DPI of 0 counts as `BASE_DPI` | At most `MAX_ACTS` acts per call; notes within `NOTE_ROOM`, and moves coalesce | Property tests (`engine/props.rs`: random runs, lapses, missed releases) |

## Threats (STRIDE-lite)
- **Spoofing:** the engine hears only what the hook hands it: the hand's input, and our keyboard's `TAG` Esc, which can only cancel a carry (ADR-0027 decision 3, spec §5.1). Another program's input passes untouched, and the gate-only `--sim-input` build never ships (`keyxtend.md`).
  - Toggle and latch events need `InjectInput`, which the kernel grants only to first-party modules that declare it. A forged `AssistAsk` only sends the state again.
- **Tampering:** a hand-edited `[mouse]` value is checked and resets alone, so a still press can neither fire at once nor never. A saved `rclick = true` only turns Right-click on at start, which its lit key shows.
- **Logging / repudiation:** the module logs a refused hook start with its `HookError` (an OS code at most), a given-up engine, a refused `rclick` write by its kind only, and an actor panic. At each stop it logs `STOPPED_LINE` with the hook's numbers only (`the_stopped_line_logs_the_hook_numbers_and_no_point`).
  - Debug output hides every point and scan (`a_note_s_debug_hides_its_point`, and in `kx-module-api` `events_and_acts_hide_every_point_and_key` and `notes_hide_every_point`).
- **Information disclosure:** no network, and nothing on disk but `rclick`. `AssistNote` and `PointerMoved` carry click points to any subscriber, with no capability gate (see Accepted).
- **Denial of service:**
  - The engine's step never blocks, logs or allocates: its notes and a click's modifiers have room reserved at the start (`the_hot_path_never_grows_its_buffers`), and one call pushes at most `MAX_ACTS` (`the_worst_call_pushes_exactly_max_acts`).
  - The hook engine's `max_acts` adds `CMD_ACTS` for a `settle`'s commands, as only the first command that lets go of a held or carried press pushes input (`a_settle_that_fires_a_hold_and_takes_a_burst_of_toggles_stays_within_its_room`).
  - Moves coalesce to one in flight at the latest point (`two_moves_before_a_publish_give_one_pointer_moved_at_the_latest_point`), and a huge hold time or a far move never overflows.
  - The hook runs only while a mode, a latch, a held press or a carry needs it (property `the_hook_runs_while_needed_and_a_stop_leaves_nothing_down`).
  - A refused start for a mode key shows `notice.mouse-hook-failed` and the key stays off; one for latches warns once per streak.
- **Elevation:** with uiAccess the engine's input reaches admin windows, but it only reshapes the hand's own clicks there. It lets a held press through, right-clicks at the press point, picks up and drops for Grab, puts the latches around a click, and sends Esc to cancel a carry.
- **Stuck input:** every path lets go of the button and the modifiers.
  - `release` cancels a carry with Esc, then the Up, then the modifiers, and forgets a held-back press (`release_cancels_a_carry_with_esc_then_lets_go_of_it_and_its_modifiers`, properties `nothing_stays_down_after_release` and `nothing_sticks_after_a_stop_mid_run`).
  - The host calls it at a stop, a health reinstall and the panic hook's let-go, and the run goes on after the last two (`stopping_during_a_carry_cancels_it_with_esc_and_lets_go_of_the_button`, `a_let_go_mid_run_cancels_a_carry_tells_the_new_state_and_the_engine_works_on`).
  - Every dropped press gets its release, and the next click recovers from a missed one (properties `every_dropped_press_gets_its_release`, `the_next_click_recovers_from_a_missed_release`).
- **An engine the host gave up:** after its one release, the host drops a panicked engine, whose drop tells the actor (`Msg::Lost`). The actor drops the run, turns both modes off for this session only (the saved `rclick` stays), stays live and shows `notice.mouse-reset`.
  - A mode key starts a fresh run, and held latches get one at once, so a click keeps its modifier; a refused one takes the quiet latch path (`tests/mouse/reset.rs`).
  - A late word from an older run's engine changes nothing (`a_late_word_from_an_older_runs_engine_leaves_the_live_run_alone`).
- **An actor panic:** the thread publishes `AssistChanged::OFF` (`live: false`) at once, so the keyboard dims its keys, then `notice.mouse-stopped`. The run drops as the thread unwinds, which lets go (`an_actor_that_panics_mid_run_says_at_once_that_the_module_stopped`).

## Findings
### Must fix (blocks Definition of Done)
- None open.
- Done (P4 review, 3362cd4, dd9f271, 22e1eec): an engine the host gave up was never heard of again, so the keys stayed lit and clicks passed plain with no word. Its modes now reset with a notice, as above.
- Done (P4 review, 92b7a51): an actor that panicked mid-run said nothing until the module stopped. It now publishes `live: false` at once.
- Done (P4 review, ae3619d): a let-go during a carry dropped the item where the pointer was. It now cancels with Esc first.
### Should fix (tracked issue)
- None open.
- Done (P4 review, 175f4f9): the port's `PointerEvent` and `PointerAct` derived `Debug`, which would print points and scan codes. Both now hide them.
### Accepted (with reason)
- **Points on the bus with no gate:** `AssistNote` and `PointerMoved` carry click points, and neither names a capability, so any module could hear them. Every module is first-party and compiled in, and a new capability would change spec §9.4; P7 decides it with the face's gate (`kx-ui.md`, `docs/items/P7.md`).
- **Unbounded inboxes:** the actor's inbox and the engine's command queue have no bound. Only the bus fills them, one message per toggle, latch change, ask or coalesced move, from first-party modules.
- **A forged `TAG` Esc:** another program that tags its Esc as our keyboard's can cancel a carry (`kx-platform-windows.md`). It only lets the item go after an Esc, and the user picks it up again.
- **An engine that keeps failing:** each mode tap after a reset starts a fresh run, so a repeating fault shows `notice.mouse-reset` each time. Nothing stays held, as each give-up lets go first.

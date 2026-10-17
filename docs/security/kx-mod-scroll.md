# Security note: kx-mod-scroll

- **Owner:** product owner
- **Last review:** 2026-10-17 (P5 review: the threats, the read counts and the kept point's end)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-scroll`: the manifest, the `[scroll]` settings, the tap machine (`pad`), the glide (`glide`), the stop rules (`stops`) and the actor on its `kx-scroll` thread. It runs inside the uiAccess `keyxtend` process and moves targets only through the `ScrollTarget` port.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Scroll` | It gates the platform's `ScrollTarget`, which moves other apps' windows (ADR-0029). Only this module declares it. | The module does not start, so the pad dims and nothing scrolls. |

- The pad's events (`ScrollTap`, `ScrollStop`, `ScrollPeek`, `ScrollAsk`) are ungated, so any module may ask it to scroll, stop or outline. Its steps still go only where the user pointed, as the platform picks the target from the pointer.
- `PointerPressed`, which stops a scroll, is ungated too; it carries no point. What the module sends (`ScrollChanged`, `ScrollAimed`, `ScrollPage`) needs no capability either.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| `[scroll]` settings: the two speeds, the time limit, the two stop switches, and the tick, check, read and edge values | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| The target the platform aims at (`Aim`: its point, whether the pointer is on it, its box, ours or not, its id), and the box it publishes (`ScrollAimed`) | Sensitive (where the user points and what they look at) | Memory only: the actor keeps the latest aim of a scroll and the outline's target and box (`shown`, `actor.rs:81`), and the box rides one bus event | No | Until the next aim, the scroll's end or the module's stop, or until the outline hides, as `ScrollAsk` publishes the box again until then |
| The point kept before the keyboard, which the platform's entry watch holds for this module's aims (`kx-platform-windows.md`) | Sensitive (where the user pointed) | Memory only, in the platform (`window/entry.rs:38`), and only while this module is live | No | Until a later entry replaces it, the keyboard window's end, or this module's stop |
| The pad's state (live, way, speed), taps, Stop, peeks and presses (`ours` only) | Public | Memory only | No | Not kept |
| The actor's numbers (`STOPPED_LINE`: taps, ticks, turns, refusals and each stop's count) | Public (counts) | The log, one line when the actor quits | No | As the log |

The module never writes its settings. The port's `Aim` and `ScrollAimed` hide every point and box in debug output, the actor's `Msg` carries no point, and no log line holds a point or a box; the failed-open warning holds only the error's fixed text.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `ScrollTap`, `ScrollStop`, `ScrollPeek`, `ScrollAsk` and `PointerPressed` from the bus (ungated) | Typed values only, relayed into the actor's inbox; a press on our windows never stops a scroll, and nothing while still turns a target | One message per event; the inbox has no bound (see Accepted) | Actor property test over random taps, stops, peeks, presses and asks (`tests/scroll/props.rs`) |
| The target's answers (`aim`, `turn`, `reach`) | A refused aim or turn never stops the scroll, and its lines drop, so nothing piles up; an aim with no point at a check keeps the last target, so only a closed target stops it; `limit_s` bounds a scroll that stays refused, unless it is 0, when only the user stops it. An aim at our own windows only pages our list or keeps the last target | One aim per tap, peek or `check_ms`; up to two reach reads at a tap, or at the check that retries a refused first aim, otherwise one reach per `check_ms`; one turn per tick and per tap | Same property test, with aim and turn errors |
| `[scroll]` settings | `deny_unknown_fields`; each value within its range in `settings.rs`, `limit_s` either 0 or within its range, `fast` above `slow`, and `tick_ms` below `check_ms` | Checked at load: a refused value goes back to its default with the `settings-reset` notice. The kernel caps the file (`MAX_FILE_BYTES`) | No; `settings/tests.rs` tries each bound and every speed pair |

## Threats (STRIDE-lite)
- **Spoofing:** the pad's events name no sender and need no capability (`lib.rs:54-58`; `the_events_it_hears_and_sends_need_no_capability`), so any first-party module could tap, stop, peek or ask, and the bus is in-process, so no other program can (see Accepted).
  - Such a tap only starts what a user's tap would: the platform picks the target from the pointer, or from the point kept before the keyboard while the pointer is on it, never from the event (`kx-platform-windows/src/scroll/run.rs:111-136`). While our open list shows it wins the aim (`run.rs:117-120`), our other windows under the pointer give no target (`run.rs:121-125`), and a kept point that our window now covers aims at the app's window behind it, or at none (`run.rs:127-135`).
  - An aim at our own windows turns nothing there (`run.rs:215-217`): it pages our open list (`actor/taps.rs:23`, `:99-102`) or keeps the last target (`actor/ticks.rs:72-79`).
  - A forged `PointerPressed` can only stop a scroll, with `ours` false, or do nothing.
- **Tampering:** a hand-edited `[scroll]` value is checked and resets alone, so a speed can never be 0, fast never falls to slow, and the limit is either none or 10 s to 30 min. The module reads no other file, and the target's answers are typed values (`Aim`, `Reach`, `ScrollError`).
- **Logging / repudiation:** the module logs three lines, none with a point or a box.
  - A refused open warns once per streak with `ScrollError`'s fixed text (`actor.rs:199`; `a_target_that_does_not_open_tells_each_tap_and_logs_once`), and a panicked actor logs a fixed line (`lib.rs:87`; `an_actor_that_panics_says_at_once_the_module_is_off_and_is_logged_once`).
  - `STOPPED_LINE` holds counts only (`actor/stats.rs:44-60`; `the_stopped_line_holds_counts_only`).
  - Debug output: `Msg` holds only a way or a flag (`actor.rs:25-39`), and `Pad`, `Glide`, `Stops` and `Halt` only ways, speeds and times. `Aim` and `ScrollAimed` hide the point and box (`kx-module-api/src/ports/scroll.rs:91-102`, `:241-248`; `aims_and_outlines_hide_every_point_and_box`), and the actor and its `Scroll` have no `Debug`.
  - The platform's scroll part logs only its start warnings (`uia.rs:69`, `uia/work.rs:29`, `:32`, `win/pinger.rs:51`), and its `Read` and `Kept` hide their boxes and points (`a_read_shows_its_box_and_hides_it_in_debug`, `a_kept_entry_hides_every_point_in_debug`).
- **Information disclosure:** no network and nothing on disk. `ScrollAimed` carries the target's box to any subscriber with no gate (see Accepted).
  - The point kept before the keyboard exists only while this module is live. The keyboard window asks for the watch only then and unwatches as it stops (`kx-ui/qml/KbWindow.qml:91-96`, `:204-212`), and the hub refuses a watch otherwise (`kx-ui/src/hub/scroll.rs:180-184`).
  - `unwatch_entries` forgets the point (`kx-platform-windows/src/window/entry.rs:48-55`), and so does the keyboard window's end (`entry.rs:153-159`). An entry from our windows that do not page, such as the banner, keeps the last point and never takes one of theirs (`scroll/entry.rs` `Kept::past`, `Kept::after`); only the open list, marked as paging, can hold it (`window/pages.rs`).
  - The outline shows the box only while the module is live (`hub/scroll.rs:103`). Its four edge windows are click-through and take no focus (`Edge.qml:21`, `Qt.WindowTransparentForInput`, `Qt.WindowDoesNotAcceptFocus`) and lie under the keyboard (`Outline.qml:64`), so they never take a click or the focus.
- **Denial of service:**
  - Runaway scrolling: a scroll stops `limit_s` after the last pad tap (`stops.rs:142-147`), checked first at every `check_ms` (`actor/ticks.rs:26-43`), with `limit_s` 0 or 10 s to 30 min (`settings.rs:12`, `:87-91`); it also stops at the end, at a press outside our windows, when the keyboard hides and when the target goes.
  - The actor ticks only while scrolling (`actor.rs:168-170`), and each gap counts `check_ms` at most (`glide.rs:64`), so a late tick or a sleep never jumps; one step turns at most `fast` × `check_ms` of lines (property `it_glides_only_while_it_says_so_and_only_where_the_user_pointed`), and the platform cuts a turn at `MAX_TURN`.
  - A refused turn drops its lines (`actor/ticks.rs:51-68`; `a_refused_turn_drops_its_lines_and_the_next_owes_one_tick`), so nothing piles up behind a target that refuses.
  - A hung or busy target: the port never waits on the target (`ports/scroll.rs:137-142`). Each aim, turn and reach pings it first, and one that left a ping unanswered, owes one past `check_ms` or that Windows calls hung gets no turn and no read (`run.rs:141`, `:222-224`, `:245-248`; `scroll/ping.rs` `Pings::busy`).
  - A read waits `read_ms` at most (`scroll/uia.rs:76-89`), and a read behind a stuck one gives none at once, so a check holds the actor that long at most (see Accepted).
- **Elevation:**
  - With uiAccess, the platform's posted `WM_MOUSEWHEEL` and `WM_MOUSEHWHEEL` reach windows above our integrity level, which is why `ScrollTargetKey` needs `Scroll`, held only by this module (`manifest.rs:16`, `ports/scroll.rs:160-164`).
  - It sends only wheel steps, never a click, a key or text. A post carries numbers only (`win.rs:221-238`), a sent wheel is tagged `TAG` and goes only while the pointer is on the target and Windows routes wheels to the pointer (`run.rs:234`, `win.rs:254-270`), and UI Automation only takes `ScrollPattern` line steps.
  - The uiAccess process parses no data from the target (ADR-0004): UI Automation gives a box and two percents, which the platform refuses when they are bad (`kx-platform-windows.md`).
  - Nothing scrolls without a pad tap: a still pad turns nothing (`actor/ticks.rs:11-15`), and a peek or ask only aims and tells (`actor/notes.rs:53-69`).

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
- Done (P5 review, 5bb303c, 9eace75): a tap whose first aim the OS refused skipped the tap's checks once a later check found the target, so with `stop_at_end` off it glided into nothing until the limit. A retried first aim now runs the tap's checks and says why, and one that finds our list pages it and stops.
- Done (P5 review, 2d23148, 6fb97f7): the entry watch kept the user's last point outside the keyboard with the scroll module off. The keyboard now watches only while the module is live, the hub refuses otherwise, and a stop forgets the point.
- Done (P5 review, 36ecda6, 08e03c8): a read that could not finish cleared the kept box, and an entry from our own banner replaced the kept point with one on our window. Both now keep the last.
### Accepted (with reason)
- **Ungated pad events (a confused deputy):** any module can send `ScrollTap`, `ScrollStop`, `ScrollPeek` or `ScrollAsk`. Every module is first-party and compiled in, a tap scrolls only what the user last pointed at and stops at the limit, and a gate would change spec §9.4, which P7 decides with the face's gate (`kx-ui.md`).
- **The box on the bus with no gate:** `ScrollAimed` carries the target's box to any subscriber, as `PointerMoved` carries points (`kx-mod-mouse.md`), for the same reason.
- **An unbounded inbox:** the actor's inbox has no bound. Its senders are first-party modules, one message per event, and the actor handles each in one step.
- **A read holds a Stop:** a tap or check may wait on one UI Automation read, `read_ms` at most (300 ms by default, 2 s at most), so a Stop or press behind it acts that much later. The glide's cap keeps the next tick from jumping after it.
- **The user's own limits:** with `limit_s` 0 and both stop switches off, a scroll runs until Stop, a third tap, the keyboard hiding or a closed target. That is the user's choice, and Stop shows while it scrolls.
- **Press-stop needs mouse assist:** `PointerPressed` comes from the mouse module's hook, so with that module off a press does not stop a scroll; the end, the limit, Stop, the keyboard hiding and a closed target still do.
- **Admin windows:** the uiAccess build scrolls an elevated window the user points at, which is the feature (ADR-0029), with wheel steps only.

# Security note: kx-mod-scroll

- **Owner:** product owner
- **Last review:** 2026-10-17 (P5 docs: data and capability; the threats wait for `kx-security-check`)
- **Status:** Draft
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
| The target the platform aims at (`Aim`: its point, whether the pointer is on it, its box, ours or not, its id), and the box it publishes (`ScrollAimed`) | Sensitive (where the user points and what they look at) | Memory only: the actor keeps the latest aim of a scroll, and the box rides one bus event | No | Until the next aim, the scroll's end or the module's stop |
| The pad's state (live, way, speed), taps, Stop, peeks and presses (`ours` only) | Public | Memory only | No | Not kept |
| The actor's numbers (`STOPPED_LINE`: taps, ticks, turns, refusals and each stop's count) | Public (counts) | The log, one line when the actor quits | No | As the log |

The module never writes its settings. The port's `Aim` and `ScrollAimed` hide every point and box in debug output, the actor's `Msg` carries no point, and no log line holds a point or a box; the failed-open warning holds only the error's fixed text.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `ScrollTap`, `ScrollStop`, `ScrollPeek`, `ScrollAsk` and `PointerPressed` from the bus (ungated) | Typed values only, relayed into the actor's inbox; a press on our windows never stops a scroll, and nothing while still turns a target | One message per event | Actor property test over random taps, stops, peeks, presses and asks (`tests/scroll/props.rs`) |
| The target's answers (`aim`, `turn`, `reach`) | A refused aim or turn never stops the scroll, and its lines drop, so nothing piles up; `limit_s` bounds a scroll that stays refused. An aim at our own windows only pages our list or keeps the last target | One aim and reach per tap and per `check_ms`, one turn per tick | Same property test, with aim and turn errors |
| `[scroll]` settings | `deny_unknown_fields`; each value within its range in `settings.rs`, `limit_s` either 0 or within its range, `fast` above `slow`, and `tick_ms` below `check_ms` | Checked at load: a refused value goes back to its default with the `settings-reset` notice. The kernel caps the file (`MAX_FILE_BYTES`) | No; `settings/tests.rs` tries each bound and every speed pair |

## Threats (STRIDE-lite)
- **Spoofing:**
- **Tampering:** a hand-edited `[scroll]` value is checked and resets alone, so a speed can never be 0, fast never falls to slow, and the limit is either none or 10 s to 30 min.
- **Logging / repudiation:**
- **Information disclosure:**
- **Denial of service:**
- **Elevation:**

## Findings
### Must fix (blocks Definition of Done)
### Should fix (tracked issue)
### Accepted (with reason)

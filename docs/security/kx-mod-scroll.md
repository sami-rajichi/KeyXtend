# Security note: kx-mod-scroll

- **Owner:** product owner
- **Last review:** 2026-10-16 (P5 scaffold)
- **Status:** Draft
- **Scope:** `crates/kx-mod-scroll`: the scroll module's manifest, its `[scroll]` settings and its start. The tap machine, the glide, the stops and the runtime come later in P5, and the threats get filled in then with `kx-security-check`.

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

The module never writes its settings. Target points and boxes arrive with the runtime; the port's `Aim` and `ScrollAimed` already hide them in debug output.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
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

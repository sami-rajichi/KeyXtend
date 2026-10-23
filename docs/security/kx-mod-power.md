# Security note: kx-mod-power

- **Owner:** product owner
- **Last review:** 2026-10-23 (P6 final fixes: the `notice.power-stopped` notice after a panic)
- **Status:** Draft
- **Scope:** `crates/kx-mod-power`: the manifest, the actor on its `kx-power` thread, and the short `kx-power-open` thread each click opens the dialog on. It runs inside the uiAccess `keyxtend` process and reaches the OS only through the `PowerDialog` port (ADR-0031).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Power` | It gates the platform's `PowerDialog`, which opens the system's own power dialog and nothing else (`manifest.rs:15`). Only this module declares it. | The module does not start, so the Power key dims with its tip. |

- The events it hears (`PowerAsked`, `PowerAsk`) and sends (`PowerChanged`, `Notice`) need no capability (`the_events_it_hears_and_sends_need_no_capability`), as the keyboard publishes and hears them.
- It provides no service and owns no settings, so it reads and writes no file.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The module's state (`PowerChanged { live }`) | Public | Memory only, one bus event | No | Not kept |
| An open's failure (`PowerError`: `Unavailable` or `Failed`) and the `notice.power-failed` notice | Public | Memory only; the warning holds the error's fixed text | No | Not kept |
| The `notice.power-stopped` notice after a panic | Public | Memory only, one bus event | No | Not kept |

The module handles no typed text, no point, no key and no user data.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `PowerAsked` and `PowerAsk` from the bus (ungated, unit events) | None needed: they carry no data, and each is relayed into the actor's inbox (`lib.rs:42-43`) | One message per event; the inbox has no bound (see Accepted) | Actor property test over random clicks, state asks and refused opens (`tests/power/asks.rs`) |
| The port's answer to `open` | A typed `Result`; an error shows the notice once and is never retried, and a panic turns the module off (`actor.rs:75-79`, `:93-97`) | One answer per click, read only while the actor runs | No |

## Threats (STRIDE-lite)
- **Spoofing:** `PowerAsked` names no sender, so any first-party module could ask; the bus is in-process, so no other program can. Such an ask only opens the same dialog a click would, where the user still picks or cancels.
- **Tampering:** nothing to tamper with: no settings, no file, and the events carry no data.
- **Logging / repudiation:** the module logs three fixed lines and nothing else: a failed open with `PowerError`'s fixed text (`actor.rs:76`), an open's thread that did not start (`actor.rs:101`), and a panicked actor (`lib.rs:72`).
- **Information disclosure:** no network, nothing on disk, and nothing user-made to disclose.
- **Denial of service:**
  - A flood of `PowerAsked` only opens the same dialog again: each click starts one short thread that ends when the port returns (`lib.rs:90-95`).
  - A dialog call that does not return holds only its own thread: the actor goes on answering, and a stop or the app's quit never waits on it (`a_stop_never_waits_on_an_open_and_a_late_answer_tells_nothing`). An answer after the stop goes to an inbox nobody reads, so it publishes nothing (`actor.rs:98`).
  - A panic in an open or in the actor says at once that the module is off, so the key dims, and `notice.power-stopped` says to restart (`lib.rs:80-87`; `an_open_that_panics_says_at_once_the_module_is_off`).
- **Elevation:** with uiAccess, the port could reach the shell above our integrity level, which is why `PowerDialogKey` needs `Power`, held only by this module. The module calls only `PowerDialog::open`, which shows Windows' own Shut Down Windows dialog (`kx-platform-windows.md`); no code path shuts down, restarts or confirms, and no test opens the real dialog.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- The inbox and the open threads have no bound: asks come from the user's clicks or a first-party module, and each open thread ends when the port returns.

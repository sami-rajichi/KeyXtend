# Security note: kx-ui

- **Owner:** product owner
- **Last review:** 2026-09-30
- **Status:** Draft
- **Scope:** `crates/kx-ui`: the texts, the `[ui]` settings, the `ui` module and its hub, and the QML module `KeyXtend.Ui` with its cxx-qt bridges. It runs inside the uiAccess `keyxtend` process, with the bridges on the Qt GUI thread.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None declared | The `ui` module only draws; it sends keys through the keyboard's face, never through `KeyInjector`. | N/A |
| `KeyboardFace` service (required) | The board, the live state and every click on a key. | The `ui` module fails and offers Try again; the rest keeps running. |
| UI ports the app hands in (`WindowGuard`, `Screens`, `SystemLook`, `Backdrop`) | No-focus guard and window shape, screen areas and the pointer, the system look, and the backdrop. | The windows take focus, or the look stops following Windows. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Key ids QML sends on a click | Sensitive (together they spell what the user types) | Memory only, passed straight to the face | N/A | The call |
| Board and state JSON (key boxes, legends, lights) | Public | Memory only | N/A | Until the next redraw |
| Notices: setting keys, module ids and counts | Public | Memory only, until dismissed | N/A | Until dismissed |
| `[ui]` settings | Public | `settings.toml` through the kernel | No | Until changed |
| The focus token (a window handle) | Public | Memory only | N/A | Until the app quits |

Nothing typed is shown in a notice or logged: key ids, legends and replies never reach `tracing`. Log lines name only `[ui]` keys, counts and OS error codes.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[ui]` values from `settings.toml` | `deny_unknown_fields`; the check refuses an unknown theme or language, a contrast below `MIN_CONTRAST`, and a timing or row count outside its range; the kernel then resets that key alone | Ranges in `settings.rs` | No; tests cover each rule |
| Calls from QML (key ids, list places, box JSON, corners, reals) | Box JSON is parsed with `deny_unknown_fields`; a bad one warns and does nothing; a negative place is past every list; reals saturate to whole pixels, NaN is 0 | Bounded by the board | No |
| `Notice` events from the bus | Arguments are typed (text, count, list); lists are bounded by the settings store (`MAX_NAMED_KEYS`, `MAX_KEY_CHARS`) and counted past that | Duplicates show once | No |
| System look changes and single-instance commands (adapter threads) | Queued to the Qt thread with cxx-qt's thread queue, never blocking the adapter; commands wait until the `Ui` object is ready | One wake per change | No |

## Threats (STRIDE-lite)
- **Spoofing:** the QML files are compiled into the binary as Qt resources, so a file swapped on disk cannot change a window. The objects QML makes reach only the hub the app installed once.
- **Tampering:** a hand-edited `[ui]` value that fails the check never reaches the look or the timers; only that key goes back to its default.
- **Logging / repudiation:** errors name setting keys, OS codes and counts, never key ids or text (`Redacted<T>` is not needed because nothing sensitive is logged).
- **Information disclosure:** notice arguments are bounded setting names and counts, each isolated with Unicode bidi marks by Fluent. List items and text arguments also lose their control characters, line breaks and bidi controls first (`kx_module_api::is_hidden`), so a crafted setting key can neither split a notice nor turn it around.
- **Denial of service:** the hub never holds its lock during a face, port or waker call, so an adapter thread cannot deadlock the Qt thread. Guard tries stop at `guard_tries` (at most 100), and equal notices are merged.
- **Elevation:** the crate root has `#![deny(unsafe_code)]`; only `src/bridges/` allows it, for the cxx-qt glue, and each block has a `// SAFETY:` comment (tidy F3). The windows never take focus: the guard hands the focus back and no bridge activates a window.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- The request queue to the app (Try again, Quit) has no bound: only the user's clicks fill it, one request per click, and the app drains it at once.

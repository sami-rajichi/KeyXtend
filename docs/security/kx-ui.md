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
| UI ports the app hands in (`WindowGuard`, `Screens`, `SystemLook`, `Backdrop`, `Tray`) | No-focus guard and window shape, screen areas and the pointer, the system look, the backdrop, and the tray icon. | The windows take focus, the look stops following Windows, or a hidden keyboard has no way back but a second start. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Key ids QML sends on a click | Sensitive (together they spell what the user types) | Memory only, passed straight to the face | N/A | The call |
| Board and state JSON (key boxes, legends, lights) | Public | Memory only | N/A | Until the next redraw |
| Notices: setting keys, module ids and counts | Public | Memory only, until dismissed | N/A | Until dismissed |
| `[ui]` settings | Public | `settings.toml` through the kernel | No | Until changed |
| The focus token (a window handle) | Public | Memory only | N/A | Until the app quits |
| The tray picture (our own drawing of the theme badge) and where a tray click happened | Public | Memory only; the adapter keeps the picture while the icon shows | N/A | Until the icon hides |

Nothing typed is shown in a notice or logged: key ids, legends and replies never reach `tracing`. Log lines name only `[ui]` keys, counts and OS error codes.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[ui]` values from `settings.toml` | `deny_unknown_fields`; the check refuses an unknown theme or language, a contrast below `MIN_CONTRAST`, and a timing or row count outside its range; the kernel then resets that key alone | Ranges in `settings.rs` | No; tests cover each rule |
| Calls from QML (key ids, list places, box JSON, corners, reals) | Box JSON is parsed with `deny_unknown_fields`; a bad one warns and does nothing; a negative place is past every list; reals saturate to whole pixels, NaN is 0 | Bounded by the board | No |
| `Notice` events from the bus | Arguments are typed (text, count, list); lists are bounded by the settings store (`MAX_NAMED_KEYS`, `MAX_KEY_CHARS`) and counted past that | Duplicates show once | No |
| Tray clicks and "clicked elsewhere" (the adapter's thread) | Queued to the Qt thread as a wake, like the look; the hub holds itself weakly there, so the icon never keeps it alive. A click only opens or closes our menu at a spot kept inside the work area; a new front window closes it only after `[ui] tray_grace_ms` (at most 5 s) | One wake per event | No |
| The grabbed tray picture (QML `QImage`) | A null picture or size is refused; it is scaled to the tray's size and read pixel by pixel into a `TrayImage`, which checks its length | The taskbar's small icon size | Unit tests with pictures made in Rust; the QML load test checks a real grab reaches the tray |
| System look changes and single-instance commands (adapter threads) | Queued to the Qt thread with cxx-qt's thread queue, never blocking the adapter; a second copy's Show waits until the `Ui` object is ready, and its Quit goes straight to the app's request queue | One wake per change | No |

## Threats (STRIDE-lite)
- **Spoofing:** the QML files, fonts and icons are compiled into the binary as Qt resources, so a file swapped on disk cannot change a window. Every text is drawn as plain text, so a translation or a layout name cannot add rich text or links. The objects QML makes reach only the hub the app installed once.
- **Tampering:** a hand-edited `[ui]` value that fails the check never reaches the look or the timers; only that key goes back to its default.
- **Logging / repudiation:** errors name setting keys, OS codes and counts, never key ids or text (`Redacted<T>` is not needed because nothing sensitive is logged).
- **Information disclosure:** notice arguments are bounded setting names and counts, each isolated with Unicode bidi marks by Fluent. List items and text arguments also lose their control characters, line breaks and bidi controls first (`kx_module_api::is_hidden`), so a crafted setting key can neither split a notice nor turn it around.
- **Denial of service:**
  - The hub never holds its lock during a face, port or waker call, so an adapter thread cannot deadlock the Qt thread.
  - Quit, from the user or a second copy, ends the request queue, so the app's drain thread cannot keep a hidden uiAccess process alive; when that thread is gone, Quit ends the UI itself.
  - Guard tries stop at `guard_tries` (at most 100) per run, and equal notices are merged.
  - A second copy's tries to reach the running one stop at `send_tries` (at most 100), `send_wait_ms` (at most 10 s) apart.
  - `tray_hide` on `quitAsked` removes the tray icon for good; a picture drawn later cannot bring it back, so no dead icon stays in the taskbar.
- **Elevation:** the crate root has `#![deny(unsafe_code)]`; only `src/bridges/` allows it, for the cxx-qt glue, and each block has a `// SAFETY:` comment (tidy F3). The windows never take focus: the guard hands the focus back and no bridge activates a window.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- The request queue to the app (Try again, Quit) has no bound: only the user's clicks and a second copy's Quit fill it, one request each, and Quit ends it.

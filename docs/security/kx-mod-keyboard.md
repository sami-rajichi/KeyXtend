# Security note: kx-mod-keyboard

- **Owner:** product owner
- **Last review:** 2026-10-01
- **Status:** Draft
- **Scope:** `crates/kx-mod-keyboard`, which runs inside the uiAccess `keyxtend` process from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `InjectInput` | It gates the three services typing needs: `InputInjector` (send keys and text, name the app in front), `LayoutSource` (Caps Lock, whether a key types) and `Layouts` (the table shown). | The module does not start. |

- Besides its settings, the module writes only its own `size`, `x`, `y` and `placed`.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| `[keyboard]` settings: size, place, modifiers mode, language-key style, bubble corner, `key_apps` | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| The board: rows, widths, scan codes, roles | Public | Compiled into the app (`board.toml`) | No | The build |
| The text of each click, built from the layout table | Sensitive (typed text) | Memory only, as `Redacted<String>`, until the injector takes it | No | One click |
| The app in front's file name, for `key_apps` | Sensitive (shows what the user runs) | Memory only | No | One click |

Typed text never leaves the click that makes it: it is not stored, logged or sent anywhere but the injector. The debug output of a `Route` hides its keys and text, since a run of clicks would show what was typed.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[keyboard]` settings | `deny_unknown_fields`; choices by name; sizes finite, inside `size_min..=size_max`, step above 0, `relabel_ms` within `RELABEL_MS`; no blank app names | The kernel caps the settings file (`MAX_FILE_BYTES`) | No |
| `board.toml` | Compiled in, never read from disk; `deny_unknown_fields`, rows span the grid, whole columns, unique ids, roles fit; a broken board stops the module with an error, never a panic | Fixed at build time | Property tests |
| Screens and pointer points from the UI | Integer maths saturates; a place off every screen goes home; a zero or odd DPI or screen never panics | — | Property tests |
| Key ids from the UI | Looked up on the board; an unknown id is `UnknownKey`, never a panic | — | Property tests (runs of clicks) |
| Face actions from the UI: panel keys, rows per page, list and preset indexes | A panel key must have a `nav` on the board; zero rows count as one; an index past its list is `NoChoice`; while the list is open its keys are never typed | — | Property tests (list moves) |
| Layout tables, Caps Lock and the app name from the ports | Read as data; a missing key goes as its scan code, with every latch held; while the shown table lags the app in front, a scan-code click sends nothing, the face refreshes the layouts once, and the user clicks again | One click | Golden-table tests |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network.
- **Tampering:** a hand-edited value is checked, and a bad one resets alone (kernel repair). A hand-edited place off every screen brings the keyboard back to the bottom centre.
- **Logging / repudiation:** the module never logs. Typed text that reached a log through a field named for it is redacted (the app's log test), and `kx-review` check 4 refuses a `tracing` call on an input path that could carry it.
- **Information disclosure:** the settings are not sensitive; typed text is `Redacted` and never logged. The face's debug output shows no key, place or typing state.
- **Denial of service:** a size or time of zero or not-a-number is refused, so nothing divides by zero or spins, and a relabel time past its ceiling is refused, so the labels never stall. A stale click is never replayed, so a lagging table cannot loop.
- **Elevation:** input goes only through the `InputInjector` port, and only for a click on a board key; nothing types on its own.
- **Stuck modifiers:** latches are the module's state, never held keys; each chord presses and releases its modifiers in one batch. A send refused whole changes no state; one taken in part uses the latches, so a retry never repeats a modifier.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- None.

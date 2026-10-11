# Security note: kx-mod-keyboard

- **Owner:** product owner
- **Last review:** 2026-10-11 (key repeat)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-keyboard`, which runs inside the uiAccess `keyxtend` process from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `InjectInput` | It gates the three services typing needs: `InputInjector` (send keys and text, name the app in front), `LayoutSource` (Caps Lock, whether a key types) and `Layouts` (the table shown). | The module does not start. |

- Besides its settings, the module writes only its own `size`, `x`, `y` and `placed`.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| `[keyboard]` settings: size, place, modifiers mode, language-key style, key repeat, bubble corner, `key_apps` | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| The board: rows, widths, scan codes, roles | Public | Compiled into the app (`board.toml`) | No | The build |
| The text of each click, built from the layout table | Sensitive (typed text; may be part of a password) | Memory only, as `Redacted<String>`, until the injector takes it; not zeroised (`kx-platform-windows.md`) | No | One click |
| Typing state: the latched modifiers and the dead-key wait | Sensitive together with the clicks around it (the latches alone show on screen and to screen readers) | Memory only; kept across a module restart | No | Until a click lets go of it, or the app quits |
| The app in front's file name, for `key_apps` | Sensitive (shows what the user runs) | Memory only | No | One click |
| The held key's id and its next repeat time | Sensitive together with the clicks around it | Memory only (`Repeat`) | No | Until the pointer lets go or leaves, the keyboard hides, the list opens, a repeat fails, is stale or types nothing, the next press, or a restart |

Typed text never leaves the click that makes it: it is not stored, logged or sent anywhere but the injector. Debug output hides scan codes, text and legends (`Route`, `Plan`, `Legend`) and the typing state (`Typing`, `Latches`, `LegendMods`, `KeyState`, `StateView`, and `Repeat`, which shows only its phase), since a run of clicks would show what was typed (ae6f192, d3b925e).

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[keyboard]` settings | `deny_unknown_fields`; choices by name; sizes finite; `size_min` and `size_max` within `SIZE_FLOOR` (0.8, where key cells on the 31 px row stay at least 24 px apart, WCAG 2.2 SC 2.5.8 through its spacing exception) and `SIZE_CEILING` (4.0); `size` and the rising presets inside `size_min..=size_max`, the step above 0 and no wider than that range; `relabel_ms` from 50 to 10,000 ms (`RELABEL_MS`); `repeat_delay_ms` from 300 to 3,000 ms and `repeat_every_ms` from 33 to 1,000 ms (`REPEAT_DELAY_MS`, `REPEAT_EVERY_MS`); no blank app names | Checked at load: a refused value goes back to its default with the `settings-reset` notice. The kernel caps the settings file (`MAX_FILE_BYTES`) | No |
| `board.toml` | Compiled in, never read from disk; `deny_unknown_fields`, rows span the grid, whole columns, unique ids, roles fit; a broken board stops the module with an error, never a panic | Fixed at build time | Property tests |
| Screens, pointer points and the resize corner's overhang from the UI | Integer maths saturates; a place off every screen goes home; a zero or odd DPI or screen never panics; an overhang below 0 or not a finite number counts as none | — | Property tests |
| Key ids from the UI | Looked up on the board; an unknown id is `UnknownKey`, never a panic | — | Property tests (runs of clicks) |
| Face actions from the UI: panel keys, rows per page, list and preset indexes | A panel key must have a `nav` on the board; zero rows count as one; an index past its list is `NoChoice`; while the list is open its keys are never typed | — | Property tests (list moves) |
| Held-key actions from the UI: `press`, `tick`, `let_go`, `leave` (spec §4.6) | A pressed or let-go key must be on the board (`UnknownKey`, which changes nothing); a tick types only while a key is held and a repeat is due, and a late tick types once; a let-go of another key types nothing | One repeat per `repeat_every_ms` (33 ms or more) | Property tests (`Repeat` runs; face runs of presses, ticks, let-gos and leaves) |
| Layout tables, Caps Lock and the app name from the ports | Read as data; a missing key goes as its scan code, with every latch held; while the shown table lags the app in front, a scan-code click sends nothing, the face refreshes the layouts once, and the user clicks again | One click | Golden-table tests; a property run of the dead-key wait |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network. A key acts on a click or a UI Automation Invoke from any program UIPI lets through, and the face cannot tell who sent it; the P3 manual check must confirm that a normal program's click and Invoke type nothing in the signed uiAccess build (`docs/items/P3.md`).
- **Tampering:** a hand-edited value is checked, and a bad one resets alone (kernel repair). Sizes stop at `SIZE_FLOOR` and `SIZE_CEILING`, so a hand-edited size can neither shrink the keys past clicking nor open the keyboard off every screen, and a place off every screen brings the keyboard back to the bottom centre.
- **Logging / repudiation:** the module never logs; a held key's failed repeat is one `keys-not-sent` notice, which kx-ui's hub also logs as one warning, with no key. Typed text that reached a log through a field named for it is redacted (the app's log test), and `kx-review` check 4 refuses a `tracing` call on an input path that could carry it.
- **Information disclosure:** the settings are not sensitive; typed text is `Redacted` and never logged. The face's debug output shows no key, place or typing state.
- **Denial of service:**
  - A size or time of zero or not-a-number is refused, so nothing divides by zero or spins; a relabel time or size past its ceiling is refused, so the labels never stall and the keyboard never opens far off every screen.
  - A stale click is never replayed, so a lagging table cannot loop.
  - A click holds the face's lock while it sends, but no OS call on that path sends a message to another window, so a hung app cannot stall a click.
  - A held key repeats at most once a beat, and a late tick types once and never catches up, so a slow frame cannot flood the app.
- **Elevation:** input goes only through the `InputInjector` port, and only for a click on a board key or a board key held down; nothing types on its own. With Ctrl, Alt or Win latched (or `AltGr` acting as Right Alt), or on a dead key, a held key types once, so a shortcut such as Ctrl+W never fires many times. The face service itself has no capability gate (see Should fix).
- **Stuck repeat:** a burst ends on let-go, on leaving the key, when the keyboard hides or stops (the UI sends `leave`), when the list opens, when a repeat is stale, empty or fails, on the next click (a burst left open, as after a window rebuilt for a lost graphics device), and on a restart. A face property run shows that no tick types and no latch stays down after a let-go or leave, and the UI's hold timer runs only while a key is held on a shown keyboard.
- **Stuck modifiers:** latches are the module's state, never held keys; each chord presses and releases its modifiers in one batch. A send refused whole changes no state; one taken in part uses the latches, so a retry never repeats a modifier. During a burst the latches stay latched, not pressed, and let go when it ends.

## Findings
### Must fix (blocks Definition of Done)
- **Who can press a key (manual check):** only UIPI keeps a normal program from making the uiAccess keyboard type into an admin window, since a key acts on any click or Invoke Windows lets through. The P3 manual check must confirm it in the signed build (`docs/items/P3.md`); if a normal program can type, P3 is not done.
- Done (P3, c163124 and 5dcce0e): `size_max`, `size` and `size_step` had no ceiling, so a hand-edited `size_max = 1000` opened the keyboard off every screen on each start. `SIZE_CEILING` and `SIZE_FLOOR` now bound every size at load.
### Should fix (tracked issue)
- **The face has no capability gate (P7):** `KeyboardFaceKey` carries none, yet a `Tap` types, as do `Press`, `Tick` and `LetGo`, so any module that requires the face types without `InjectInput`. Today only `ui` requires it (`docs/items/P7.md`).
- Done (P3, cc31ec9): the dead-key wait had no property test; a run of clicks on French AZERTY now pins how the wait starts, ends and stays, and that each Unicode send equals the key's legend without its ◌.
### Accepted (with reason)
- **A front layout that is not installed:** the table shown stands, so a character key sent as a scan code types in the app's own layout and may not match its legend. Telling the user is later work; nothing is typed that the user did not click.
- **`AltGr` as Right Alt:** on a layout without `AltGr` characters, `AltGr` is Right Alt, so a character key runs an Alt shortcut (such as Alt+Y on a dialog) while its legend shows the letter. Ctrl, Alt and Win latches do the same and the key stays lit, as a physical right Alt does.
- **Check, then send:** the stale check and the send are separate OS reads, so a layout switch in the microseconds between them types in the new layout. Only the user's own hotkey or another program can switch then.
- **The dead-key wait is a guess:** a dead key used up elsewhere leaves the wait set. The next character then goes as a key press, which the stale check keeps equal to its legend.
- **No cap on a burst:** a held key repeats for as long as it is held, as the owner wants, and every way the press ends stops it.
- **A burst follows a focus change:** if another window comes to the front mid-burst, the repeats go to it, as a physical key does.

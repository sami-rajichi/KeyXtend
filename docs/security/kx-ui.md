# Security note: kx-ui

- **Owner:** product owner
- **Last review:** 2026-10-11
- **Status:** Reviewed
- **Scope:** `crates/kx-ui`: the texts, the `[ui]` settings, the `ui` module and its hub, and the QML module `KeyXtend.Ui` with its cxx-qt bridges. It runs inside the uiAccess `keyxtend` process, with the bridges on the Qt GUI thread.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None declared | The `ui` module only draws; it sends keys through the keyboard's face, never through `InputInjector`. | N/A |
| `KeyboardFace` service (required) | The board, the live state, every click on a key, and the language of the layout shown, which the keyboard's texts follow (ADR-0020). The face has no capability gate, so the `ui` module, which holds no capability, types through it (see Should fix). | The `ui` module fails and offers Try again; the rest keeps running. |
| UI ports the app hands in (`WindowGuard`, `Screens`, `SystemLook`, `Backdrop`, `Tray`, `Memory`) | No-focus guard, window shape and a tip's lift, screen areas and the pointer, the system look, the backdrop, the tray icon, and giving the heaps' free pages back after a look change (ADR-0023). | The windows take focus, the look stops following Windows, a hidden keyboard has no way back but a second start, or look changes keep freed memory committed. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Key ids QML sends on a click | Sensitive (together they spell what the user types) | Memory only, passed straight to the face | N/A | The call |
| Board and state JSON (key boxes, legends, lights) | Public | Memory only | N/A | Until the next redraw |
| Notices: setting keys, module ids and counts | Public | Memory only, until dismissed | N/A | Until dismissed |
| The language code of the layout in front and the system's display language | Public | Memory only | N/A | Until the next switch |
| `[ui]` settings | Public | `settings.toml` through the kernel | No | Until changed |
| The focus token (a window handle) | Public | Memory only | N/A | Until the app quits |
| The tray picture (our own drawing of the theme badge) and where a tray click happened | Public | Memory only; the adapter keeps the picture while the icon shows | N/A | Until the icon hides |

Nothing typed is shown in a notice or logged: key ids, legends and replies never reach `tracing`. Log lines name only `[ui]` keys, counts and OS error codes.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[ui]` values from `settings.toml` | `deny_unknown_fields`; the check refuses an unknown theme or language, a contrast below `MIN_CONTRAST`, and a timing or row count outside its range; the kernel then resets that key alone | Ranges in `settings.rs` | No; tests cover each rule |
| Calls from QML (key ids, list places, box JSON, corners, reals) | Box JSON is parsed with `deny_unknown_fields`; a bad one warns and does nothing; a negative place is past every list; reals saturate to whole pixels, NaN is 0 | Bounded by the board | No |
| The layout's language code (`KeyboardFace::lang`, from the OS) and the system's language (`SystemLook::ui_language`) | Only matched against `LANGS` by Fluent's language matching; an unknown, broken or missing tag gives English. Never a path, never logged, never shown | One text set built per switch | Unit tests cover odd tags |
| `Notice` events from the bus | Arguments are typed (text, count, list); lists are bounded by the settings store (`MAX_NAMED_KEYS`, `MAX_KEY_CHARS`) and counted past that | Duplicates show once | No |
| Tray clicks and "clicked elsewhere" (the adapter's thread) | Queued to the Qt thread as a wake, like the look; the hub holds itself weakly there, so the icon never keeps it alive. A click only opens or closes our menu at a spot kept inside the work area; a new front window closes it only after `[ui] tray_grace_ms`, within its range in `settings.rs` | One wake per event | No |
| Icon ids QML asks the icon provider for (`<rrggbb>/<name>.svg`, `cpp/tint.h`) | The tint must be six hex digits and the name plain ASCII (letters, digits, `-`, `_` and dots, no leading dot) of the icon type, so it only names a file in the compiled-in icons folder; anything else gives a clear image and one warning | One image of the asked size | No; the tint test tries `..`, slashes, a drive, `%` escapes, other types and bad tints |
| `KX_UI_CHURN` (debug builds only) | Set and not empty, it starts a fixed churn through the app's own calls: a key's tip, the language key, minimise to the bubble and back, then theme, Caps Lock and size changes. The language key is pressed only in the List design, where it opens a list and switches no layout, and Caps Lock changes only on the face (`FlipCaps`), so no key is sent. Release builds have neither the variable, the calls nor `Churn.qml` | One plan: a round of the pop-ups and every theme, 20 Caps Lock and 24 size steps | Unit test, `tst_churn` |
| The grabbed tray picture (QML `QImage`) | A null picture or size is refused; it is scaled to the tray's size and read pixel by pixel into a `TrayImage`, which checks its length | The taskbar's small icon size | Unit tests with pictures made in Rust; the QML load test checks a real grab reaches the tray |
| System look changes and single-instance commands (adapter threads) | Queued to the Qt thread with cxx-qt's thread queue, never blocking the adapter; a second copy's Show waits until the `Ui` object is ready, and its Quit goes straight to the app's request queue | One wake per change | No |

## Threats (STRIDE-lite)
- **Spoofing:** the QML files, the bundled fonts and the icons are compiled into the binary as Qt resources, so a file swapped on disk cannot change a window. The hub adds a theme's fonts to Qt's font list only from those resources, never from a user path (ADR-0021). Native's Segoe UI and Qt's glyph fallback come from Windows' font stack (`keyxtend.md`, Accepted). Every text is drawn as plain text, so a translation or a layout name cannot add rich text or links. The objects QML makes reach only the hub the app installed once.
- **Tampering:** a hand-edited `[ui]` value that fails the check never reaches the look or the timers; only that key goes back to its default.
- **Logging / repudiation:** errors name setting keys, OS codes and counts, never key ids or text (`Redacted<T>` is not needed because nothing sensitive is logged). A refused icon names its id, which holds only our own icon name and tint.
- **Information disclosure:** notice arguments are bounded setting names and counts, with no bidi marks around them (ADR-0022). List items, text arguments and the OS names on the space bar and in the language list lose their control characters, line breaks and bidi controls first (`kx_module_api::is_hidden`), so a crafted setting key or layout name can neither split a text nor turn it around.
- **Denial of service:**
  - The hub never holds its lock during a face, port or waker call, so an adapter thread cannot deadlock the Qt thread.
  - Quit, from the user or a second copy, ends the request queue, so the app's drain thread cannot keep a hidden uiAccess process alive; when that thread is gone, Quit ends the UI itself.
  - Guard tries stop at `guard_tries` per run, and equal notices are merged.
  - The heaps' free pages go back once after a look change, or once for back-to-back changes, after every shown window dropped its cached graphics, never at idle; a refused give-back warns once and stops nothing (ADR-0023).
  - A second copy's tries to reach the running one stop at `send_tries`, `send_wait_ms` apart; `settings.rs` bounds each timing and count.
  - `tray_hide` on `quitAsked` removes the tray icon for good; a picture drawn later cannot bring it back, so no dead icon stays in the taskbar.
  - A lost graphics device, or a window whose scene graph cannot start, ends Qt's loop once and brings the windows back at most `[ui] device_rebuilds` times per run (0 to 10), so neither can loop. Past that, a loss leaves each window its own device, and a window that still cannot draw ends the app with one error and a failure code, where Qt would end it with no word. Each try at a new device counts, `device_retry_ms` (at most 5 s) apart; a timer times the wait, so the GUI thread never sleeps. A frame drawn off the GUI thread ends the sharing for the run, so no later rebuild makes a shared device; nor does one when the drawing variables did not read back. Only debug builds can mark a device lost, a frame off the thread or a window failed.
- **Elevation:**
  - The crate root has `#![deny(unsafe_code)]`; only `src/bridges/` allows it, for the cxx-qt glue and the calls into `cpp/`, and each block has a `// SAFETY:` comment (tidy F3). The windows never take focus: the guard hands the focus back and no bridge activates a window.
  - `harden_qt` keeps another program of the user from loading its plugin, QML module or cached shader into the uiAccess process. It clears every name with a Qt prefix from the C runtime's narrow copy of the environment, which Qt's byte reads use, so a name the code page narrows to a Qt one goes too; `qunsetenv` also clears it from the wide copy that Qt's text reads use. It then turns off Qt Quick's shader cache on disk.
  - It reads that copy without the runtime's lock, which is safe on Windows because nothing writes it while `harden_qt` reads it; the only later writes are the adapter pins of a device rebuild, on the GUI thread. With a folder, it and `pin_imports` limit plugins and QML modules to the app's own folders; `cpp/harden.h` calls only Qt's and the C runtime's own functions.
  - After the clear, `pin_adapter` sets one Qt variable, `QT_D3D_ADAPTER_INDEX`, to the number the Windows adapter picked (the lowest-power GPU), on Qt's thread before Qt starts, so no outside value chooses Qt's graphics adapter. A device rebuild picks it again the same way, never from outside input.
  - `pin_drawing` then sets `QSG_RENDER_LOOP=basic` and `QSG_NO_DEPTH_BUFFER=1` (`harden::DRAW_VARS`), so the one shared `QRhi` is used on the GUI thread only; the app shares it only when both read back through Qt's byte read, as Qt Quick reads them, and makes it only on the thread of Qt's app object (ADR-0021).
  - `pin_timing` then sets three more fixed Qt names, `QT_D3D_MAX_FRAME_LATENCY`, `QT_D3D_NO_VBLANK_THREAD` and `QT_QPA_UPDATE_IDLE_TIME`, and the default swap interval, from the checked `[ui]` settings: whole numbers within their ranges and two flags, never outside text. The defaults are Qt's own values; an outside value of the same name is cleared first.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- **The face has no capability gate (P7):** `KeyboardFaceKey` carries none, while `FaceAction::Tap` types keys, so any module that requires the face could type. P7 decides between a dedicated capability and a read-only face before another module requires it (`docs/items/P7.md`).
- Done (P3, 4cfa0c8 and 553fa82): a plain `Text` reads markup, and notice arguments carry setting keys from the user-writable file, so rich text could have made the uiAccess process load an image. `tests/texts/markup.rs` now fails on any QML text element that does not read plain text.
### Accepted (with reason)
- The request queue to the app (Try again, Quit) has no bound: only the user's clicks and a second copy's Quit fill it, one request each, and Quit ends it.
- Any program of the session can forge the tray's callback, which the filter lets through for the taskbar, and open our tray menu at a spot it picks. The menu holds only Show keyboard and Quit, which `keyxtend.exe --show` and `--quit` give any program anyway.
- Qt's own messages go to the debugger channel, which a program of the session can read, never to the log file, also for a category the user-writable `qtlogging.ini` turns on. Our windows hold no typed text, and the click positions such logs could show are what that program's mouse hook sees anyway.
- `harden_qt` warns and carries on when a Qt-prefixed name stays set. Qt reads only short fixed names, which `qunsetenv` always clears, so a name left over is never one Qt reads.

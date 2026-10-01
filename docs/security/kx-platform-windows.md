# Security note: kx-platform-windows

- **Owner:** product owner
- **Last review:** 2026-09-30
- **Status:** Draft
- **Scope:** `crates/kx-platform-windows`: the QPC clock, the program and per-user folders, COM start-up, the uiAccess check, where libraries load from (`dll`), input injection (`inject`, `strokes`, `keys`), layout reading and the switch request (`layouts`, `langinfo`), the app in front (`front`), the window guard and window queries (`window`), screens (`screen`), the system look and its watcher (`sysui`, `watch`), DWM backdrops (`backdrop`), the one running copy with its command window (`instance`, `hidden`), and the tray icon with its window and front-window hook (`tray`). It runs inside the uiAccess `keyxtend` process.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None held | The adapter implements ports; the kernel grants capabilities to the modules that use them. The injector's and layout source's service keys need `InjectInput`. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Program folder and per-user folder paths | Sensitive (the per-user path holds the account name) | Memory only | N/A | For the run |
| Clock readings | Public | Memory only | N/A | Not kept |
| uiAccess flag of our own token | Public | Memory only | N/A | Not kept |
| Typed text and the keys it becomes | Secret (may be a password) | Memory only, as `Redacted` text and key events for one send | N/A | Dropped after the send |
| File name of the app in front | Sensitive (shows what the user runs) | Memory only; returned to the caller | N/A | Not kept |
| Installed layouts, their language facts and what each key types | Public (the same for every user of that layout; not typed text) | Memory only; returned to the caller | N/A | Not kept |
| Caps Lock state and the layout of the app in front | Public | Memory only | N/A | Not kept |
| Titles and boxes of our own windows; the window in front as a focus token | Public (our own text; a token is a window handle) | Memory only; returned to the caller | N/A | Not kept |
| Work areas, DPI and the pointer position | Public | Memory only; returned to the caller | N/A | Not kept |
| The system look (dark apps, accent, high-contrast colours, animations) and the UI language tag | Public | Memory only; the watcher keeps the last reading to spot a change | N/A | Until the watch ends |
| Commands from a later copy (`show`, `quit`) | Public (a code from a fixed table) | Memory only | N/A | Not kept |
| The tray icon's picture and tooltip; where a tray click happened; the class name of the window that comes to the front | Public (our own drawing and text; a point; a class name, never a title) | Memory only; the picture is kept to add the icon again after the taskbar restarts, and the class name is read into a stack buffer and dropped at once | N/A | The picture until the icon drops |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `SHGetKnownFolderPath` (Local AppData) | Error mapped to `PlatformError::LocalData`; the path is taken as UTF-16 as Windows gives it, never through `TEMP` or other environment variables | Windows bounds the string; it is copied once and freed once | No |
| `current_exe` (program path) | A path with no folder is refused (`NoExeDir`), so the portable `data` check never runs in the working folder | N/A | No |
| `QueryPerformanceFrequency` / `QueryPerformanceCounter` | Frequency failure refused at start-up; tick maths in 128 bits, clamped, negative and zero values handled (property tests) | N/A | Property tests |
| `CoInitializeEx` result | Success pairs one `CoUninitialize` on the same thread (`Com` is not `Send`); another thread mode is accepted and never undone | N/A | No |
| Our own process token (`TokenUIAccess`) | Reads 4 bytes (`size_of::<u32>()`) into a `u32`; any failure reads as "no uiAccess" | Fixed size | No |
| `SendInput` count | Clamped to the batch length; a batch too long for a `u32` is not sent | One batch per call | Property test of the release rule |
| `VkKeyScanExW` answer | Only no modifier, Shift, Ctrl+Alt (`AltGr`) or Shift+Ctrl+Alt is typed; anything else is `Unmappable` before a key is sent. `ToUnicodeEx` (the layout source's reader, no state change) must confirm the key types the character under the current Caps Lock, in that Shift state or the other; a dead key must give the character as its own, and Space follows it | Fixed size | Unit tests of every bit; fake-layout tests; a real AZERTY test |
| `MapVirtualKeyExW` scan code | Only its low byte and prefix byte are read | Fixed size | Unit tests |
| `GetKeyboardLayoutList` | Asked for the count first, then filled into a buffer of that count, so there is no cap; the copy is cut to what Windows wrote | Windows' own count | Fake-OS tests (40 layouts, a list that shrinks) |
| `ToUnicodeEx` / `MapVirtualKeyExW` | Called only for an installed layout or the layout of the app in front; flag `0x4` leaves every keyboard state as it is; the count is clamped to a 32-unit buffer and its sign read as a dead key; controls read as empty | 32 UTF-16 units per key | Unit and property tests of the answer reader |
| Registry `HKLM\...\Keyboard Layouts` (`Layout Text`, `Layout Id`) | Read-only key, `REG_SZ` values only, into a 128-unit buffer; the size Windows writes back is clamped and cut at the first null; `Layout Id` must be hex; the key is closed once | 128 UTF-16 units per value; the variant search ends at the first enumeration error | Unit and property tests of the text, id and key-name readers |
| `LCIDToLocaleName` / `GetLocaleInfoEx` | Fixed buffers; counts clamped; a number is read from exactly two units | 128 UTF-16 units | Unit and property tests |
| `GetGUIThreadInfo` (window with the keyboard) | A failed call or no focus falls back to the front window; thread 0 is refused | Fixed size | No |
| `QueryFullProcessImageNameW` (app in front) | Buffer of the Windows path limit; the length Windows writes back is checked; invalid UTF-16 reads as unknown; the process handle is closed once | 32,767 UTF-16 units | No |
| Messages to a guarded window (`WM_STYLECHANGING`, `WM_WINDOWPOSCHANGING`, `WM_MOUSEACTIVATE`, `WM_NCDESTROY`) | The style index is read from wParam's low 32 bits; the lParam struct is read and changed only for the two messages whose struct Windows fills; every other message goes unchanged to the window's own procedure | Fixed size | Unit tests of the style rules and the index reader; real hidden-window tests |
| The guard's window property (`KeyXtend.Guard`) | Holds the original procedure; with no property the guard falls back to `DefWindowProcW`. Unhooking restores it only while the guard's procedure is still in front, then removes the property | One pointer | Hidden-window tests (guard, renew, unhook, a later subclass in front) |
| `EnumWindows`, `GetWindowThreadProcessId`, `InternalGetWindowText` | Only visible windows of our process count, never the console host's; a title is read into a fixed buffer and the count clamped, with no message sent, so a busy window thread never blocks the reader | 512 UTF-16 units per title | Unit tests of the rules; hidden-window tests, one read from another thread |
| `EnumDisplayMonitors`, `GetMonitorInfoW`, `GetDpiForMonitor`, `GetCursorPos` | Fixed-size structs; a failed monitor is skipped, a failed DPI reads as 96, boxes are clamped to non-negative sizes | Fixed size | Unit tests of the box and DPI maths, including NaN and infinity |
| Registry `HKCU\...\Themes\Personalize` `AppsUseLightTheme` | `REG_DWORD` only; missing or refused reads as light | 4 bytes | Unit test |
| `UISettings` accent, `GetSysColor`, `SystemParametersInfoW` (high contrast, animations) | Fixed-size values; a failure reads as no accent, high contrast off or animations on; colours are always opaque. The first reading keeps the process COM apartment alive, so a cached factory never outlives COM, and no thread changes its COM mode | Fixed size | Unit tests of the colour and flag readers; tests that a thread can still pick its COM mode and that a reading after an STA ended still works, run in its own process |
| `GetUserDefaultUILanguage`, `LCIDToLocaleName` | Fixed buffer; an unknown language reads as an empty tag | 128 UTF-16 units | Real test that the tag is BCP-47 |
| Look broadcasts to the hidden watch window (`WM_SETTINGCHANGE`, `WM_SYSCOLORCHANGE`, `WM_THEMECHANGED`, `WM_DWMCOLORIZATIONCOLORCHANGED`) | Their parameters are never read; each one only triggers a fresh reading, and only a real change calls back. `WM_CLOSE` and a `WM_QUIT` posted to the window or its thread are ignored | One reading per broadcast | Unit tests of the change rule and of a panicking callback; a hidden-window test that `WM_CLOSE` is ignored; a unit test of the loop's end rule |
| `CreateMutexW` (`Local\` + app name) | "Already exists" or "access denied" reads as another copy; any other failure means Windows cannot check, and this copy runs | Fixed name | Unit and property tests of the name and the answer rule |
| `WM_APP` to the command window | Only wParam is read and must be a code in the command table; any other code and lParam are ignored, and so are `WM_CLOSE` and a `WM_QUIT` posted to the window or its thread. The listener runs outside any lock | One code per message | Property test of the table; real tests with an unknown code, `WM_CLOSE`, `WM_QUIT` to the window and to its thread, a panicking listener and a listener that waits for the UI thread |
| Tray callbacks to the icon's hidden window (`WM_APP + 1`) and the taskbar's `TaskbarCreated` | Only a click, the Enter or Space key, or the menu key (`NIN_SELECT`, `NIN_KEYSELECT`, `WM_CONTEXTMENU`) in the low word is acted on; the anchor is read as two signed 16-bit words; any other notification is ignored. `TaskbarCreated` only queues the icon's add again, never after it dropped. Both messages are let through UIPI (`ChangeWindowMessageFilterEx`) on this one window, since the taskbar runs below a uiAccess process; the worst a lower program can do with them is open our menu or add our icon again | One event per message | Unit tests of the reader (every opening code, other codes, negative anchors); real hidden-window tests over a fake taskbar |
| The front-window hook (`EVENT_SYSTEM_FOREGROUND`, out of context, our own process skipped) | Only the new front window's class name is read, into a 256-unit buffer; the taskbar's and its hidden-icon flyout's classes (a fixed table) do not count as a click elsewhere. The hook runs on the icon's thread and is removed with its window | One reading per change | Unit test of the class table; a real test of the class reader |
| The tray picture (RGBA from the UI) | `TrayImage` refuses a picture whose bytes do not fill its size; the colour bitmap is a DIB section of exactly that size, the pixels are copied only when their length matches it, the mask is built with rows padded to 16 bits, and both bitmaps are deleted once the icon holds its copies | None beyond `TrayImage`'s check; the UI sends the taskbar's small icon size | Unit tests of the byte order and mask size; a real test that the icon has the picture's size |
| `DwmSetWindowAttribute`, `DwmExtendFrameIntoClientArea` | Set only on one of our own windows, found by title; the material goes first, so a refusal (a `UiError`) leaves the frame and tint as they were. The no-backdrop answer of old Windows is success for `None` | Fixed size | Unit tests of the kind table, the answer rule and the order of changes |

## Threats (STRIDE-lite)
- **Spoofing:** a shell or parent process could set `LOCALAPPDATA` or `TEMP` to aim our data elsewhere. Mitigation: the folder comes from the known-folder API, not the environment.
- **Tampering:** our own injected input could loop back through our own hooks (P4) and be acted on twice. Mitigation: every event carries `keys::TAG` in `dwExtraInfo`, and our hooks skip it; the tag is pinned by a test.
- **Logging / repudiation:** `PlatformError` never holds a path, and `InjectError` holds only counts. The injector, the layout source and the window, look and tray parts log nothing, `UiError` holds at most an OS error code, and the debug output of key events and decoded keys never shows a key or unit. Layout names and characters are not typed text, but a caller must never log `Scan` values next to what the user typed.
- **Information disclosure:** no network; nothing leaves the process. Text is read only through `Redacted::expose` to build the send.
- **Denial of service:** start-up fails with a typed error, never a panic, when Windows refuses the clock or a folder. After a partial send it tries to release every key still held, and once more one event at a time for what Windows refuses, so no modifier should stay stuck down; a mask key (`keys::MASK_VK`, unassigned) goes first when Alt or Win is released, so neither opens the menu bar or Start.
- **Elevation (layouts):** `request` posts only `WM_INPUTLANGCHANGEREQUEST`, with a layout checked against the installed list, to the window that has the keyboard; that window decides whether to switch. With uiAccess the post may reach a window above our integrity level, which is why the port needs `InjectInput`. The adapter never loads, activates, adds or removes a layout; only the test window's `--layout` activates one (see Accepted).
- **Tampering (layouts):** the layout names come from `HKLM`, which only administrators can change; they are display text, bounded and never run or parsed further.
- **Tampering (guard):** the guard puts its procedure in front of visible windows of our own process only, on the thread that owns them (Windows refuses any other). It keeps no global state and unhooks itself on `WM_NCDESTROY`, so no procedure outlives its window.
- **Spoofing (command window):** the command window is message-only and reachable from this session only: Windows keeps window messages inside one desktop, and UIPI drops posts from a lower integrity level. A post does not name its sender, so the window acts only on a code from the fixed table (`show`, `quit`); a program that can post there runs as this user at our level and could end the process anyway.
- **Denial of service (UI parts):** a panic in a look or command callback is caught before it reaches Windows, and the next change or command is still heard. Another program's `WM_CLOSE`, or a `WM_QUIT` it posts to the window or to its thread, cannot end the command or watch window: the loop ends only on `stop`'s `WM_QUIT`, sent after `stop` sets its flag. Dropping a claim or a watch waits for a running callback, so a callback must never wait for the thread that drops it. A flood of look broadcasts costs one reading each and calls back only on a real change; a failed OS read gives a safe default, never a panic.
- **Spoofing (tray):** the tray window takes the taskbar's callback and `TaskbarCreated` from any program in the session, which UIPI would otherwise drop from below our level; a forged message can only open our own menu or add our icon again. The front-window hook reads class names only, never titles or content, and skips our own windows.
- **Denial of service (tray):**
  - A panic in the tray listener is caught before it reaches Windows.
  - Adds and changes run from the icon's own queue and no lock is held during a taskbar call, so a message that arrives inside the call cannot deadlock the thread, and neither `show` nor `set` waits on the taskbar: a hung Explorer stalls only the icon's thread, never the keyboard. A change queued behind an add, or behind a taskbar restart, follows the add.
  - The icon's thread deletes the icon as its window ends, after any add it had queued, so no dead icon stays behind while the app runs.
- **Elevation (libraries):** `dll::pin` limits libraries named without a folder to the app's folder and System32, so the working folder and `PATH` can no longer lend one to the uiAccess process. It does not cover the libraries the exe imports, which load before `main`; those must sit beside the exe.
- **Elevation (focus):** `give_back` moves the focus only from one of our windows to another app's live window. The adapter never asks for the focus for our own windows (ADR-0013); only the spike and the test tools keep such a request.
- **Elevation:** injection runs only on a module's call through the `InjectInput`-gated port. The Keys route types exactly the text or sends nothing, with dead keys checked on the golden layouts only (see Accepted): Ctrl-only and Alt-only answers are refused, a line feed is typed as Enter, not Ctrl+Enter, and every key is checked against the layout before the send. Every `unsafe` block is scoped, has a `// SAFETY:` comment, and touches only locals, our own token, handles we close, or Windows-allocated strings freed once.

## Findings
### Must fix (blocks Definition of Done)
None.

### Should fix (tracked issue)
None.

### Accepted (with reason)
`QueryPerformanceCounter` cannot report failure through `Clock::now`; Windows XP and later never fail it, so the clock reads the result as given.

`SHGetKnownFolderPath` in the `windows` crate drops the out-string on failure without `CoTaskMemFree`; Windows returns no string then, so nothing leaks in practice.

Another program can put our tag on its own input so that our hooks skip it. The tag only keeps us from acting on our own input; it is not a trust boundary, and forging it only hides that input from KeyXtend.

`ToUnicodeEx`'s no-state-change flag needs Windows 10 1607 or later, and builds before it are long out of support; on them, reading a dead key could leave it waiting in our own thread only.

`caps_on` reads Caps Lock with `GetKeyState`, the spike's method; it is this thread's view of the toggle, so the manual check of the non-activating keyboard must confirm it follows the key.

A dead key is checked by the character `ToUnicodeEx` gives for it, since its no-state-change flag cannot read the dead key and Space as a pair. That holds for the EN, FR and AR golden layouts; on others a dead key and Space may type another character (Swiss French ¨ gives ", Czech ˙ gives ·), so the Keys route may type that instead.

Windows does not report input that UIPI blocks (a window above our integrity level), so `SendInput` may count it as sent. The uiAccess build is allowed to reach those windows, which is why the app runs with uiAccess.

`kx-target-window --layout` (a test tool, never shipped) activates a layout, which on Windows 8+ switches the whole session's input language. It puts the previous layout back on close and when the switch does not take; a killed window cannot, so gates close it with `WM_CLOSE` first.

Another program in this session can create `Local\KeyXtend.Running` first, so KeyXtend reads a copy as running and then finds no command window. A named object cannot be kept from a same-user program; the app (Task 19) decides what to do when no command window answers.

DWM may draw Mica and Acrylic as a solid colour on an inactive window, and the keyboard is never active. The manual look check (G13, Task 16) decides; the spike's undocumented accent-policy blur ended in Task 9.

A monitor whose DPI Windows cannot give reads as 96, so a window there may be sized as if unscaled until the next reading.

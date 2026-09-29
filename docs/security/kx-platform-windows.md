# Security note: kx-platform-windows

- **Owner:** product owner
- **Last review:** 2026-09-29
- **Status:** Draft
- **Scope:** `crates/kx-platform-windows`: the QPC clock, the program and per-user folders, COM start-up, the uiAccess check, input injection (`inject`, `strokes`, `keys`), layout reading and the switch request (`layouts`, `langinfo`), and the app in front (`front`). It runs inside the uiAccess `keyxtend` process. Later P3 tasks add windows and system parts, and this note grows with them.

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
| `ToUnicodeEx` / `MapVirtualKeyExW` | Called only for an installed layout; flag `0x4` leaves every keyboard state as it is; the count is clamped to a 32-unit buffer and its sign read as a dead key; controls read as empty | 32 UTF-16 units per key | Unit tests of the answer reader |
| Registry `HKLM\...\Keyboard Layouts` (`Layout Text`, `Layout Id`) | Read-only key, `REG_SZ` values only, into a 128-unit buffer; the size Windows writes back is clamped and cut at the first null; `Layout Id` must be hex; the key is closed once | 128 UTF-16 units per value; the variant search ends at the first enumeration error | Unit tests of the text and id readers |
| `LCIDToLocaleName` / `GetLocaleInfoEx` | Fixed buffers; counts clamped; a number is read from exactly two units | 128 UTF-16 units | Unit tests |
| `GetGUIThreadInfo` (window with the keyboard) | A failed call or no focus falls back to the front window; thread 0 is refused | Fixed size | No |
| `QueryFullProcessImageNameW` (app in front) | Buffer of the Windows path limit; the length Windows writes back is checked; invalid UTF-16 reads as unknown; the process handle is closed once | 32,767 UTF-16 units | No |

## Threats (STRIDE-lite)
- **Spoofing:** a shell or parent process could set `LOCALAPPDATA` or `TEMP` to aim our data elsewhere. Mitigation: the folder comes from the known-folder API, not the environment.
- **Tampering:** our own injected input could loop back through our own hooks (P4) and be acted on twice. Mitigation: every event carries `keys::TAG` in `dwExtraInfo`, and our hooks skip it; the tag is pinned by a test.
- **Logging / repudiation:** `PlatformError` never holds a path, and `InjectError` holds only counts. The injector and the layout source log nothing, and the debug output of key events and decoded keys never shows a key or unit. Layout names and characters are not typed text, but a caller must never log `Scan` values next to what the user typed.
- **Information disclosure:** no network; nothing leaves the process. Text is read only through `Redacted::expose` to build the send.
- **Denial of service:** start-up fails with a typed error, never a panic, when Windows refuses the clock or a folder. After a partial send it tries to release every key still held, and once more one event at a time for what Windows refuses, so no modifier should stay stuck down; a mask key (`keys::MASK_VK`, unassigned) goes first when Alt or Win is released, so neither opens the menu bar or Start.
- **Elevation (layouts):** `request` posts only `WM_INPUTLANGCHANGEREQUEST`, with a layout checked against the installed list, to the window that has the keyboard; that window decides whether to switch. With uiAccess the post may reach a window above our integrity level, which is why the port needs `InjectInput`. The adapter never loads, activates, adds or removes a layout.
- **Tampering (layouts):** the layout names come from `HKLM`, which only administrators can change; they are display text, bounded and never run or parsed further.
- **Elevation:** injection runs only on a module's call through the `InjectInput`-gated port. The Keys route types exactly the text or sends nothing: Ctrl-only and Alt-only answers are refused, a line feed is typed as Enter, not Ctrl+Enter, and every key is checked against the layout before the send. Every `unsafe` block is scoped, has a `// SAFETY:` comment, and touches only locals, our own token, handles we close, or Windows-allocated strings freed once.

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

A dead key is checked by the character `ToUnicodeEx` gives for it, since its no-state-change flag cannot read the dead key and Space as a pair. Windows layouts type that character for a dead key and Space, so the Keys route relies on it.

Windows does not report input that UIPI blocks (a window above our integrity level), so `SendInput` may count it as sent. The uiAccess build is allowed to reach those windows, which is why the app runs with uiAccess.

# Security note: kx-platform-windows

- **Owner:** product owner
- **Last review:** 2026-09-29
- **Status:** Draft
- **Scope:** `crates/kx-platform-windows`: the QPC clock, the program and per-user folders, COM start-up, the uiAccess check, input injection (`inject`, `strokes`, `keys`) and the app in front (`front`). It runs inside the uiAccess `keyxtend` process. Later P3 tasks add layouts, windows and system parts, and this note grows with them.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None held | The adapter implements ports; the kernel grants capabilities to the modules that use them. The injector's service key needs `InjectInput`. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Program folder and per-user folder paths | Sensitive (the per-user path holds the account name) | Memory only | N/A | For the run |
| Clock readings | Public | Memory only | N/A | Not kept |
| uiAccess flag of our own token | Public | Memory only | N/A | Not kept |
| Typed text and the keys it becomes | Secret (may be a password) | Memory only, as `Redacted` text and key events for one send | N/A | Dropped after the send |
| File name of the app in front | Sensitive (shows what the user runs) | Memory only; returned to the caller | N/A | Not kept |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `SHGetKnownFolderPath` (Local AppData) | Error mapped to `PlatformError::LocalData`; the path is taken as UTF-16 as Windows gives it, never through `TEMP` or other environment variables | Windows bounds the string; it is copied once and freed once | No |
| `current_exe` (program path) | A path with no folder is refused (`NoExeDir`), so the portable `data` check never runs in the working folder | N/A | No |
| `QueryPerformanceFrequency` / `QueryPerformanceCounter` | Frequency failure refused at start-up; tick maths in 128 bits, clamped, negative and zero values handled (property tests) | N/A | Property tests |
| `CoInitializeEx` result | Success pairs one `CoUninitialize` on the same thread (`Com` is not `Send`); another thread mode is accepted and never undone | N/A | No |
| Our own process token (`TokenUIAccess`) | Reads 4 bytes (`size_of::<u32>()`) into a `u32`; any failure reads as "no uiAccess" | Fixed size | No |
| `SendInput` count | Clamped to the batch length; a batch too long for a `u32` is not sent | One batch per call | Property test of the release rule |
| `VkKeyScanExW` answer | Only no modifier, Shift, Ctrl+Alt (`AltGr`) or Shift+Ctrl+Alt is typed; anything else is `Unmappable` before a key is sent | Fixed size | Unit tests of every bit |
| `MapVirtualKeyExW` scan code | Only its low byte and prefix byte are read | Fixed size | Unit tests |
| `QueryFullProcessImageNameW` (app in front) | Buffer of the Windows path limit; the length Windows writes back is checked; invalid UTF-16 reads as unknown; the process handle is closed once | 32,767 UTF-16 units | No |

## Threats (STRIDE-lite)
- **Spoofing:** a shell or parent process could set `LOCALAPPDATA` or `TEMP` to aim our data elsewhere. Mitigation: the folder comes from the known-folder API, not the environment.
- **Tampering:** our own injected input could loop back through our own hooks (P4) and be acted on twice. Mitigation: every event carries `keys::TAG` in `dwExtraInfo`, and our hooks skip it; the tag is pinned by a test.
- **Logging / repudiation:** `PlatformError` never holds a path, and `InjectError` holds only counts. The injector logs nothing, and the debug output of key events and decoded keys never shows a key or unit.
- **Information disclosure:** no network; nothing leaves the process. Text is read only through `Redacted::expose` to build the send.
- **Denial of service:** start-up fails with a typed error, never a panic, when Windows refuses the clock or a folder. A partial send releases every key still held, so no Shift, Ctrl or Alt stays stuck down for the user.
- **Elevation:** injection runs only on a module's call through the `InjectInput`-gated port. The Keys route never turns text into a shortcut: Ctrl-only and Alt-only answers are refused, and a line feed is typed as Enter, not Ctrl+Enter. Every `unsafe` block is scoped, has a `// SAFETY:` comment, and touches only locals, our own token, handles we close, or Windows-allocated strings freed once.

## Findings
### Must fix (blocks Definition of Done)
None.

### Should fix (tracked issue)
None.

### Accepted (with reason)
`QueryPerformanceCounter` cannot report failure through `Clock::now`; Windows XP and later never fail it, so the clock reads the result as given.

`SHGetKnownFolderPath` in the `windows` crate drops the out-string on failure without `CoTaskMemFree`; Windows returns no string then, so nothing leaks in practice.

Another program can put our tag on its own input so that our hooks skip it. The tag only keeps us from acting on our own input; it is not a trust boundary, and forging it only hides that input from KeyXtend.

Windows does not report input that UIPI blocks (a window above our integrity level), so `SendInput` may count it as sent. The uiAccess build is allowed to reach those windows, which is why the app runs with uiAccess.

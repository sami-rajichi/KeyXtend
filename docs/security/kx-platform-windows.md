# Security note: kx-platform-windows

- **Owner:** product owner
- **Last review:** 2026-09-29
- **Status:** Draft
- **Scope:** `crates/kx-platform-windows`: the QPC clock, the program and per-user folders, COM start-up and the uiAccess check. It runs inside the uiAccess `keyxtend` process. Later P3 tasks add input injection, layouts, windows and system parts, and this note grows with them.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | The adapter implements ports; the kernel grants capabilities to the modules that use them. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Program folder and per-user folder paths | Sensitive (the per-user path holds the account name) | Memory only | N/A | For the run |
| Clock readings | Public | Memory only | N/A | Not kept |
| uiAccess flag of our own token | Public | Memory only | N/A | Not kept |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `SHGetKnownFolderPath` (Local AppData) | Error mapped to `PlatformError::LocalData`; the path is taken as UTF-16 as Windows gives it, never through `TEMP` or other environment variables | Windows bounds the string; it is copied once and freed once | No |
| `current_exe` (program path) | A path with no folder is refused (`NoExeDir`), so the portable `data` check never runs in the working folder | N/A | No |
| `QueryPerformanceFrequency` / `QueryPerformanceCounter` | Frequency failure refused at start-up; tick maths in 128 bits, clamped, negative and zero values handled (property tests) | N/A | Property tests |
| `CoInitializeEx` result | Success pairs one `CoUninitialize` on the same thread (`Com` is not `Send`); another thread mode is accepted and never undone | N/A | No |
| Our own process token (`TokenUIAccess`) | Reads 4 bytes (`size_of::<u32>()`) into a `u32`; any failure reads as "no uiAccess" | Fixed size | No |

## Threats (STRIDE-lite)
- **Spoofing:** a shell or parent process could set `LOCALAPPDATA` or `TEMP` to aim our data elsewhere. Mitigation: the folder comes from the known-folder API, not the environment.
- **Tampering:** none here; the adapter writes no files. The data folder rule (portable `data` beside the program) lives in `kx-platform` (ADR-0015).
- **Logging / repudiation:** `PlatformError` never holds a path, so its text cannot leak the account name into a log.
- **Information disclosure:** no network; nothing leaves the process.
- **Denial of service:** start-up fails with a typed error, never a panic, when Windows refuses the clock or a folder.
- **Elevation:** every `unsafe` block is scoped, has a `// SAFETY:` comment, and touches only locals, our own token or Windows-allocated strings freed once.

## Findings
### Must fix (blocks Definition of Done)
None.

### Should fix (tracked issue)
None.

### Accepted (with reason)
`QueryPerformanceCounter` cannot report failure through `Clock::now`; Windows XP and later never fail it, so the clock reads the result as given.

`SHGetKnownFolderPath` in the `windows` crate drops the out-string on failure without `CoTaskMemFree`; Windows returns no string then, so nothing leaks in practice.

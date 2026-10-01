# Security note: keyxtend (the app)

- **Owner:** product owner
- **Last review:** 2026-10-01
- **Status:** Draft
- **Scope:** `apps/keyxtend`: the command line, the one running copy as the app uses it, the start and stop order, the log subscriber, the panic hook, the manifest and Qt's start. The release build is the uiAccess process; the adapters, the kernel and the modules have their own notes.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| Builds the kernel's policy: each shipped module gets exactly what its manifest declares | Only the app knows which modules it ships and vouches for them | A module that needs a grant fails at load and offers Try again |
| uiAccess (release manifest only, `asInvoker`) | Stay above the Start menu and type into windows above our integrity level | UIPI drops keys sent to elevated windows, and the keyboard hides under the Start menu |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The command line | Public | Memory only | N/A | Not kept |
| The log (`logs/keyxtend.log` and the previous run's) | Public (sensitive fields redacted by `kx_kernel::log`) | Data folder (ADR-0015) | No | This run and the previous one, each capped at `log_max_mb` |
| A panic's place (file, line, column) | Public | The log | No | As the log |
| Qt's shader pipeline cache (`qtpipelinecache-*`) | Public | `%LOCALAPPDATA%\KeyXtend\cache`, outside the data folder | No | Never removed (see Should fix) |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Command line | At most one argument, `--show` or `--quit`; anything else, a non-Unicode argument or any Qt option (`-platform`, `-qmljsdebugger`), stops the app before Qt starts, so Qt only ever sees a checked argument | One argument | Property test |
| A later copy's Show or Quit (the command window, see `kx-platform-windows.md`) | Only the adapter's fixed codes reach the app; until the hub exists they wait in order, then go to `Hub::command` | Tries stop at `[ui] send_tries` (at most 100), `send_wait_ms` (at most 10 s) apart | Unit tests with a fake instance |
| `[kernel] log_level`, `log_max_mb` | Checked by the kernel's settings; a bad level opens no log | As the kernel's note | Property tests (kernel) |
| Qt's environment (`QT_PLUGIN_PATH`, `QT_QPA_PLATFORM_PLUGIN_PATH`, `QML2_IMPORT_PATH`, and more) | None yet: Qt reads them as usual | None | No |

## Threats (STRIDE-lite)
- **Spoofing:** a later copy speaks only through the adapter's command window, which acts on its fixed codes. A same-user program that holds `Local\KeyXtend.Running` without a command window makes the next copy stop with "another copy runs but did not answer", after its tries; it never starts a second keyboard.
- **Tampering:** the QML, fonts and icons are compiled in (see `kx-ui.md`); the command line is refused unless it is exactly one known flag.
- **Logging / repudiation:**
  - The log opens after the kernel loads its settings and before any module starts; if the file cannot open, the app logs to standard error and says so once.
  - The panic hook replaces Rust's default one and logs only the file, line and column, never the payload, which could hold typed text.
  - The first line gives the version and whether uiAccess is on; the last says the app ended.
- **Information disclosure:** nothing is sent anywhere; `cargo tree -p keyxtend` holds no network, TLS or media crate (ADR-0004).
- **Denial of service:**
  - Quit ends the request queue; the kernel's thread stops the modules, then ends the UI even if stopping panics (a drop guard), and the tray icon goes on every exit path.
  - `--quit` with no copy running starts nothing and exits at once.
  - A same-user program that squats the running mutex can keep KeyXtend from starting (Accepted).
- **Elevation:** the manifest asks for uiAccess only in release, with `asInvoker` and PerMonitorV2; the binary exports nothing (`/NOIMPLIB /NOEXP`). The app's code forbids `unsafe`. Qt's environment and its user-writable pipeline cache are the open points below.

## Findings
### Must fix (blocks Definition of Done)
- None for P3's debug build; the two items below must be settled before the signed uiAccess build is handed to users.
### Should fix (tracked issue)
- Qt's environment: a same-user program could set `QT_PLUGIN_PATH` (or a similar variable) and load its own plugin into the uiAccess process. The app should load plugins and imports only from its own install folder (Task 20 or 25; needs a small kx-ui bridge, since the app forbids `unsafe` and edition 2024 makes `env::set_var` unsafe).
- Qt's shader pipeline cache sits in a user-writable folder that the uiAccess process reads, and it is left behind by the portable edition. Turn it off (`Qt::AA_DisableShaderDiskCache`) through the same bridge.
### Accepted (with reason)
- A same-user program can squat `Local\KeyXtend.Running`: a named object cannot be kept from the same user, who could end the process anyway.
- Commands that arrive before the hub exists wait in an unbounded list: only same-session programs can post them, one code each, during the few milliseconds of load.
- Lines logged while the kernel loads are lost, since the log opens from the loaded settings; their notices still reach the banner.
- In the release build standard error goes nowhere, so a failed log file or a refused argument is silent there; the next run's log shows the start.

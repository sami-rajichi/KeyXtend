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
| A panic's place (file, line, column) | Public | The log; standard error until the log is set up | No | As the log |
| The settings file, read by a later copy | Public | Data folder | No | Read only, never changed by that copy |

Qt Quick keeps no shader pipeline cache on disk (`AA_DisableShaderDiskCache`), so the app writes nothing outside the data folder.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Command line | At most one argument, `--show` or `--quit`; anything else, a non-Unicode argument or any Qt option (`-platform`, `-qmljsdebugger`), stops the app before Qt starts, so Qt only ever sees a checked argument | One argument | Property test |
| A later copy's Show or Quit (the command window, see `kx-platform-windows.md`) | Only the adapter's fixed codes reach the app; until the hub exists they wait in order, then go to `Hub::command` | Tries stop at `[ui] send_tries` (at most 100), `send_wait_ms` (at most 10 s) apart | Unit tests with a fake instance |
| `[ui] send_tries`, `send_wait_ms` in a later copy | `kx_settings::peek` reads the file with the settings' size cap and never repairs or writes it; the `[ui]` section is checked over the defaults, and any fault gives the defaults | `MAX_FILE_BYTES` (1 MiB) | Unit tests |
| `[kernel] log_level`, `log_max_mb` | Checked by the kernel's settings; the kernel's defaults open the log when they did not load | As the kernel's note | Property tests (kernel) |
| Qt's environment (`QT_`, `QML`, `QSG_` and `QV4_` variables, in any case) | Cleared with `qunsetenv` before Qt's app object exists, in every build | All of them | Property test of the name filter; a Qt test plants them in a child process |
| Libraries named without a folder | Release: `SetDefaultDllDirectories` limits them to the app's folder and System32 before anything else runs | N/A | A test that a copy in the working folder stops loading |

## Threats (STRIDE-lite)
- **Spoofing:** a later copy speaks only through the adapter's command window, which acts on its fixed codes. A same-user program that holds `Local\KeyXtend.Running` without a command window makes the next copy stop with "another copy runs but did not answer", after its tries; it never starts a second keyboard. Each try claims again first, so a copy that quits meanwhile leaves the session to the new one.
- **Tampering:** the QML, fonts and icons are compiled in (see `kx-ui.md`); the command line is refused unless it is exactly one known flag.
- **Logging / repudiation:**
  - The log opens after the kernel loads its settings and before any module starts; when they did not load, the kernel's defaults open it, so a failed boot is logged. If the file cannot open, the app logs to standard error and says so once.
  - A later copy never opens the log, since opening sets the running copy's log aside; its refusals go to standard error only.
  - The panic hook replaces Rust's default one and logs only the file, line and column, never the payload, which could hold typed text; until the log is set up it also writes the place to standard error.
  - The first line gives the version and whether uiAccess is on; the last says the app ended.
- **Information disclosure:** nothing is sent anywhere; `cargo tree -p keyxtend` holds no network, TLS or media crate (ADR-0004).
- **Denial of service:**
  - Quit ends the request queue; the kernel's thread stops the modules, then ends the UI even if stopping panics (a drop guard). After Qt's loop, `drain::stop` removes the tray icon and joins the thread, whether it was running, already asked to quit, or ended in a panic.
  - The tray icon goes when the app ends or a panic unwinds. A panic inside a Qt callback aborts the process, and a killed process leaves the icon until the pointer passes over it, as Windows does for any app.
  - `--quit` with no copy running starts nothing and exits at once.
  - A same-user program that squats the running mutex can keep KeyXtend from starting (Accepted).
- **Elevation:** a uiAccess process can drive elevated windows, so nothing another program of the user controls may load code into it.
  - Release builds call `SetDefaultDllDirectories(APPLICATION_DIR | SYSTEM32)` first, so a library named without a folder never comes from the working folder or `PATH`.
  - Every build clears Qt's variables before Qt starts, through `qunsetenv`. Qt reads the C runtime's copy of the environment, which Rust's `remove_var` does not change, and `remove_var` is unsafe in edition 2024, which the app forbids. Clearing them also keeps a global `QT_SCALE_FACTOR` from skewing the keyboard's DPI layout.
  - Release builds pin Qt's plugin folders to the app's folder, and the QML imports to the compiled-in modules and the app's `qml` folder; each module's plugin still loads from beside its `qmldir`. Debug builds use the installed Qt.
  - The manifest asks for uiAccess only in release, with `asInvoker` and PerMonitorV2; the linker writes no import library or exports file (`/NOIMPLIB /NOEXP`). The app's code forbids `unsafe`; the hardening calls live in `kx-platform-windows` and in a `kx-ui` bridge.

## Findings
### Must fix (blocks any signed build)
- Done (fix round 1): Qt's environment could load a same-user program's plugin or QML module into the uiAccess process. Every Qt variable is now cleared before Qt starts, and release builds pin Qt's plugin and import folders.
- Done (fix round 1): libraries named without a folder could come from the working folder or `PATH`. Release builds now load them only from the app's folder and System32.
- Done (fix round 1): Qt Quick read and wrote a shader pipeline cache in a user-writable folder. It is now turned off.
### Should fix (tracked issue)
- The libraries the exe imports load before `main`, by Windows' usual order, so a Qt library missing from the stage would be looked for in `PATH`. Task 20's stage check must find every imported library in the app's folder or System32.
### Accepted (with reason)
- A same-user program can squat `Local\KeyXtend.Running`: a named object cannot be kept from the same user, who could end the process anyway.
- Commands that arrive before the hub exists wait in an unbounded list: only same-session programs can post them, one code each, during the few milliseconds of load.
- Lines the kernel logs while it loads are lost, since the log opens from the loaded settings; their notices still reach the banner.
- In the release build standard error goes nowhere, so a refused argument or a later copy's failure is silent there; the running copy's log is never touched by a later copy.

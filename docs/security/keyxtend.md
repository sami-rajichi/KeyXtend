# Security note: keyxtend (the app)

- **Owner:** product owner
- **Last review:** 2026-10-02
- **Status:** Reviewed
- **Scope:** `apps/keyxtend`: the command line, the one running copy as the app uses it, the start and stop order, the log subscriber, the panic hook, the mouse hook it hands the mouse module, the gate-only build, the manifest and Qt's start. The release build is the uiAccess process; the adapters, the kernel and the modules have their own notes.

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

Qt Quick keeps no shader pipeline cache on disk (`AA_DisableShaderDiskCache`), so the app writes nothing outside the data folder. Every QML file is compiled ahead by qmlcachegen and the stage ships no `.qml` file, so no `.qmlc` cache is written or read (Task 20); pinning that is a Should fix.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Command line | At most one argument, `--show` or `--quit`; anything else, a non-Unicode argument or any Qt option (`-platform`, `-qmljsdebugger`), stops the app before Qt starts, so Qt only ever sees a checked argument. Only the gate-only build also takes `--sim-input`, alone or before `--show` (see Gate-only input) | One argument, two in the gate-only build | Property test |
| A later copy's Show or Quit (the command window, see `kx-platform-windows.md`) | Only the adapter's fixed codes reach the app; until the hub exists they wait in order, then go to `Hub::command` | Tries stop at `[ui] send_tries`, `send_wait_ms` apart, each bounded by kx-ui's settings check | Unit tests with a fake instance |
| `[ui] send_tries`, `send_wait_ms` in a later copy | `kx_settings::peek` reads the file with the settings' size cap and never repairs or writes it; `peek_section` reads the `[ui]` section as the store does: migrated, a bad value back to its default, and the defaults for a newer version or any fault | `MAX_FILE_BYTES` | Unit tests |
| `[kernel] log_level`, `log_max_mb` | Checked by the kernel's settings; the kernel's defaults open the log when they did not load | As the kernel's note | Property tests (kernel) |
| Qt's environment (names starting `QT_`, `QML`, `QSG_` or `QV4_`, in any case, whatever follows) | Before Qt's app object exists, in every build, `qunsetenv` clears every such name found in the C runtime's narrow copy (Qt's byte reads) from it and from the wide copy (Qt's text reads); a name the ANSI code page narrows to a Qt one (a fullwidth `ＱＴ_`, a best-fit last letter) goes too | All of them | Property tests of the name filter; a Qt test plants plain names and two such aliases in a child process and checks Qt's narrow and wide reads find none |
| Libraries named without a folder | The uiAccess build: `SetDefaultDllDirectories` limits them to the app's folder and System32 first thing in `main`. The exe's own imports load before `main`, by Windows' standard order (see Elevation) | N/A | A test that a copy in the working folder stops loading; the stage check of every import |

## Threats (STRIDE-lite)
- **Spoofing:** a later copy speaks only through the adapter's command window, which acts on its fixed codes. A same-user program that holds `Local\KeyXtend.Running` without a command window makes the next copy stop with "another copy runs but did not answer", after its tries; it never starts a second keyboard. Each try claims again first, so a copy that quits meanwhile leaves the session to the new one.
- **Tampering:** the QML, fonts and icons are compiled in (see `kx-ui.md`); the command line is refused unless it is exactly one known flag, or, in the gate-only build only, `--sim-input` alone or before `--show`.
- **Logging / repudiation:**
  - The log opens after the kernel loads its settings and before any module starts; when they did not load, the kernel's defaults open it, so a failed boot is logged. If the file cannot open, the app logs to standard error and says so once.
  - A later copy never opens the log, since opening sets the running copy's log aside; its refusals go to standard error only.
  - The panic hook replaces Rust's default one and logs only the file, line and column, never the payload, which could hold typed text; until the log is set up it also writes the place to standard error.
  - Then it calls the adapter's `release_now`, so a panic during a Grab carry never leaves a button down. The hook lets go and keeps running, so a panic the kernel catches never ends mouse assist; it waits at most the adapter's `RELEASE_WAIT`, and returns at once on the hook's own thread, where the callback's own recovery lets go.
  - The first line gives the version and whether uiAccess is on; the last says the app ended.
- **Information disclosure:** nothing is sent anywhere; `cargo tree -p keyxtend` holds no network, TLS or media crate (ADR-0004). Qt's network library loads only because Qt Qml and Qt Quick import it: the stage check refuses any other importer and every TLS plugin, and a `kx-ui` test keeps network addresses and `XMLHttpRequest` out of the app's QML.
- **Denial of service:**
  - Quit ends the request queue; the kernel's thread stops the modules, then ends the UI even if stopping panics (a drop guard). After Qt's loop, `drain::stop` removes the tray icon and joins the thread, whether it was running, already asked to quit, or ended in a panic.
  - The tray icon goes when the app ends or a panic unwinds. A panic inside a Qt callback aborts the process, and a killed process leaves the icon until the pointer passes over it, as Windows does for any app.
  - `--quit` with no copy running starts nothing and exits at once.
  - A same-user program that squats the running mutex can keep KeyXtend from starting (Accepted).
- **Elevation:** a uiAccess process can drive elevated windows, so nothing another program of the user controls may load code into it.
  - One cfg, `kx_ui_access`, which `build.rs` sets together with the manifest's uiAccess, turns on the pinning below, so they cannot drift apart. A second, `kx_no_console`, hides the console in every release build, the gate-only one too, and so holds the first; tests read the built exe's manifest and subsystem.
  - The uiAccess build calls `SetDefaultDllDirectories(APPLICATION_DIR | SYSTEM32)` first in `main`, so a library named later without a folder never comes from the working folder or `PATH`.
  - The exe's own imports (Qt's DLLs, the MSVC runtime) load and set up before `main`, by Windows' standard order, which includes the working folder and `PATH`. The stage check holds each import in the stage or System32, and shipping the runtime is P14's (`docs/items/P14.md`).
  - Every build clears Qt's variables before Qt starts, through `qunsetenv`, by the names in the C runtime's narrow copy of the environment, which Qt's byte reads use; `qunsetenv` clears each from the wide copy, which Qt's text reads use, too. The narrow copy converts names with the ANSI code page's best fit, so a Win32 name such as `ＱＴ_PLUGIN_PATH` reads there as `QT_PLUGIN_PATH`; Rust's `remove_var` changes only the Win32 copy and is unsafe in edition 2024, which the app forbids. Clearing them also keeps a global `QT_SCALE_FACTOR` from skewing the keyboard's DPI layout.
  - The manifest also names the UTF-8 code page (`activeCodePage`, Windows 10 1903 and later), so in the app that copy holds each name exactly and none narrows to a Qt one; the walk over narrow names still covers a Windows that ignores it. A test reads both from the built exe.
  - The clearing runs on the main thread after the adapter's threads start: Windows locks both environments, and no thread of the app writes them.
  - The uiAccess build pins Qt's plugin folders to the app's folder, and the QML imports to the compiled-in modules and the app's `qml` folder; each module's plugin still loads from beside its `qmldir`. Debug builds use the installed Qt.
  - The manifest asks for uiAccess only in release, with `asInvoker`, PerMonitorV2 and the UTF-8 code page; the linker writes no import library or exports file (`/NOIMPLIB /NOEXP`). The app's code forbids `unsafe`; the hardening calls live in `kx-platform-windows` and in a `kx-ui` bridge.

## Gate-only input
- **What:** the `gate-input` feature turns on the adapter's `sim-input` and adds `--sim-input`, with which the mouse engine acts on the gate tool's tagged input instead of the hand's (ADR-0027 decision 3). The run logs one warning that it is a gate build.
- **Why it never ships:** in the uiAccess app, any program of the session could send that tagged input and steer our clicks into admin windows. The feature is off by default, and nothing in the workspace turns it on.
- **How it is enforced:** the stage check (`cargo xtask stage`, and `dev-install` on an app folder) refuses a program that holds the flag's text (`[workspace.metadata.stage] gate_flag`, which the app's agree test pins), and `dev-install` refuses to sign any exe that holds it. The gate build's warning logs the flag as a field, which keeps its text whole in a release exe, and a test reads the built exe: only the gate build holds the flag.

## Findings
### Must fix (blocks any signed build)
- Done (fix round 1): Qt's environment could load a same-user program's plugin or QML module into the uiAccess process. Every Qt variable is now cleared before Qt starts, and release builds pin Qt's plugin and import folders.
- Done (fix round 2): a name the ANSI code page narrows to a Qt variable (a fullwidth `ＱＴ_`, or a best-fit last letter) escaped the clearing, which read the Win32 names. The clearing now walks the C runtime's narrow names, which Qt's byte reads use, and the manifest names the UTF-8 code page outright.
- Done (fix round 1): libraries named without a folder could come from the working folder or `PATH`. Release builds now load them only from the app's folder and System32.
- Done (fix round 1): Qt Quick read and wrote a shader pipeline cache in a user-writable folder. It is now turned off.
### Should fix (tracked issue)
Only the staged uiAccess build runs the pinned paths. Task 20 checked its stage (`cargo xtask stage`), and ran a scratch copy of it whose manifest alone had uiAccess off, since Windows starts no unsigned uiAccess program outside Program Files:
- Done (Task 20): every library the exe and each staged plugin import, the MSVC runtime too, is in the stage or System32. The stage check walks their import tables and fails on any other, and the running copy loaded each one from there, with Qt's own folder first in `PATH`.
- Done (Task 20): Qt's plugins are in subfolders of the app's folder (`platforms/`, `imageformats/`) and its QML modules in `<app>/qml`, each module's plugin beside its `qmldir`; the release app loaded `main.qml` with the pinned lists.
- Done (Task 20): with `QT_PLUGIN_PATH`, `QT_QPA_PLATFORM_PLUGIN_PATH`, `QML_IMPORT_PATH`, `QML2_IMPORT_PATH` and fullwidth aliases (`ＱＴ_PLUGIN_PATH`, `ＱＴ_QPA_PLATFORM_PLUGIN_PATH`, `ＱＭＬ2_IMPORT_PATH`) planted, the app loaded nothing from their folders.
- Done (Task 20): copies of `Qt6Svg.dll`, `Qt6Network.dll` and `d3dcompiler_47.dll` in the working folder were never loaded.
- Done (Task 20): `%LOCALAPPDATA%\KeyXtend` (shader and `.qmlc` caches) was never created; the copy wrote only its portable log.
- Open (P14): typing under the planted variables, since a test click would move the owner's mouse; nothing loading from them is confirmed above.
- Open (P14): the MSVC runtime is not staged and loads from System32, where the Visual C++ redistributable puts it. On a PC without it Windows would look in the working folder and `PATH`, so the installer must provide it.
- Open (P14, before any signed build leaves the development PC): COM may read a per-user registration (`HKCU\Software\Classes\CLSID`) before the machine's for a process that is not elevated, and a uiAccess process is not. On the staged signed build, list the in-process classes it activates (Process Monitor) and check that a planted `HKCU` override for each never loads (`docs/items/P14.md`).
- Open (P14): pin the QML disk cache off in the uiAccess build, with Qt's own cache option in `harden_qt` and a stage check that refuses `.qml`, `.js` and `.mjs` files (`docs/items/P14.md`).
- Open (P14, `kx-release`): a panic inside a dependency logs that dependency's source path as the build machine saw it, which the binary also holds and which can name the builder's account. Release builds trim or remap those paths (`docs/items/P14.md`).
### Accepted (with reason)
- A same-user program can squat `Local\KeyXtend.Running`: a named object cannot be kept from the same user. A normal program cannot end the uiAccess process itself, but it can start `keyxtend.exe --quit`, so squatting gives it nothing more.
- Commands that arrive before the hub exists wait in an unbounded list: only same-session programs can post them, one code each, during the few milliseconds of load.
- Lines the kernel logs while it loads are lost, since the log opens from the loaded settings; their notices still reach the banner.
- `Qt6Network.dll` loads in the uiAccess process because Qt Qml and Qt Quick import it (ADR-0013). No TLS backend ships, the stage check lets only those two import it, and the app's QML names no network address (`kx-ui` offline test).
- In the release build standard error goes nowhere, so a refused argument or a later copy's failure is silent there; the running copy's log is never touched by a later copy.
- Variables from `HKCU\Environment`, or any a parent passes, reach the process. Every Qt name is cleared, `PATH` and the working folder reach only the exe's own imports, Qt reads `LANG` only as a locale name, and `__COMPAT_LAYER` applies only Windows' own shims.
- Fonts from outside the bundle: Native asks Windows for Segoe UI by name, and Qt falls back to the system's fonts for a glyph no theme font has. Windows also reads per-user fonts (`%LOCALAPPDATA%\Microsoft\Windows\Fonts`) and per-user GPU shader caches in every process of the user, elevated ones too, so the uiAccess process adds no new path; whether Qt's font list takes per-user fonts is not yet checked (P14, `docs/items/P14.md`).
- Any program of the session can start `keyxtend.exe --show` or `--quit`, since a uiAccess program starts without a prompt. Show only shows the keyboard, and Quit only ends it.

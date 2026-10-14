# Security note: uiAccess dev tools

- **Owner:** product owner
- **Last review:** 2026-10-14
- **Status:** Reviewed
- **Scope:** `cargo xtask stage`, `dev-cert`, `dev-install` with its Start-menu shortcut, `check-uiaccess`, and `xtask/scripts/dev-cert.ps1`, `dev-admin.ps1` and `dev-shortcut.ps1`. Developer tools only; never shipped.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| Create a certificate in the user's personal store | Windows grants uiAccess only to signed programs. | Test builds cannot be signed. |
| Trust that certificate in `LocalMachine\Root` (admin, started by the owner) | The signature must chain to a root the machine trusts. | uiAccess is refused, so gates G3 and G4 cannot run. |
| Write and delete `Program Files\KeyXtend-dev\<name>` (admin, Windows prompt) | Windows grants uiAccess only from a secure folder. | Same as above. |
| Read a program's signature and manifest | `check-uiaccess` explains why uiAccess is missing. | Only diagnosis is lost. |
| Write and delete one shortcut in the user's own Start menu | The owner starts the installed app with one click. | The app is started from its folder instead. |
| Hold a low-level mouse hook while a gate runs (`kx-gates`, the mouse guard) | A hand resting on the mouse moves it with each breath, which shows tips and spoils clicks mid-run. | Gates that move the pointer can be disturbed by the owner's hand. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Test certificate private key | Secret | Windows user key store, non-exportable | By Windows | 90 days, or until `dev-cert --remove` when the owner decides (ADR-0014) |
| Exported certificate, launchers, admin error file | Public | `target/dev-tools/` | No | Until `target/` is cleaned |
| Extracted manifest | Public | `target/dev-tools/` | No | Deleted after each check |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[workspace.metadata.devtools]` and `[workspace.metadata.stage]` | Typed; unknown keys, empty or zero values, exit codes below 1 or equal, a subject with `"`, and non-plain names or paths rejected; `cargo xtask tidy` checks both in CI | N/A | No |
| `QMAKE` and `qmake -query` | An absolute path, else an error; each Qt folder must be absolute, and Qt's version must be `qt_version`, the one `THIRD_PARTY.md` reviewed | N/A | Unit tests |
| Staged programs and libraries (PE import tables) | Every read is bounds-checked and a malformed file fails the stage, never panics; only descriptors with image-relative places are read | The file's own length | Property test |
| `APPDATA` and `USERPROFILE` for the shortcut | Both absolute, else an error, checked before anything is signed, installed or removed; the shortcut goes only under the Start-menu folder of `APPDATA`, and only inside `USERPROFILE`, never `ProgramData` | N/A | Unit tests |
| Script output (thumbprints) | Exactly 40 hex digits each, else an error | N/A | No |
| The running copy's `settings.toml`, for the gates | Gates read its `[keyboard]` and `[ui]` sections through `kx_settings::peek` (size cap, never writes) and `peek_section`, which checks it as the store does: a bad value gives its default, and a section that does not check gives the defaults | `MAX_FILE_BYTES`; `relabel_ms` capped by the keyboard's settings check | Unit tests |
| `dev-install` folder and `--remove` name | One plain folder name; separators, `..`, wildcards, device names and trailing dots rejected (property test); the source must hold a program, no links or junctions, and must not overlap the target | Source capped by `install_max_mb` in Rust; the admin script re-checks the target and links | Property test |
| `check-uiaccess` path and `ProgramFiles` variables | Compared folder by folder, ignoring case; `.` and `..` steps rejected (property test) | N/A | Property test |
| Manifest XML from `mt.exe` | Comments skipped; only `uiAccess` on `requestedExecutionLevel` (any prefix) is read; missing means fail | Developer-chosen file | No |
| Mouse hook events (`kx-gates` mouse guard) | Only the message, the injected flag, the injection tag and the X button's number are read, never the position; any other message is a plain event that drops when physical | One event per call; the hook does no I/O, locking or allocation, and counts with atomics | Unit tests of the filter |
| The hook's stats lines (`kx-gates`, `[gate_input] hook_line` in the test build's log, or in the running copy's for the hand try) | Only lines stamped since the test build or the hand try started are read; each named field must be a whole number, else the read fails naming it, and nothing else on the line is kept | The log's own `log_max_mb` | Unit tests |

## Threats (STRIDE-lite)
- **Spoofing:** a same-user program could edit a launcher or `dev-admin.ps1` before the owner opens it, and so run code as admin. This is a known risk; see Accepted.
  - Launchers call PowerShell by its full path under `%SystemRoot%`, so a fake `powershell.exe` on the PATH is not used.
- **Tampering:** a swapped `.cer` file. Mitigation: the trust step imports it only if its thumbprint matches the one in the launcher.
  - A network or TLS plugin, or a library nothing loads, slipping into the signed uiAccess install. Mitigation: the stage copies only the files `[workspace.metadata.stage]` lists, and its check refuses TLS, network-information and QML debugging plugins, any import found neither in the stage nor in System32, any library nothing imports, and Qt's network library imported by anything but Qt Qml and Qt Quick.
  - A staged file the check never reads, or a network name the guard never meets. Mitigation: `libs` and `plugins` may name only `.dll` files, which the check reads, and the network library and its importers must be listed in `libs`.
  - A refused or half-copied stage being signed later. Mitigation: the stage removes `<out_dir>/<app>` when its copy or check fails, and `dev-install` runs the same check on a folder holding the app before it signs anything.
  - A gate-only build (the app's `gate-input`, which acts on another program's tagged input) getting signed and uiAccess. Mitigation: the stage check refuses a program holding `[workspace.metadata.stage] gate_flag`, and `dev-install` refuses to sign any exe that holds it.
  - `kx-gates` reads `kx-gates.toml`, then `kx-gates-p3.toml` and `kx-gates-p4.toml` from the same folder, beside its exe, else from the repo's `tools/kx-gates` (user-writable); they name the programs it starts, PresentMon included, and those it starts without the admin prompt (`[apps.*]`, `[g8] program`) inherit its token. It reads none when Windows will not say whether it has uiAccess. With uiAccess it reads all five of its settings files (those three, `spike.toml` and `kx-target-window.toml`) only beside its exe, and refuses to run when one is missing, so a repo edit cannot hand uiAccess to a program. `dev-install` copies every file of the build folder, so put the five files there first. A relative path starts at its file's folder, so those copies write the repo paths in full, while `[g8] program` stays relative to reach the installed app.
  - P4's driving gates run the product's test build (`[gate_input]` in `kx-gates-p4.toml`), whose engine acts on kx-gates' tagged input. It never holds uiAccess: its build script leaves uiAccess out of the gate-input build, and `kx-gates` refuses to start it while it has uiAccess itself or Windows will not say, so only the plain kx-gates starts it. It starts only a program whose bytes hold the gate flag, makes a `data` folder beside it so it never reads or writes the owner's settings or log, and quits it at once when its log lacks the gate warning. It asks the running copy to quit first and starts that copy again once the test build has ended, else prints a line to start it from the Start menu; it never kills the test build, which could leave an injected button down. A uiAccess copy refuses the plain kx-gates, which then stops and says to close it from the installed kx-gates.
  - G5 (`plain` only), G12, G17, G18, `g8 hook` and `g8 hook-weak` drive the test build: they turn its modes and latches on with its own keys and send only `SIM_TAG` input. `g8 hook-weak` starts it in `g8 weak`'s job object, which ends it if it is still running when the gate ends. G5's admin-window case is the owner's manual check on the installed product, as the test build never holds uiAccess. G5's hung case posts target-window a private message (`WM_APP` plus an offset) that makes it sleep, capped by its `hang_max_ms`, which must stay below Windows' 5 s hung-window mark, so each message from any program on the desktop pauses that test window at most that long. The hand try (`kx-gates assist`) starts nothing: it refuses a running test build and only reads the owner's copy's stats lines.
  - `kx-gates close keyxtend` posts Quit to the running copy's command window, as `--quit` does; it starts nothing.
  - `kx-gates g8` starts the program `[g8] program` names through the shell, so the same rule holds. With `weak`, it caps that program and kx-gates itself in one job object (cores, CPU time, memory) until kx-gates exits; a copy it cannot open or cap gets Quit.
  - When `[g8] presentmon_exe` names PresentMon, `kx-gates g8` starts it as administrator, as G4 does, but with no window. It records for `presentmon_secs` and stops itself, and G8 waits for that before it reads the CSV. `kx-gates g12` starts the same PresentMon the same way, for `[g12] presentmon_secs` into its own CSV, `[g12] presentmon`, which must be a plain file name in the out folder.
  - `kx-gates g8` presses `[g8] press_key`, which types at the press, with only target-window in front: it brings that window back before the pointer rests on the key and again before each press, or stops, so the characters reach no other window, even from a keyboard that types on hover.
  - `kx-gates g8 lit` presses the running product's key the same way, with no start or quit. `kx-gates g8 osk` presses the shown On-Screen Keyboard's `[osk] key`, found by UI Automation, also into target-window alone; it stops when osk is not shown or another window covers the key, and never starts, closes or fronts osk.
  - G8 ends with a churn, after PresentMon's capture: it moves the real pointer over keys without clicking, and presses the language key (List design with two or more layouts only), minimise, the bubble and the theme button by UI Automation. Each round presses the theme button once per theme, so the theme ends where it started, and G8 stops and says so when it does not; G8 only reads the product's memory and never trims it.
  - `kx-gates g9` turns off topmost and clears the guard's style flags on the keyboard and its guarded windows, as an outside program could, then puts each back as it found it. It writes click-through on to every one of them, which the guard must refuse where a window took clicks at first, and never touches another program's windows. When target-window loses the front, it records the window in front by class and program only, never its title. It also reads whether the guard's mark (`KeyXtend.Guard`, a `GetPropW` presence check) is on each of our windows, as a bool; the keyboard's mark at the first look is the control that the mark can be read at all. It names each of our windows' owner by handle only.
  - `kx-gates g10` presses every key by UI Automation, as Narrator does, with only target-window in front, except a dead key or a key that types nothing in the layout in front. It presses Caps Lock twice, sends each modifier only with Enter, lets Win go with no key, and puts back a layout the language key changed. It also asks UI Automation which element is at each key's centre, as Narrator's mouse mode does, without moving the pointer. It records that element's class, control type, process and window, and the window under the point, but reads an element's id and name only when the keyboard shows it. `kx-gates g26` switches the session's input language through `kx-target-window --layout`, so it runs on the GitHub runner only.
- **Logging / repudiation:** the tools print only thumbprints and paths; no secrets. The mouse guard prints one line when it starts and one when it ends, with why, never a position or a button. Its result adds two numbers, the hand's events it dropped and its own it passed, so each run shows its hook was live.
- **Information disclosure:** no network; the private key cannot be exported.
- **Denial of service:** every step ends with an exit code. While a gate runs, `kx-gates` drops every physical mouse move, click and wheel turn and passes only the input it injected and tagged itself (`[mouse_guard]` in `kx-gates-p3.toml`), so it holds the owner's only input. Mitigations:
  - It lets go when the gate ends or breaks off, on a panic (a drop guard), while an admin prompt is up, once G8's or G12's pointer work is over, after `limit_ms` (15 minutes) of holding even if the gate hangs, and at once on the owner's first physical button press (`escape_clicks`, `escape_window_ms`), as the owner cannot hold a button. That press drops and its release passes.
  - The limit, the owner's click and a guard that cannot come back after a prompt stop the gate: it then fails, shows no prompt and sends no new press, and only the steps that put things back still go out.
  - Windows silently removes a low-level hook that misses its timeout (at most 1 s; Microsoft's `LowLevelMouseProc` docs), so the guard fails closed. Once kx-gates sent its own click, or a move far enough from the pointer to surely move it, its hook must count one own event within `alive_ms` (twice Windows' 1 s hook timeout); else the gate stops as lost, and no extra input is sent to check. Stretches with no input at all stay unwatched.
  - The hook lives on a thread of `kx-gates`, and Windows removes a low-level hook when its thread or process ends, so it never outlives a run; it changes no Windows setting.
  - A release whose press came before the guard passes, so no button sticks. After a stop the gate's own releases still go out without their move, except one away from a key the gate pressed, so no click lands.
  - Gates that need the owner's own mouse, run on the CI runner or move nothing are marked free in the gate table in `tools/kx-gates/src/cli.rs`, so they are never guarded.
  - The test build's hook sits ahead of the guard's and replays a held press tagged `REPLAY`, so the guard passes `REPLAY` input only for the gates the table marks as driving it (G5, G12, G17, G18, and G8's `hook` and `hook-weak` by name). For every other gate a `REPLAY` press is the hand's: it drops and gives the mouse back. A `REPLAY` value on input that was not injected is always the hand's.
- **Elevation:**
  - A root-trusted key could, in theory, vouch for other certificates. Mitigation: key usage is digital signature only, the only purpose is code signing, and the certificate is marked "not a CA".
  - An elevated folder delete could hit the wrong folder. Mitigation: `dev-admin.ps1` checks every argument before the Windows prompt, and only touches a direct child of `Program Files\<install dir>`; `install_dir` must be a plain name.
  - Untrust could remove someone else's root. Mitigation: it removes only roots with our subject **and** a thumbprint from our own list.
  - `--remove` could delete another shortcut. Mitigation: it deletes ours only when it starts that install's program.
  - The stage empties `<out_dir>/<app>` first. Mitigation: both settings are plain relative names, so the folder is always inside the workspace.
  - Program files signed with the test key would show a verified publisher on this PC. Mitigation: the trust is removed when the owner decides, at the latest when the certificate expires (`dev-cert --remove` plus the untrust launcher; ADR-0014).

## Findings
### Must fix (blocks Definition of Done)
- Done: the certificate is marked "not a CA" and its key usage is pinned. The old certificate is recreated before it is trusted.
- Done: an `install_dir` that is not a plain folder name is rejected, in the settings and in `dev-admin.ps1`.

### Should fix (tracked issue)
- Done (P4): with uiAccess, `kx-gates` reads its settings only beside its exe, all five files, and refuses to run when one is missing, so the repo's user-writable copies can no longer hand uiAccess to a program.

### Accepted (with reason)
A same-user attacker who can edit files in the repo can already act as the owner. The launchers and scripts run only on the owner's own development PC, and Windows shows its admin prompt each time.

`-ExecutionPolicy Bypass` applies only to the one PowerShell process that runs our scripts; it changes no system setting.

The overlap check compares path text, so a short 8.3 name for the install folder slips past it. The worst case is losing that test install, which `dev-install` recreates.

The mouse guard's lost-hook check misses a hook lost within the last `alive_ms` before a gate ends, and a late count of an earlier batch can hide one lost batch until the next. Waiting at every gate's end, or sending input only to check, would slow every run or move the owner's pointer.

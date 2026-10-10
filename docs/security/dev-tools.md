# Security note: uiAccess dev tools

- **Owner:** product owner
- **Last review:** 2026-10-02
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

## Threats (STRIDE-lite)
- **Spoofing:** a same-user program could edit a launcher or `dev-admin.ps1` before the owner opens it, and so run code as admin. This is a known risk; see Accepted.
  - Launchers call PowerShell by its full path under `%SystemRoot%`, so a fake `powershell.exe` on the PATH is not used.
- **Tampering:** a swapped `.cer` file. Mitigation: the trust step imports it only if its thumbprint matches the one in the launcher.
  - A network or TLS plugin, or a library nothing loads, slipping into the signed uiAccess install. Mitigation: the stage copies only the files `[workspace.metadata.stage]` lists, and its check refuses TLS, network-information and QML debugging plugins, any import found neither in the stage nor in System32, any library nothing imports, and Qt's network library imported by anything but Qt Qml and Qt Quick.
  - A staged file the check never reads, or a network name the guard never meets. Mitigation: `libs` and `plugins` may name only `.dll` files, which the check reads, and the network library and its importers must be listed in `libs`.
  - A refused or half-copied stage being signed later. Mitigation: the stage removes `<out_dir>/<app>` when its copy or check fails, and `dev-install` runs the same check on a folder holding the app before it signs anything.
  - The signed uiAccess `kx-gates` reads `kx-gates.toml`, and `kx-gates-p3.toml` from the same folder, beside its exe, else from the repo's `tools/kx-gates` (user-writable); they name the programs it starts, PresentMon included, and those it starts without the admin prompt (`[apps.*]`, `[g8] program`) inherit its uiAccess token. `dev-install` copies every file of the build folder, so put both files there first and keep them beside the installed exe. A relative path starts at its file's folder, so those copies write the repo paths in full, while `[g8] program` stays relative to reach the installed app.
  - `kx-gates close keyxtend` posts Quit to the running copy's command window, as `--quit` does; it starts nothing.
  - `kx-gates g8` starts the program `[g8] program` names through the shell, so the same rule holds. With `weak`, it caps that program and kx-gates itself in one job object (cores, CPU time, memory) until kx-gates exits; a copy it cannot open or cap gets Quit.
  - When `[g8] presentmon_exe` names PresentMon, `kx-gates g8` starts it as administrator, as G4 and G5 do, but with no window. It records for `presentmon_secs` and stops itself, and G8 waits for that before it reads the CSV.
  - G8 ends with a churn: it moves the real pointer over keys without clicking, and presses the language key and the theme button by UI Automation. Each round presses the theme button once per theme, so the theme ends where it started; G8 only reads the product's memory and never trims it.
  - `kx-gates g9` turns off topmost and clears the guard's style flags on the keyboard and its guarded windows, as an outside program could, then puts each back as it found it. It writes click-through on to every one of them, which the guard must refuse where a window took clicks at first, and never touches another program's windows.
  - `kx-gates g10` presses every key by UI Automation, as Narrator does, with only target-window in front, except a dead key or a key that types nothing in the layout in front. It presses Caps Lock twice, sends each modifier only with Enter, lets Win go with no key, and puts back a layout the language key changed. It also asks UI Automation which element is at each key's centre, as Narrator's mouse mode does, without moving the pointer. It records that element's id, class, control type, process and window, and the window under the point, but keeps an element's name only when the keyboard shows it. `kx-gates g26` switches the session's input language through `kx-target-window --layout`, so it runs on the GitHub runner only.
- **Logging / repudiation:** the tools print only thumbprints and paths; no secrets.
- **Information disclosure:** no network; the private key cannot be exported.
- **Denial of service:** not relevant for a developer tool; every step ends with an exit code.
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
The uiAccess `kx-gates` falls back to the repo's user-writable `kx-gates.toml` when no copy sits beside its exe, so an edited config could hand uiAccess to any program, with no admin prompt (P4, `docs/items/P4.md`). Until P4 reads it only beside the exe when the token has uiAccess, the manual check copies both files beside the installed exe.

### Accepted (with reason)
A same-user attacker who can edit files in the repo can already act as the owner. The launchers and scripts run only on the owner's own development PC, and Windows shows its admin prompt each time.

`-ExecutionPolicy Bypass` applies only to the one PowerShell process that runs our scripts; it changes no system setting.

The overlap check compares path text, so a short 8.3 name for the install folder slips past it. The worst case is losing that test install, which `dev-install` recreates.

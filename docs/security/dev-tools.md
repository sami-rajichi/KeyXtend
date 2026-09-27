# Security note: uiAccess dev tools

- **Owner:** product owner
- **Last review:** 2026-09-27
- **Status:** Reviewed
- **Scope:** `cargo xtask dev-cert`, `dev-install`, `check-uiaccess`, and `xtask/scripts/dev-cert.ps1` and `dev-admin.ps1`. Developer tools only; never shipped.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| Create a certificate in the user's personal store | Windows grants uiAccess only to signed programs. | Test builds cannot be signed. |
| Trust that certificate in `LocalMachine\Root` (admin, started by the owner) | The signature must chain to a root the machine trusts. | uiAccess is refused, so gates G3 and G4 cannot run. |
| Write and delete `Program Files\KeyXtend-dev\<name>` (admin, Windows prompt) | Windows grants uiAccess only from a secure folder. | Same as above. |
| Read a program's signature and manifest | `check-uiaccess` explains why uiAccess is missing. | Only diagnosis is lost. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Test certificate private key | Secret | Windows user key store, non-exportable | By Windows | 90 days, or until `dev-cert --remove` at the end of P1 |
| Exported certificate, launchers, admin error file | Public | `target/dev-tools/` | No | Until `target/` is cleaned |
| Extracted manifest | Public | `target/dev-tools/` | No | Deleted after each check |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[workspace.metadata.devtools]` | Typed; unknown keys, empty or zero values, exit codes below 1 or equal, a subject with `"`, and non-plain names rejected; `cargo xtask tidy` checks it in CI | N/A | No |
| Script output (thumbprints) | Exactly 40 hex digits each, else an error | N/A | No |
| `dev-install` folder and `--remove` name | One plain folder name; separators, `..`, wildcards, device names and trailing dots rejected (property test); the source must hold a program, no links or junctions, and must not overlap the target | Source capped by `install_max_mb` in Rust; the admin script re-checks the target and links | Property test |
| `check-uiaccess` path and `ProgramFiles` variables | Compared folder by folder, ignoring case; `.` and `..` steps rejected (property test) | N/A | Property test |
| Manifest XML from `mt.exe` | Comments skipped; only `uiAccess` on `requestedExecutionLevel` (any prefix) is read; missing means fail | Developer-chosen file | No |

## Threats (STRIDE-lite)
- **Spoofing:** a same-user program could edit a launcher or `dev-admin.ps1` before the owner opens it, and so run code as admin. This is a known risk; see Accepted.
  - Launchers call PowerShell by its full path under `%SystemRoot%`, so a fake `powershell.exe` on the PATH is not used.
- **Tampering:** a swapped `.cer` file. Mitigation: the trust step imports it only if its thumbprint matches the one in the launcher.
- **Logging / repudiation:** the tools print only thumbprints and paths; no secrets.
- **Information disclosure:** no network; the private key cannot be exported.
- **Denial of service:** not relevant for a developer tool; every step ends with an exit code.
- **Elevation:**
  - A root-trusted key could, in theory, vouch for other certificates. Mitigation: key usage is digital signature only, the only purpose is code signing, and the certificate is marked "not a CA".
  - An elevated folder delete could hit the wrong folder. Mitigation: `dev-admin.ps1` checks every argument before the Windows prompt, and only touches a direct child of `Program Files\<install dir>`; `install_dir` must be a plain name.
  - Untrust could remove someone else's root. Mitigation: it removes only roots with our subject **and** a thumbprint from our own list.
  - Program files signed with the test key would show a verified publisher on this PC. Mitigation: the trust is removed at the end of P1 (`dev-cert --remove` plus the untrust launcher).

## Findings
### Must fix (blocks Definition of Done)
- Done: the certificate is marked "not a CA" and its key usage is pinned. The old certificate is recreated before it is trusted.
- Done: an `install_dir` that is not a plain folder name is rejected, in the settings and in `dev-admin.ps1`.

### Should fix (tracked issue)
None.

### Accepted (with reason)
A same-user attacker who can edit files in the repo can already act as the owner. The launchers and scripts run only on the owner's own development PC, and Windows shows its admin prompt each time.

`-ExecutionPolicy Bypass` applies only to the one PowerShell process that runs our scripts; it changes no system setting.

The overlap check compares path text, so a short 8.3 name for the install folder slips past it. The worst case is losing that test install, which `dev-install` recreates.

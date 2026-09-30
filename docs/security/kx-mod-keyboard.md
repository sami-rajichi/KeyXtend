# Security note: kx-mod-keyboard

- **Owner:** product owner
- **Last review:** 2026-09-30
- **Status:** Draft
- **Scope:** `crates/kx-mod-keyboard`, which runs inside the uiAccess `keyxtend` process from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None yet | The module only reads its settings so far. | Nothing. |

- Typing (P3 Task 12) and the language key (Task 13) will add `InjectInput`, which gates the `InputInjector` and `Layouts` services; this table changes with them.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| `[keyboard]` settings: size, place, modifiers mode, language-key style, bubble corner, `key_apps` | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |

No typed text, clipboard content or secret passes through this module yet.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[keyboard]` settings | `deny_unknown_fields`; choices by name; sizes finite, inside `size_min..=size_max`, step and `relabel_ms` above 0; no blank app names | The kernel caps the settings file (`MAX_FILE_BYTES`) | No |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network.
- **Tampering:** a hand-edited value is checked, and a bad one resets alone (kernel repair).
- **Logging / repudiation:** the module never logs.
- **Information disclosure:** the settings are not sensitive.
- **Denial of service:** a size or time of zero or not-a-number is refused, so nothing divides by zero or spins.
- **Elevation:** nothing yet; typing will send input only through the `InputInjector` port.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- None.

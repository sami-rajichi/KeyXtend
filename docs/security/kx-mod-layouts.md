# Security note: kx-mod-layouts

- **Owner:** product owner
- **Last review:** 2026-10-02
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-layouts`, which runs inside the uiAccess `keyxtend` process from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `InjectInput` | The layout source it reads is gated by `InjectInput`, because its switch request acts on the app in front; the `Layouts` service it provides is gated the same way, and a provider must hold its service's capability. | The module fails to start, so the keyboard has no key labels and no language switch. |

- `LayoutChanged` names no capability, so any module may hear which layout is shown; a layout id is not sensitive.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| What each key types per layout, language facts and labels | Public | Memory only | N/A | One table per installed layout, until it is uninstalled or the module stops |
| `[layouts] short` | Public | `settings.toml` (only the user's changes) | No | Until the user changes it |

No typed text, clipboard content or secret passes through this module.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Layout list, layout in front, key texts and facts from `LayoutSource` | Duplicate ids dropped; a layout in front that is not installed is never shown; a short or empty answer only leaves keys out | Key texts capped by the adapter (`TEXT_CAP`); keys read only for `scans.toml`'s codes, once per layout | Property tests over the fake (random adds, removals, switches and failures) |
| `[layouts] short` | `deny_unknown_fields`; keys are two or three lower-case ISO 639 letters; names may not be blank | Names hold at most `MAX_SHORT_CHARS` characters; the kernel caps the settings file (`MAX_FILE_BYTES`) | No |
| `scans.toml` | Compiled in; parsed with `deny_unknown_fields` and a range check, and a broken table fails the start | Fixed at build time | N/A |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network. The layout in front comes from the adapter; an app cannot make the keyboard show a layout that is not installed.
- **Tampering:** a hand-edited `short` value is checked, and a bad one resets alone (kernel repair). The scan table is compiled in, never read from disk.
- **Logging / repudiation:** the module never logs.
- **Information disclosure:** the installed languages are visible to any module through `list`; they are not sensitive.
- **Denial of service:** tables are read once per layout and labels once per list change; `refresh` reads only the list and the app in front. A source error never panics and keeps the shown layout.
- **Elevation:** `step` and `pick` only ask the app in front through the adapter (`WM_INPUTLANGCHANGEREQUEST` on Windows) and only for installed layouts; `pick` refuses an id not in the list before asking. Both read the list under the lock and ask only after letting it go, and a step that lands on the shown layout asks nothing.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- **Our own window in front (P13):** with Settings in front, the keyboard follows our own thread's layout, as the adapter's `foreground` does not tell our windows apart. P13 hides Settings from the layout source (`docs/items/P13.md`); until then the keyboard only shows the layout Settings types in.
### Accepted (with reason)
- **A misleading short name:** a hand-edited `short` value can label a language as another (`ar = "EN"`). The space bar and the legends still show the real layout, and only the user's own programs can write the file.

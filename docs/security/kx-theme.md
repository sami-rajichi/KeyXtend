# Security note: kx-theme

- **Owner:** product owner
- **Last review:** 2026-09-30
- **Status:** Draft
- **Scope:** `crates/kx-theme`, a library with no Qt and no module of its own; the `ui` module in the uiAccess `keyxtend` process calls it from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | It only turns tokens and the system look into colours, sizes and motion; it sends no input and reads no files. | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Theme tokens (`tokens/*.toml`) | Public | Compiled into the binary | N/A | Build lifetime |
| The system look: dark mode, accent pair, high-contrast colours, animations off | Public | Memory only | N/A | Until the next look |

No typed text, clipboard content or secret passes through this crate. The `[ui]` look settings reach it as a `LookChoice` that the `ui` module has already checked.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Token files | Compiled in with `include_str!`, never read from disk (ADR-0016); parsed with `deny_unknown_fields` and range checks, and a broken file fails `Themes::shipped` with the file and token named | Fixed at build time | No; tests cover each rule |
| `SysLook` from the `SystemLook` port | Every value is a valid colour (four bytes) or flag, so nothing can fail; an unknown theme id gives no look, never a panic | Fixed size | No |
| `LookChoice.min_contrast` | The accent never drops a legend below `WCAG_AA`, whatever the value, NaN included; the `[ui]` check refuses values outside `MIN_CONTRAST` | N/A | No |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network.
- **Tampering:** a token file swapped on disk changes nothing, since the tokens are compiled in. Icon names are plain file names (lower-case letters, digits and `-`), so a name can never point outside the icon folder when `kx-ui` builds its resource path.
- **Logging / repudiation:** the crate never logs; its errors name only token files and tokens.
- **Information disclosure:** the system look is not sensitive.
- **Denial of service:** loading is a fixed, small parse at start-up; the look view is pure and bounded by the token count.
- **Elevation:** none; the crate runs no code from its input and `#![forbid(unsafe_code)]` holds.

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- Dimmed keys read at about 1.9:1 to 3.0:1 against their faded cap: WCAG sets no minimum for inactive parts (1.4.3), their names say "Coming in a later version", and high contrast shows them in `GrayText`. The owner confirms the look at the P3 look check.

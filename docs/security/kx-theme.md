# Security note: kx-theme

- **Owner:** product owner
- **Last review:** 2026-10-02
- **Status:** Reviewed
- **Scope:** `crates/kx-theme`, a library with no Qt and no module of its own; the `ui` module in the uiAccess `keyxtend` process calls it from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | It only turns tokens and the system look into colours, sizes and motion, and sends no input. The app's code reads no files: `assets::bundle` walks the repo's `assets/` folder only for kx-ui's build script and the tests. | N/A |

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
| `LookChoice.min_contrast` and the system accent | A legend on the accent reaches at least `WCAG_AA` whatever the value, NaN included; a higher value is best effort, and the `[ui]` check refuses values outside `MIN_CONTRAST`. The accent is kept only when every pair the keyboard draws still passes; otherwise the theme keeps its own colours | N/A | No; a sweep of accents |
| The board's row height (`row_px`, from the keyboard's face) | The drawn check measures each legend on it; a row that is not a finite number above 0 tries no accent | N/A | No |
| Fonts | The bundled fonts are compiled into kx-ui; Native names Windows' own Segoe UI (`SYSTEM_FONTS`), and Qt falls back to system fonts for a glyph no theme font has (`keyxtend.md`, Accepted) | N/A | No |

## Threats (STRIDE-lite)
- **Spoofing:** no IPC or network.
- **Tampering:** a token file swapped on disk changes nothing, since the tokens are compiled in. Icon names are plain file names (lower-case letters, digits and `-`), so a name can never point outside the icon folder when `kx-ui` builds its resource path.
- **Logging / repudiation:** the crate never logs; its errors name only token files and tokens.
- **Information disclosure:** the system look is not sensitive.
- **Denial of service:** loading is a fixed, small parse at start-up; the look view and the drawn check are pure and bounded by the token count.
- **Elevation:** none; the crate runs no code from its input and `#![forbid(unsafe_code)]` holds.

## Findings
### Must fix (blocks Definition of Done)
- Done (P3, 7666e09, fe5a6fe, f9bdea4, a918bc4): the tests checked flat token pairs only, so Dolch's sculpted top, the hover and press tints and the lit size letter fell below their minimums. `drawn` now checks what the keyboard really draws, each legend where it sits on its cap on the board's own row, and the nudged tokens are marked in `themes.toml`.
- Done (P3, 302c48b, 658a5a8): the system accent replaced the theme's colours unchecked. `with_accent` keeps it only when every drawn pair still passes on the keyboard's rows, and without a measurable row the theme keeps its own colours.
- Done (P3, a049b94): high contrast kept the caps' gradient tints and mixed system pairs. It now draws flat caps and only pairs Windows makes readable.
### Should fix (tracked issue)
- None.
### Accepted (with reason)
- Dimmed keys read at about 1.9:1 to 3.0:1 against their faded cap: WCAG sets no minimum for inactive parts (1.4.3), their names say "Coming in a later version", and high contrast shows them in `GrayText`. The owner confirms the look at the P3 look check.
- The Carousel's side names keep the mock-up's 45 % fade as decoration, below 4.5:1 (owner's ruling, 2026-10-02); the current layout's name beside them stays opaque. `drawn::EXCEPTION` names this one pair, and a test fails as soon as any other falls below its minimum.
- Over a system backdrop the plate keeps 88 % of its colour, and the check measures it over Windows' plain window colour in the keyboard's mode, so a wallpaper of the opposite brightness can drop plate text to about 3.5:1. The backdrop is off by default, and its full check comes with Settings (`docs/items/P13.md`).
- The check measures each legend over the Latin capital height; accents, brackets and tall or descending Arabic letters reach a little beyond it, where a nudged token can read about 4.4:1. The owner checks those keys by eye (2026-10-02).

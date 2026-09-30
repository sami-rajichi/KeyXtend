# kx-theme

The theme tokens and the look: three themes (Native Adaptive, ET66, Modern Dolch), each light and soft dark, the sizes, motion and icons they share, contrast checks and the high-contrast palette. It has no Qt and is not a module (it has no manifest, services or settings of its own), so its tests run on every system and the spike can use it. The `ui` module in `kx-ui` calls it (ADR-0016).

## Tokens

The four files in `tokens/` are compiled in with `include_str!`, never read from disk, so the uiAccess app reads nothing it draws from a file (ADR-0016). Values come from `design/keyboard-style-lab.html`; each comment names the mock-up rule and line.

| File | What it holds |
|---|---|
| `themes.toml` | Each theme's shape and type (radii, gap, legend sizes, fonts, cap build, D-pad colours, whether it can take the system accent) and its `light` and `dark` palettes; the `[common]` colours every theme shares. |
| `shape.toml` | Sizes all themes share, at size 1: the top bar, chips, legends, LED, the language key (Carousel, `[lang.cycle]`, `[lang.list]`), D-pad, press, tooltips, the bubble and corner, hairlines, dimmed keys (`[dim]`), the high-contrast hover outline (`[hc_outline]`), the notice banner, the tray menu, the language list and the later windows (pill, caption, snip, panel, hold ring). |
| `motion.toml` | Each animation's length and curve, its amounts, and the hold ring's curves; reduced motion keeps the curves with every length 0, except the ring's waves, which show the hold time. |
| `icons.toml` | The icon each part shows, as a file name in `assets/icons/lucide/` (Lucide) or `assets/icons/` (our own drawings): named keys by board id, the top bar (the theme button shows `palette`), the D-pad and the language key designs. |

`Themes::shipped()` loads and checks all four; an error names the file and the token. Unknown or missing tokens, weights outside 100–900, shares outside 0–1, a curve that runs back in time, a burst that leaves the ring window, an accent on sculpted caps and an icon name that is not a plain file name are refused.

Dimmed keys (keys that come in a later version) have no colours of their own: every palette's `dim` and `dim_ink`, and Dolch's skirt `dim`, are the side key faded over the plate at `[dim] share` (the mock-up's disabled opacity). A palette file may not set them.

## The look

`look_view(themes, choice, sys)` gives a face everything it draws, as `LookView` (serialized to JSON for QML):

- `LookChoice` is what the user chose: the theme id, `ModeChoice` (light, dark or auto), `follow_accent`, `reduced_motion` and `min_contrast`, all from the `[ui]` settings.
- `sys` is the system look from the `kx-platform` port (`SysLook`): dark mode, the accent pair, the seven high-contrast colours and animations off.
- **Mode:** `mode_for` shows the chosen mode, or the system's with `auto`.
- **Accent:** only a theme with `accent = true` (Native) takes it, and only when `follow_accent` is on. Enter, on, locked, the LED, the ring and the badge take the accent shade for the mode; an ink below `min_contrast` switches to the main legend if that reads better. The look view never uses less than WCAG AA, even if the setting says so.
- **High contrast:** every colour becomes one of the seven system colours, keys get a 1 px inner outline, dimmed keys show `GrayText` on `ButtonFace`, and `hover_line` is `Highlight` (drawn `[hc_outline] px` wide) instead of a tint, which is not a system colour.
- **Motion:** every length is 0 with the reduced-motion setting or the system's animations off.

## Contrast

`contrast` holds the WCAG 2.x maths and the standard's minimums: `WCAG_AA` (4.5:1 for text), `WCAG_SHAPES` (3:1 for shapes such as the ring) and `MIN_CONTRAST` (`WCAG_AA..=21`), the only range the `[ui] min_contrast` check may accept, so the setting's floor is this constant and not a second copy. The shipped-theme test checks every legend pair against `WCAG_AA`; the only pairs below it are the owner's exceptions from the look check (2026-09-28), listed in `OWNER_KEPT` in `src/theme/tests.rs`. Dimmed keys are left out, since WCAG sets no minimum for inactive parts (1.4.3).

## QML look fixtures

`tests/fixtures.rs` writes the look JSON the Qt Quick tests read into `fixtures/` (one per theme and mode, plus Native light with reduced motion and in high contrast), as `.pragma library` files with `var lk`. A token change fails the test until it runs with `KX_BLESS=1`, so QML tests never hardcode a token.

## Public API

`Themes` (`shipped`, `find`), `ThemeError`, `Theme`, `Look`, `Cap`, `Gradient`, `DpadRefs`, `Dpad`, `Palette`, `Skirt`, `Shadow`, `Rgba` (`parse`, `over`, `CLEAR`), `Common`, `Shape`, `Motions`, `Motion`, `Timed`, `Amounts`, `MOVES`, `RingMotion`, `RingAmounts`, `Icons` (`names`) with `BarIcons`, `DpadIcons`, `LangIcons`, `Mode`, `ModeChoice`, `mode_for`, `LookChoice`, `LookView`, `look_view`, `reach`, `accent_fill`, `with_accent`, `contrast_palette`, `contrast_common`, `hover_line`, and the `contrast` module.

## Security

See `docs/security/kx-theme.md`. The tokens are compiled in, the crate never logs, and its only outside input is the system look, where every value is a valid colour or flag.

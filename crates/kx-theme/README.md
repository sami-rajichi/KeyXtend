# kx-theme

The theme tokens and the look: three themes (Native Adaptive, ET66, Modern Dolch), each light and soft dark, the sizes, motion and icons they share, contrast checks and the high-contrast palette. It has no Qt and is not a module (it has no manifest, services or settings of its own), so its tests run on every system and the spike can use it. The `ui` module in `kx-ui` calls it (ADR-0016).

## Tokens

The four files in `tokens/` are compiled in with `include_str!`, never read from disk, so the uiAccess app reads nothing it draws from a file (ADR-0016). Values come from `design/keyboard-style-lab.html`, and each comment names the mock-up rule and line; where the P1 test build and the mock-up differ, the test build's tokens are the owner's reference.

| File | What it holds |
|---|---|
| `themes.toml` | Each theme's shape and type (radii, gap, legend sizes, fonts and their metrics, the font for marks on a dotted circle (`mark_fonts`), cap build (a sculpted cap also says how far a label may spill onto its skirt), D-pad colours, whether it can take the system accent) and its `light` and `dark` palettes; the `[common]` colours every theme shares. |
| `shape.toml` | Sizes all themes share, at size 1: the top bar, chips, legends, LED, the named-key labels, which shrink to fit their cap (`label_pad_px`, `label_min_px`), the language key (Carousel, `[lang.cycle]`, `[lang.list]`), D-pad, press, tooltips, the bubble and corner, hairlines, dimmed keys (`[dim]`), the high-contrast hover outline (`[hc_outline]`), the plate's share over a system backdrop (`[backdrop]`), the notice banner, the tray menu and the tray icon's drawing (`[tray]`), the language list and the later windows (pill, caption, snip, panel, hold ring). |
| `motion.toml` | Each animation's length and curve, its amounts, and the hold ring's curves; reduced motion keeps the curves with every length 0, except the ring's waves, which show the hold time. |
| `icons.toml` | The icon each part shows, as a file name in `assets/icons/lucide/` (Lucide) or `assets/icons/` (our own drawings): named keys by board id, the top bar (the theme button shows `palette`), the D-pad, the language key designs and the list's tick, paged lists' page buttons (`[page]`), the notice banner's icon, and the tray icon's glyph and menu items (`[tray]`). |

`Themes::shipped()` loads and checks all four; an error names the file and the token. Unknown or missing tokens, weights outside 100–900, shares outside 0–1, a curve that runs back in time, a burst that leaves the ring window, an accent on sculpted caps and an icon name that is not a plain file name are refused.

A palette's `legend_2_plate` (secondary text and icons on the plate) takes its `legend_2` unless the palette writes its own, which it does only where the two differ.

Dimmed keys (keys that come in a later version) have no colours of their own: every palette's `dim` and `dim_ink`, and Dolch's skirt `dim`, are the side key faded over the plate at `[dim] share` (the mock-up's disabled opacity). A palette file may not set them.

## The look

`look_view(themes, choice, sys, row_px)` gives a face everything it draws, as `LookView` (serialized to JSON for QML):

- `LookChoice` is what the user chose: the theme id, `ModeChoice` (light, dark or auto), `follow_accent`, `reduced_motion` and `min_contrast`, all from the `[ui]` settings.
- `sys` is the system look from the `kx-platform` port (`SysLook`): dark mode, the accent pair, the seven high-contrast colours and animations off.
- **Mode:** `mode_for` shows the chosen mode, or the system's with `auto`.
- **Accent:** only a theme with `accent = true` (Native) takes it, and only when `follow_accent` is on. Enter, on, locked, the LED, the ring and the badge take the accent shade for the mode; an ink below `min_contrast` switches to the main legend, then to black or white, so every legend reaches at least WCAG AA and a higher `min_contrast` is best effort. `with_accent` keeps the accent only when every pair the view draws still passes `drawn` on the keyboard's rows; otherwise, or with no rows to measure (`look_view` without `row_px`), the theme keeps its own colours.
- **High contrast:** every colour becomes one of the seven system colours, caps are flat (`LookView.look` has `Cap::Flat`, since a gradient's tints are not system colours), keys get a `[line] px` hairline edge, dimmed keys show `GrayText` on `ButtonFace`, and `hover_line` is `Highlight` (drawn `[hc_outline] px` wide) instead of a tint, which is not a system colour. Each drawn pair is one Windows makes readable, such as ButtonText on ButtonFace, so a chip's icon takes the chip's text colour. It wins over the accent.
- **Motion:** every length is 0 with the reduced-motion setting or the system's animations off.

## Contrast

`contrast` holds the WCAG 2.x maths and the standard's minimums: `WCAG_AA` (4.5:1 for text), `WCAG_SHAPES` (3:1 for icons and marks) and `MIN_CONTRAST` (`WCAG_AA..=21`), the only range the `[ui] min_contrast` check may accept, so the setting's floor is this constant and not a second copy.

`drawn` checks the colours the keyboard really draws, blended as QML blends them, so every pair is opaque and its ratio is what shows:

- each legend where it sits on its cap, at rest, under the hover tint and under the press shade. The place comes from the keyboard's row (`row_px`, which the caller passes: the hub gives the face's board row, and kx-theme's own tests a fixture) less the theme's gap, and the theme's font metrics (`font_metrics`, which a Quick test checks against the font). Each legend's capitals, from baseline to cap height, are measured against the gradient there, plus the skirt where they leave a sculpted top. A named key's label needs 4.5:1 and its icon 3:1;
- the language key's faded marks (Cycle's dots and globe, List's chevron) at 3:1 where they sit, and the D-pad's icons. Since an unlit dot keeps 3:1, the lit one stands out by size (`dot_on_px`);
- the top bar's texts and icons, the lit size letter (text, in `pop_ink`), its mark and the move ring (shapes, in `ring`) on the plate, chips, and the same plate see-through over a system backdrop of its mode (`WINDOWS`);
- text on the panels, their rows and buttons, the list's ring and the badge, also on a hovered row, the tray menu's faded icons and the bubble's icon.

`drawn::low` names every pair below its minimum, and the shipped-theme tests keep it empty in all six palettes and in high contrast; a kx-ui test runs it on the board's own row. Dimmed keys are left out, since WCAG sets no minimum for inactive parts (1.4.3). The colours nudged to pass are marked in `themes.toml` ("owner 2026-09-30" for flat pairs, "2026-10-02" for drawn ones).

**The see-through plate** (theme review M1, accepted): over a system backdrop the plate keeps 88 % of its colour (`[backdrop] plate`), and the check measures it over Windows' plain window colour in the keyboard's mode (`WINDOWS`), so Acrylic, or Mica over a dark wallpaper, may show a little lighter or darker. The backdrop is off by default, and its full check comes with the Settings window that turns it on (P13).

**A known limit** (owner, 2026-10-02): the check measures each legend over the Latin capital height, but accents, brackets and tall or descending Arabic letters reach a little beyond it, where a nudged token can read about 4.4:1. The owner checks those keys by eye.

**The one exception** (owner, 2026-10-02): the Carousel's side names keep the mock-up's 45 % fade (`[lang] side`) as decoration, below 4.5:1; the current layout's name beside them stays opaque. `drawn::EXCEPTION` names it, and a test fails as soon as any other pair falls below its minimum.

## Assets

`assets` names where the bundled fonts and icons sit in the repo's `assets/` folder (`ICON_DIRS`, `FONT_DIR`) and finds them (`icon_file`, `font_files`). `bundle` lists the files the app holds, under the names they take there (`ICONS_AT`, one folder of icons). kx-ui's build script uses it to compile them in, and the asset tests use it to check them, so the layout is written once. The app never reads these folders.

## QML look fixtures

`tests/fixtures.rs` writes the look JSON the Qt Quick tests read into `fixtures/` (one per theme and mode, plus Native light with reduced motion and in high contrast), as `.pragma library` files with `var lk`. A token change fails the test until it runs with `KX_BLESS=1`, and a file no look writes fails it too, so QML tests never read a stale token.

## Public API

`Themes` (`shipped`, `find`), `ThemeError`, `Theme`, `Look`, `Cap`, `Gradient`, `DpadRefs`, `Dpad`, `Palette`, `Skirt`, `Shadow`, `Rgba` (`parse`, `over`, `on`, `share`, `CLEAR`, `BLACK`, `WHITE`), `Common`, `Shape`, `Motions`, `Motion`, `Timed`, `Amounts`, `MOVES`, `RingMotion`, `RingAmounts`, `Icons` (`names`) with `BarIcons`, `DpadIcons`, `LangIcons`, `PageIcons`, `NoticeIcons` and `TrayIcons`, `Mode`, `ModeChoice`, `mode_for`, `LookChoice`, `LookView`, `look_view`, `reach`, `accent_fill`, `with_accent`, `contrast_palette`, `contrast_common`, `hover_line`; the `contrast` module; the `drawn` module (`Drawn`, `drawn`, `low`, `EXCEPTION`, `WINDOWS`); and the `assets` module (`Bundle`, `Bundled`, `bundle`, `icon_file`, `font_files`, `is_font`, `ICON_DIRS`, `ICON_EXT`, `ICONS_AT`, `FONT_DIR`, `FONT_EXTS`, `SYSTEM_FONTS`, `MARK_CIRCLE`, `MARKS`).

## Security

See `docs/security/kx-theme.md`. The tokens are compiled in, the crate never logs, and its only outside input is the system look, where every value is a valid colour or flag.

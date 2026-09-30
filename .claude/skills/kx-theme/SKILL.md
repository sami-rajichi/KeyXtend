---
name: kx-theme
description: Use when adding or changing a KeyXtend keyboard theme or any colour, font, spacing or motion token - Native Adaptive, ET66, Modern Dolch, each with light and soft-dark variants. Keeps themes consistent, readable and faithful to their real references.
---

# Theme work

- Tokens live in four files in `crates/kx-theme/tokens/`, compiled in and checked by the `kx-theme` crate (ADR-0016):
  - `themes.toml`: each theme's colours for light and dark, fonts and shape;
  - `shape.toml`: sizes all themes share;
  - `motion.toml`: each animation's length and curve, plus the reduced set;
  - `icons.toml`: the icon each part shows (Lucide files in `assets/icons/lucide/`).
- The reference values are in `design/keyboard-style-lab.html`, in the CSS blocks `[data-style=...][data-mode=...]`. Cite the mock-up line in the token's comment.
- QML reads tokens only through the look object; it never writes a colour, size or time itself.
- After a token change, run `cargo test -p kx-theme` with `KX_BLESS=1` to rewrite the QML look fixtures.

## Rules
1. **Every theme has light and soft dark.** Soft dark means:
   - no pure black: plates around #2A2C2E–#2C2F33, keys #3A3D42–#4A4D4F;
   - off-white legends (#E3E5E8 / #EDE8DF / #EDEAE4);
   - accents a little desaturated (Material dark-theme guidance; GitHub Dark dimmed).
2. **Contrast:**
   - legends ≥ 4.5:1 against their key colour, secondary legends too (target 7:1);
   - secondary text and icons on the plate use `legend_2_plate`, which equals `legend_2` unless that misses 4.5:1 on the plate;
   - shapes that carry meaning (the hold ring, icons) ≥ 3:1 against a plain window of their mode (WCAG 1.4.11);
   - the theme tests compute these; never change an owner-approved colour without asking;
   - there are no exceptions: every legend pair reaches 4.5:1, and the owner's nudges to reach it (2026-09-30) are marked in `themes.toml`;
   - where the P1 test build and the mock-up differ, the test build's tokens are the owner's reference.
3. **Colour means function** (ET66 rule): modifiers, Enter, danger keys (Esc/Backspace) and "on/locked" states each have a distinct token, in every theme.
4. **Fonts:**
   - Native: the system UI font plus Noto Sans Arabic;
   - ET66: IBM Plex Sans + IBM Plex Sans Arabic;
   - Dolch: Rubik (Latin) with Noto Sans Arabic.
   - All are OFL; bundle them with the licence (`kx-licence-check`).
   - Arabic legends are about 1.1× larger than Latin ones.
5. **Icons:** Lucide (ISC) only, platform-specific where it matters (Windows logo vs ⌘ vs Super). No emoji as icons.
6. **Scale:** every size is multiplied by the keyboard's scale. Panels use the panel scale, which never goes below 0.8.
7. **Motion:** reduced motion comes from the keyboard setting or the OS switch. Every length drops to 0, except the hold ring's waves, which carry the hold time.
8. **Character:** panels, pills and the Settings window carry the theme as strongly as the keyboard (gold, shadows, insets), not a plain default look.
9. **Review:** pictures of all 6 variants (3 themes × light/dark) at S and L, including the Arabic layout, attached to the PR.

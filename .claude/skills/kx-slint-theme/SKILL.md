---
name: kx-slint-theme
description: Use when adding or changing a KeyXtend keyboard theme or any colour, font or spacing token - Native Adaptive, ET66, Modern Dolch, each with light and soft-dark variants. Keeps themes consistent, readable and faithful to their real references.
---

# Theme work

- Tokens live in `crates/kx-ui/ui/theme.slint` as a `global Theme` with one struct per theme and mode.
- The reference values are in `design/keyboard-style-lab.html`, in the CSS blocks `[data-style=...][data-mode=...]`.

## Rules
1. **Every theme has light and soft dark.** Soft dark means:
   - no pure black: plates around #2A2C2E–#2C2F33, keys #3A3D42–#4A4D4F;
   - off-white legends (#E3E5E8 / #EDE8DF / #EDEAE4);
   - accents a little desaturated (Material dark-theme guidance; GitHub Dark dimmed).
2. **Contrast:** legends ≥ 4.5:1 against their key colour (target 7:1). Secondary legends ≥ 3:1. Check with a contrast tool and write the ratios in the PR.
3. **Colour means function** (ET66 rule): modifiers, Enter, danger keys (Esc/Backspace) and "on/locked" states each have a distinct token, in every theme.
4. **Fonts:**
   - Native: the system UI font plus Noto Sans Arabic;
   - ET66: IBM Plex Sans + IBM Plex Sans Arabic;
   - Dolch: Rubik.
   - All are OFL; bundle them with the licence (`kx-licence-check`).
   - Arabic legends are about 1.1× larger than Latin ones.
5. **Icons:** Lucide (ISC) only, platform-specific where it matters (Windows logo vs ⌘ vs Super). No emoji as icons.
6. **Scale:** every size is multiplied by the global scale. Panels use the panel scale, which never goes below 0.8.
7. **Motion:** honour the OS reduced-motion setting through the platform adapter. The hold ring still animates, because it carries meaning, but nothing else does.
8. **Review:** screenshots of all 6 variants (3 themes × light/dark) at S and L, including the Arabic layout, attached to the PR.

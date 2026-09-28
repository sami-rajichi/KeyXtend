# P1 stage 2: looks, speed and accessibility (G7–G13)

## Goal
Show that the Qt face can draw the designed keyboard and run it fast and accessibly: roadmap Phase 1 gates G7–G13, spec §3.1–3.5 and §10.

The owner's decisions (2026-09-28):
- all three themes, each light and dark, and the mock-up's animations, checked before ADR-0013;
- **Qt first, Slint as the fallback.** Under ADR-0011, Qt wins whenever it passes every gate, so Slint is tested again only if Qt fails a must-pass gate.

## Scope
- **In (Qt face):**
  - One full test keyboard built from the mock-up (`design/keyboard-style-lab.html`):
    - the top bar and chips;
    - the main block, the side block and the D-pad;
    - the Shift, AltGr and Caps keys, the language key and the Right-click toggle.
  - Top bar controls (owner):
    - a close button that quits with one click;
    - − and + size steps, and S/M/L;
    - a corner handle to resize by click, move, click (no drag, ADR-0008).
  - The keyboard starts small, at S.
  - A theme bar: Native Adaptive, ET66 and Modern Dolch, each light, dark or auto, one click each.
  - Motion: key press, hold ring with 6-dot burst, panel pop-in, LED pulse, language swap, D-pad Stop pop.
  - One mirrored Arabic panel: a 5-row paged list and an Arabic text field.
  - The pointer ring overlay, and Native's frosted plate tries.
  - Live following of the system accent, dark mode and high contrast.
  - Measurements in normal and weak mode.
- **Out:**
  - The Slint face, unless Qt fails a must-pass gate.
  - Real modules, sounds, the Settings window.
  - Stages 3 (soak) and 4 (macOS/Linux).
- **Kept as-is:** Qt 6.11.2 and Slint `=1.18.1`. Qt 6.12 LTS (due 30 Sep 2026) is noted in ADR-0013 but not installed.

## Design notes
- **Research first:** `docs/research/2026-09-28-slint-vs-qt.md`. Stage 2 checks Qt's side of each risk:
  - effects;
  - Mica;
  - Arabic editing and mirroring;
  - high contrast;
  - memory and size.
- **One source for the look.** New `spike/themes.toml`:
  - the six palettes from the mock-up (lines 105–142);
  - per-theme radius, gap, font and legend weight;
  - key and window shadows as layer lists;
  - `[motion]` durations and easings from the mock-up's transitions and keyframes.
  - The face reads it at start and on each theme switch; nothing is copied into `.qml`.
- **Core (toolkit-free, so a Slint fallback could reuse it):**
  - `theme.rs`: loads and checks the tokens, and gives a flat palette for a theme and mode.
  - `kbgeom.rs`: the 64-column grid, the side block and the D-pad as rects, scaled by one size factor.
  - `sysui.rs`: the accent colour (UISettings), dark mode, high contrast and a change notice (WM_SETTINGCHANGE). Qt's own `colorScheme`, `Accent` and `contrastPreference` are tried too, and the difference is a finding.
  - `backdrop.rs`: the frosted-plate tries through DWM: Acrylic or Mica backdrop, with a solid fallback.
  - `sizer.rs`: the size steps and presets, and the click-move-click resize that turns pointer travel into a clamped size.
- **Qt face:**
  - `qt-kb/qml/*.qml` components draw from the palette only.
  - Every mock-up effect is marked native, faked or impossible: backdrop blur, inset shadows, glows, the rotated D-pad clip, the Dolch skirt, the ET66 gradient flip.
- **Weak mode:**
  - the face runs in `weak.rs`'s job: 2 cores, 30 % CPU, 4 GB;
  - it uses Qt's `software` backend, which lacks shader effects, so the look in weak mode is recorded per theme;
  - it is measured on the default D3D11 backend too.
- **New settings:**
  - `spike.toml [look]`: theme, mode, `tip_ms = 500`, `reduced_motion = false`, the theme-bar labels;
  - `spike.toml [size]`: `presets = [1.00, 1.25, 1.50]`, `step = 0.05`, `min = 0.80`, `max = 1.80`, start at S;
  - `harness.toml [g7]`–`[g13]`: counts, waits and thresholds.
- **Downloads (a yes each, SHA-256, `kx-licence-check`, `D:\dev\assets`):**
  - fonts, all OFL: IBM Plex Sans + Arabic, Rubik, Noto Sans + Arabic;
  - Lucide icons, ISC: only the SVGs the mock-up uses.

## Tasks
**Part A: core (no windows)**
1. **Tokens.** *Files:* `spike/themes.toml`, `core/src/theme.rs`.
   - *Test first:* every palette has every token, and each legend/key pair reaches 4.5:1.
   - *Implement:* load, check, flatten.
   - *Verify:* `cargo test -p spike-core theme`.
2. **Geometry and size.** *Files:* `core/src/kbgeom.rs`, `core/src/sizer.rs`, `spike.toml [layout]`, `[size]`.
   - *Test first:* each row spans 64 columns; keys stay inside the plate at 0.80× and 1.80×; steps and the resize clamp to the limits.
3. **System look.** *Files:* `core/src/sysui.rs`, `core/src/backdrop.rs`.
   - *Test first:* registry and SPI values map to light/dark/high-contrast, and a refused backdrop falls back to solid.
4. **Installs**, after a yes for each: fonts and icons, with `THIRD_PARTY.md` rows.

**Part B: the look** (then the early look check)
5. **Full Qt keyboard.** *Files:* `qt-kb/qml/`, `qt-kb/src/`.
   - All key types, legends (primary, secondary, AltGr, Arabic with `لا` and harakat on ◌), LED, D-pad, chips.
   - Top bar with close, size and the corner handle, and the theme bar. Closing also ends the voice worker.
6. **Look check (owner).**
   - The harness screenshots the face in 3 themes × light/dark, cropped to our window.
   - The built-in browser captures the mock-up at the same size.
   - One local side-by-side page is shown in the browser pane.
   - A pop-up asks per theme whether it matches, and what to fix.

**Look-check changes (owner, 2026-09-28)**
- Native light uses the mock-up's colours (plate #E9EBEB, Enter #005FB8); following the Windows accent is an option, off.
- No labels under icons: every icon button shows a hover tooltip, and keys get a hover tint.
- Caps shows "Caps"; − minimises to a bubble in a bottom corner of the screen; the size buttons use lens icons; the fade is dropped.
- Esc no longer undoes a resize: Esc belongs to the app, and Settings will reset the size.
- Shadows are separate click-through windows, so the space around the keyboard never blocks a click.
- The Copy pill, voice caption and snip overlay take each theme's look, light and dark.

**Part C: motion, Arabic, overlay**
7. **Motion.** All mock-up animations timed from `[motion]`, and a reduced-motion switch.
8. **G7 Arabic panel.** *Files:* `qt-kb/qml/`, `harness/src/g7.rs`.
   - An RTL panel (`LayoutMirroring`), a paged list and an Arabic text field.
   - *Verify:* the harness types an Arabic word with harakat and moves the caret with ←/→/Home/End; UIA reads back the caret, and the owner confirms by eye.
   - *Design (2026-09-28):* the Settings side key (`action = "panel"`) opens it as a stand-in for Settings, the one window that takes focus.
     - Core: `pager.rs` pages a list (edge case 9); `panelcfg.rs` checks `spike.toml [panel]` (Arabic texts, 5 rows, `{n}`/`{count}` page text); `Tapped::Panel`.
     - Look: `shape.toml [panel]` from the mock-up `.pop`, `.pop-h`, `.lrow`, `.pg`, `.pgn`, `.lfoot`; a `panel` move (popIn .18s swift).
     - Qt: `Panel.qml` (with `PanelList.qml`, `PopButton.qml`) mirrored with `LayoutMirroring`, over a `PanelData` bridge; a caret reader in `core/src/uia/act.rs`.
     - Focus: the window guard spares only this window (`window::spare`); closing gives focus back to the app that had it.
     - Place: above the keyboard at its start edge (the right edge in Arabic), else below, else at the top, inside the work area (`popspot.rs`, `screen::pop_for`).
     - Left out: the mock-up's shrink-to-fit (a too-tall panel shrinks to size S); here it goes to the top of the screen instead.
     - `harness/src/g7.rs`: types `مَرْحَبًا`, then End, →, →, ←, Home, ←, End; the caret must land on letter boundaries, never between a letter and its haraka. It runs only while the panel is in front, and only with the owner's yes.
9. **G12 ring.** A click-through ring window at 60 Hz; 200 clicks underneath must all reach target-window.
   - *Design (2026-09-28):* the Qt face has no hold engine yet, so a "Ring test" choice in the test strip loops a hold at the pointer.
     - Settings: `spike.toml [hold]` (1500 ms, 500–3000, still 7 px, ring after 150 ms) becomes the one source; the harness's `[assist]` reads it. `[ring]` holds the test's window title, strip label, stats file and frame cap.
     - Look: `shape.toml [ring]` and `motion.toml` (curve `wave`, a `burst` move, ring amounts) from the mock-up `.ring`, `.burst`; the wave keeps the hold time under reduced motion, as the spec says.
     - Core: `holdcfg.rs` checks the settings; `ringstats.rs` sums frame gaps (fps, p99, longest) and writes them on stop.
     - Qt: `Ring.qml`, a click-through window that follows the pointer every frame (`FrameAnimation`), over a `RingData` bridge; `RingFace.qml` draws it, so a scratch QML test can check it without the bridge.
     - Stats: frame gaps come from the clock, not Qt's animation clock; the file lives under the user's local app-data folder (`folders.rs`), since a tools shell moves TEMP.
     - `harness/src/g12.rs`: turns the test on through UIA, then per click moves the pointer to a point in target-window, checks the ring covers it and that the point still belongs to target-window, and clicks; target-window's log must show all 200 presses there, and the stats must reach 60 Hz. It needs the owner's yes, since it moves the real mouse.
     - `[g12]` in harness.toml: 200 clicks on a 20 × 10 grid, at least 57 fps and a p99 gap of 34 ms or less; points our keyboard covers are left out.

**Part D: measurements** (harness `g8`–`g13`, installed uiAccess build)
10. **G8 budgets**, normal and weak mode, per theme:
    - idle memory, cold start, click to highlight on the next frame, click to character;
    - PresentMon frame times during every animation, idle CPU over 60 s;
    - staged size with fonts.
11. **G9 flags.** No-focus, tool and topmost bits read back after hide/show, a topmost toggle, a click-through toggle and a resize.
12. **G10 screen readers.**
    - A script checks every key's UIA name and Invoke.
    - The owner listens to a few keys with Narrator and NVDA.
13. **G11 DPI.**
    - The owner sets 150 % and 200 % in Settings and back to 100 %.
    - The harness checks each key's physical size ratio and legend edge sharpness.
14. **G13 live.**
    - Dark toggle, an accent change and high contrast on/off (the owner switches it) recolour the open face, panels included.
    - The frosted plate is tried on the inactive window.
15. **Effort record.** Qt's UI lines of code and the native/faked/impossible table; Slint's column comes from the research.

**Part E: close**
16. `kx-review` on every file, checks green, WORKLOG, commit. Checkpoint pop-up with the numbers and the look votes.

## Edge cases (each maps to a test)
1. A palette missing a token is refused, naming the theme and the token.
2. A legend/key pair under 4.5:1 is reported by name.
3. A layout row that does not span 64 columns is refused.
4. Size 0.80× and 1.80× keep every key inside the plate.
5. − at 0.80× and + at 1.80× do nothing; a resize past a limit stops at it.
6. A second click ends a move or resize, even on the keyboard itself or at a screen edge; there is no Esc undo (owner).
7. Auto mode follows the system; a change while a panel is open recolours the panel too.
8. High contrast replaces theme colours with system colours.
9. A 7-row list gives 2 pages, and the last one is padded to 5 rows.
10. Reduced motion zeroes every duration except the hold ring.
11. A failed weak-mode job is reported, never ignored.
12. A refused DWM backdrop gives a solid plate, once, without retry loops.
13. Clicks under the ring always reach the window below.
14. Close quits the face and its voice worker, even with a panel open.

## Security
- **Capabilities:** read-only registry and SPI reads, and DWM attributes on our own windows. There is no network in the face.
- **Data:** screenshots show only our window and the mock-up, cropped to their rects, and are deleted after the look check.
- **Threats to test:**
  - no new window takes focus (theme bar, panel, ring, resize);
  - the ring never swallows a click;
  - downloaded assets match their SHA-256.

## Manual Windows check (owner)
- Close the keyboard with its X; resize it with −/+, S/M/L and the corner handle.
- Look at the side-by-side page, and vote per theme and mode.
- Watch the animations and say whether they feel smooth.
- Change the display scale to 150 %, then 200 %, then back to 100 % when asked.
- Turn high contrast on and off once when asked.
- Listen to a few keys with Narrator and with NVDA.

## Done when
- G7–G13 print their numbers for the Qt face in normal and weak mode.
- The look check and the stage 2 checkpoint are answered.
- Screenshots are deleted, and the results are in `docs/NEXT-SESSION.md`.
- A failed must-pass gate reopens the Slint face for that gate.
- Stages 3 (soak) and 4 (macOS/Linux, with a no-focus panel check on GitHub's macOS machine) follow.

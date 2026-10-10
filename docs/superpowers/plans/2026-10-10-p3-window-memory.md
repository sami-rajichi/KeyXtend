# P3 window memory (ADR-0021)

## Goal
- Bring the keyboard's idle memory from about 215 MB (500 MB after a day) to about 100 MB, flat over time, per ADR-0021.
- Links: spec §10, ADR-0021, research `docs/research/2026-10-10-window-memory.md`, gate G8 in `docs/roadmap.md`.

## Scope
- **In:** one shared graphics device with the basic render loop; current-theme fonts only; no depth buffer; device-loss rebuild; G8 measuring commit, a warm-up and churn, with the new budget.
- **Out:**
  - slimmer keys and the "low memory" software mode (P13 ideas);
  - the 8-hour soak (P14);
  - weak mode's software drawing switch (G8 weak stays `partial`).

## Design notes
- **Crates and files:**
  - `crates/kx-ui`: new `cpp/device.h`, `src/bridges/device.rs`, `src/device.rs`; changes in `src/harden.rs` and `qml/KbWindow.qml`.
  - `crates/kx-theme`: new `tokens/font_files.toml`; changes in `src/assets.rs`.
  - `apps/keyxtend/src/qt.rs` and `run.rs`; `tools/kx-gates` (`proc.rs`, `g8*.rs`, `kx-gates-p3.toml`).
- **New interfaces:**
  - `kx_ui::Device`: creates the shared QRhi on the pinned adapter and has every `QQuickWindow` adopt it before its first frame.
  - `kx_ui::FontBook` (trait): loads and drops a theme's bundled font files. `QtFontBook` is the Qt one; a fake serves tests.
  - `DeviceLost`: the glue's report that the device is gone. Rust answers by reloading the QML over the same hub (ADR-0018 allows it: objects read the hub when made).
- **Qt variables:** `QSG_RENDER_LOOP=basic` and `QSG_NO_DEPTH_BUFFER=1`, set after `harden_qt` beside `QT_D3D_ADAPTER_INDEX`. Names and values live in one table in `src/harden.rs`.
- **New settings key:** `[ui] device_rebuilds = 3`, the most rebuilds in one run. Past it, the app goes back to Qt's per-window devices and logs it.
- **New token table:** `font_files.toml` maps each bundled family to its files. Native's Segoe UI is a system font, so it maps to none.
- **No new translation keys.** Logs name the adapter, never text.

## Tasks
1. **G8 reads commit and waits for a warm-up.**
   - *Files:* `tools/kx-gates/src/proc.rs`, `g8.rs`, `g8cfg.rs`, `g8parts.rs`, `kx-gates-p3.toml`.
   - *Test first:* the parts judge private working set ≤ `memory_mb` (100) and private commit ≤ `commit_mb` (140), each reported. A config without `commit_mb` is refused.
   - *Implement:* read `PrivateUsage` beside `PrivateWorkingSetSize`; sample after `warm_ms`; never trim.
   - *Verify:* `cargo test -p kx-gates` passes.
2. **G8 churn: memory stays flat while pop-ups come and go.**
   - *Files:* `tools/kx-gates/src/g8churn.rs` (new), `g8.rs`, `kx-gates-p3.toml`.
   - *Test first:* the churn verdict fails when growth exceeds `churn_growth_mb`, and passes on flat samples.
   - *Implement:* `churn_rounds` rounds of key hovers (tips), the language list and the top-bar theme cycle, sampling memory before and after.
   - *Verify:* `cargo test -p kx-gates` passes.
3. **The two Qt variables.**
   - *Files:* `crates/kx-ui/src/harden.rs`, `src/bridges/harden.rs`, `tests/harden/main.rs`, `apps/keyxtend/src/qt.rs`.
   - *Test first:* after `harden_qt` plus the pins, exactly the adapter, render-loop and depth variables are set, and an outside `QSG_RENDER_LOOP=threaded` is replaced.
   - *Implement:* the table and one setter call after `harden_qt`.
   - *Verify:* `cargo test -p kx-ui --test harden` passes.
4. **One shared device.**
   - *Files:* `crates/kx-ui/cpp/device.h`, `src/bridges/device.rs`, `src/device.rs`, `apps/keyxtend/src/qt.rs`, a new QML load test binary.
   - *Test first:* two windows, one made later by a `Loader`, report the same QRhi, and the render loop is basic.
   - *Implement:*
     - create the QRhi once after `QGuiApplication`, before the QML loads;
     - adopt it in an app-wide event filter on each window's surface creation;
     - delete it after the last window at quit.
   - *Verify:* `cargo test -p kx-ui` and `qmltestrunner` (0 failures, no QWARN).
5. **A failed device falls back.**
   - *Files:* `src/device.rs`, `apps/keyxtend/src/qt.rs`.
   - *Test first:* when the device cannot be made, start goes on with Qt's own devices and logs one warning.
   - *Implement:* the fallback path.
   - *Verify:* `cargo test -p kx-ui` passes.
6. **Fonts for the current theme only.**
   - *Files:* `crates/kx-theme/tokens/font_files.toml`, `src/assets.rs`, `crates/kx-ui/src/hub/*`, `qml/KbWindow.qml`, `cpp/fonts.h`, `src/bridges/fonts.rs`.
   - *Test first:*
     - each theme's families map to bundled files that exist, and every bundled file belongs to some theme;
     - a theme switch drops the last theme's files and loads the new ones (fake `FontBook`).
   - *Implement:*
     - load before the QML, and on each theme switch;
     - remove `FontLoader` and `fontsIn`; QML re-reads families on the hub's fonts signal.
   - *Verify:* `cargo test -p kx-theme -p kx-ui` and `qmltestrunner` pass.
7. **Device loss rebuilds the windows.**
   - *Files:* `cpp/device.h`, `src/device.rs`, `apps/keyxtend/src/qt.rs`, `crates/kx-ui/defaults.toml`, the settings schema.
   - *Test first:*
     - a debug-only trigger marks the device lost, and the app reloads the QML once over the same hub;
     - the guard runs again;
     - the keyboard's place and size stay;
     - past `device_rebuilds`, the app falls back.
   - *Implement:* check `isDeviceLost` after each frame, then reload with a new QRhi.
   - *Verify:* `cargo test --workspace` passes.
8. **Docs.**
   - *Files:* `ARCHITECTURE.md`, `crates/kx-ui/README.md`, `docs/security/kx-ui.md`, `docs/roadmap.md` (G8 text: 100 MB working set, 140 MB commit, churn).
   - *Implement:* one or two sentences each.
   - *Verify:* `cargo xtask tidy` passes.

## Edge cases
1. A window a `Loader` makes later (language list, tips) adopts the shared device before its first frame (task 4).
2. Showing and hiding pop-ups many times leaves memory flat (task 2, G8 churn).
3. Native → Dolch → ET66 → Native: only the current theme's files stay loaded, and Arabic draws in the right font each time (task 6).
4. A missing font file: the theme falls back to its next family and logs it (task 6).
5. The pinned adapter is gone: the device is made on Qt's default and logged (task 4).
6. The device cannot be made at all: Qt's per-window devices, as today (task 5).
7. The device is lost during a resize: after the rebuild, place and size are the face's (task 7).
8. Repeated losses stop at `device_rebuilds` (task 7).
9. An outside `QSG_*` variable is cleared and replaced by ours (task 3).
10. A display scale change after the switch: each window's swap chain follows (G11 re-run).

## Security
- **Capabilities:** none new. `unsafe` only in `kx-ui/src/bridges/`, each block with `// SAFETY:`.
- **Data:** no typed text. Fonts load only from the app's own bundle folder, never from a user path.
- **Threats to test:**
  - the hardening still clears every other Qt variable (task 3);
  - a device-loss loop cannot spin (task 7);
  - the QRhi is used only on the GUI thread (basic loop, task 4).

## Manual Windows check
1. Start from the shortcut. Task Manager shows about 100 MB, and it stays there after a few minutes of use.
2. Native, ET66 and Dolch, light and dark, look as before, with no missing depth effects.
3. Pop-in, hover and press motion feel as smooth as before.
4. Typing EN, FR and AR in Notepad.
5. Gates: G8 (PresentMon, admin prompt), G2, G9, G10, G11 at 100/150/200 %, G13.

## Done when
- The Definition of Done in `docs/roadmap.md` is met.
- G8 passes at ≤ 100 MB working set and ≤ 140 MB commit, with churn flat.
- No gate that passed before fails.

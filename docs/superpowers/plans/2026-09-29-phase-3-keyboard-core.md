# P3: keyboard core, "it types"

## Goal
Ship the first real KeyXtend app: the keyboard types English, French and Arabic without taking focus, in the Native, ET66 and Dolch themes (roadmap "Phase 3"; spec §3.1–§3.6, §4.1–§4.5).
Move the spike's keyboard, layout, window, theme and Qt parts into the product (`docs/spike-move-map.md`, P3; ADR-0014), and pass gates G8–G11, G13 and G26.

Owner's decisions (2026-09-29):
- The spike face stops building at the start of P3; its later-phase parts wait as source, and the installed test copy stays until the owner removes it.
- The side block shows in full; keys not built yet are dimmed and say "Coming in a later version".
- × hides to a tray icon; a left click on it opens a small themed menu (Show keyboard, Quit). After Quit, the Start-menu shortcut starts it again.
- Notices show in a banner above the keyboard with Try again or OK.
- A small theme button in the top bar cycles the three themes (a trial; Settings takes the choice in P13). The top bar keeps its size.
- All three language-key designs are built: Carousel (default), Cycle and List.
- The UI language follows Windows (EN, FR or AR, else EN), with a setting.
- The keyboard always starts as the full keyboard at its saved place, size and theme; off-screen places come back to the bottom centre.
- Fonts and icons live in the repo; translations use Fluent (`.ftl`).
- `kx-crypto` moves to P8 and G26's Japanese IME part to F16.

## Scope
- **In:** the crates below, the app, staging and dev install, the Qt CI job, gates G1–G4, G8–G11, G13, G25 (language), G26, ADR-0016 and ADR-0017, and the P3 notes in `docs/items/P3.md`.
- **Out:** modifier + mouse click (P4), the shortcuts layer and Power (P6), suggestion chips (P7), Settings window (P13), the app icon's final design and autostart (P14).

## Design notes
- **Crates:** `kx-platform-windows`, `kx-mod-layouts`, `kx-mod-keyboard`, `kx-theme` (tokens, look and contrast; no Qt, so tests run everywhere and the spike can use it), `kx-ui` (QML, bridges, translations, the `ui` module), `apps/keyxtend`.
- **Every move** deletes the spike copy in the same commit (`git mv` where the file survives) and points `spike/core` and `tools/kx-gates` users at the new crate; a re-export in `spike/core/src/lib.rs` is allowed.
- **Module ports** (`kx-module-api`, pure types):
  - `InputInjector` (InjectInput): tap, chord and text; text goes as Unicode or, for apps listed in `key_apps`, as key presses in the target's layout; typed text is `Redacted`.
  - `LayoutSource` (InjectInput, since it asks the app in front to switch): installed layouts, the layout in front, `KeyChars` per scan code, language facts, Caps state, switch request.
  - `Layouts` (provided by `kx-mod-layouts`): the current `LayoutTable`, the ordered list, step and pick; event `LayoutChanged`.
  - `KeyboardFace` (provided by `kx-mod-keyboard`): board and state as JSON for QML, and typed `FaceAction`s (tap, size, move, language pick, panel keys).
- **UI ports** (`kx-platform`): `WindowGuard`, `Screens`, `SystemLook`, `Backdrop`, `Instance` (one running copy; `show` and `quit` commands).
- **Kernel additions:** `Kernel::load` then `start`, so the log opens first; a live `SettingsHandle` for values a module writes itself (size, place); events may name a capability that subscribe, intercept and publish check.
- **The `ui` module** lives in `kx-ui`: it owns the look and window settings, takes `KeyboardFace`, shows notices and hands Try again to the app.
- **Data files:** `kx-mod-keyboard/board.toml` (rows, widths, scan codes, side grid, key roles); `kx-theme/tokens/{themes,shape,motion,icons}.toml`, all embedded, never read from disk.
- **Assets:** `assets/fonts/<family>/` and `assets/icons/lucide/`, each with its licence, compiled into the binary as Qt resources.
- **Settings `[keyboard]` v1:** `modifiers = "release"` ("release" after the next key, or "stay"), `lang_key = "carousel"`, `size = 1.0`, `presets = [1.0, 1.25, 1.5]`, `size_step = 0.05`, `size_min = 0.8`, `size_max = 1.8`, `relabel_ms = 500`, `key_apps = []`, `x = 0`, `y = 0`, `placed = false`, `bubble_corner = "right"` (place and bubble live with size, so `place.rs` does the placement maths).
- **Settings `[layouts]` v1:** `short = { ar = "ع" }`.
- **Settings `[ui]` v1:** `theme = "native"`, `mode = "auto"`, `follow_accent = false`, `reduced_motion = false`, `backdrop = "none"`, `language = "auto"`, `tip_ms = 500`, `poll_ms = 16`, `guard_delay_ms = 300`, `guard_tries = 10`, `page_rows = 5`, `min_contrast = 4.5`.
- **New theme tokens:** `notice` and `menu` shapes, `dim` (dimmed keys), `hc_outline` (hover in high contrast), `lang` cycle dots, the list panel, and the `palette` icon for the theme button.
- **Translation keys** (EN, FR, AR in `kx-ui/i18n/`): key and button names, side keys and "coming later", tray menu, language-key names, theme names, the six P2 notices plus `notice.settings-locked`, with a list of keys and "and N more".

## Tasks

### Task 1: Retire the spike face build
*Files:* `spike/qt-kb/{Cargo.toml,Cargo.lock,.gitignore}` (deleted), `docs/spike-move-map.md`, `docs/adr/0016-product-app-replaces-the-spike-face.md`, `docs/adr/README.md`.
*Test first:* none; the spike core workspace still builds and its tests pass. *Implement:* stop building `qt-kb` and keep its files as source; ADR-0016 records the retirement, `kx-theme`, the UI ports and assets in the repo. *Verify:* spike `cargo test`; root tests unchanged.

### Task 2: Licences and assets
*Files:* `assets/**`, `THIRD_PARTY.md`, `deny.toml` if needed.
*Test first:* none; `cargo deny check` is the test. *Implement:* `kx-licence-check` for `windows` (now shipped), Qt 6.11.2 modules, cxx-qt, Fluent and their dependencies, fonts and icons; copy fonts and icons into git (the `palette` icon after the owner's yes). *Verify:* `cargo deny check`; `cargo xtask licences`.

### Task 3: Kernel carry-overs
*Files:* `kx-kernel/src/{kernel,boot,control}.rs`, `kx-module-api/src/{cx,settings}.rs`, `kx-settings/src/file.rs`.
*Test first:* the log opens before modules start; a module's own write saves without a restart; `set_setting` restarts a failed module; a locked settings file runs on defaults and never saves.
*Implement:* `load`/`start`, `SettingsHandle`, the lock rule. *Verify:* `cargo test -p kx-kernel -p kx-settings`.

### Task 4: Event capability gate
*Files:* `kx-module-api/src/event.rs`, `kx-kernel/src/{bus,host}.rs`.
*Test first:* a module without the event's capability gets `NotGranted` on subscribe, intercept and publish. *Implement:* `Event::CAPABILITY`, default none. *Verify:* same crates.

### Task 5: Ports and fakes
*Files:* `kx-module-api/src/ports/{input,layout,face}.rs`, `kx-platform/src/ui.rs`, `kx-platform-fake/src/{input,layout,ui}.rs`.
*Test first:* the fake injector records strokes in order; the fake layout source serves golden tables. *Implement:* the ports above. *Verify:* `cargo test -p kx-module-api -p kx-platform-fake`.

### Task 6: `kx-platform-windows` base
*Files:* `src/{lib,clock,dirs,com,uiaccess}.rs` (moved); `tools/kx-target-window`.
*Test first:* QPC ticks convert to µs; `AppDirs` uses the known folder. *Implement:* `WindowsPlatform`; the target window drops `spike-core`. *Verify:* crate and tool tests.

### Task 7: Input injection
*Files:* `src/{inject,keys}.rs` (from spike and `kx-gates/keys.rs`).
*Test first:* strokes carry the tag, extended keys get their flag, surrogate pairs stay whole, held keys release after a failed send. *Implement:* `InputInjector`, the key-press route via `VkKeyScanEx`. *Verify:* crate tests.

### Task 8: Layout reading
*Files:* `src/{layouts,langinfo}.rs`; `tools/kx-gates/src/capture.rs`; `tools/kx-target-window` (`--layout`).
*Test first:* pure parsing of layout ids and facts; no 32-layout cap. *Implement:* `LayoutSource`; `kx-gates capture-layouts` writes the EN, FR and AR golden tables; the target window can load a layout for itself. *Verify:* crate tests; capture runs on this PC.

### Task 9: Window and system parts
*Files:* `src/{window,screen,sysui,watch,backdrop,instance}.rs`.
*Test first:* the kept ex-styles, DPI and work-area maths, colour parsing, command parsing. *Implement:* the UI ports; the guard unhooks on `WM_NCDESTROY`. *Verify:* crate tests.

### Task 10: `kx-mod-layouts`
*Files:* `crates/kx-mod-layouts/**` (via `kx-new-module`).
*Test first (proptest):* AZERTY AltGr, Arabic shifted harakat and لا from golden tables; a layout added or removed keeps the list and the current pick sane. *Implement:* tables, short names, order, step and pick. *Verify:* `cargo test -p kx-mod-layouts`.

### Task 11: Keyboard geometry
*Files:* `kx-mod-keyboard/src/{geom,spec,place,sizer}.rs`, `board.toml`, `defaults.toml`.
*Test first:* every row spans the grid; sizes clamp and step; an off-screen place returns to the bottom centre. *Implement:* moved from `kbgeom`, `place`, `sizer`. *Verify:* crate tests.

### Task 12: Typing
*Files:* `src/{latch,legend,plan,run}.rs`.
*Test first (proptest):* "release" drops latches after the next key; "stay" keeps them until clicked; dead keys, Caps, AltGr and `key_apps` pick the right route. *Implement:* moved from `latch`, `legend`, `kbctl`. *Verify:* crate tests.

### Task 13: Language key and panels
*Files:* `src/{langkey,panel,face,view}.rs`, `tests/golden/*.json`.
*Test first:* Carousel, Cycle and List move as the mock-up does; while the List is open, arrows, Enter and Esc drive it; view JSON matches golden files. *Implement:* `KeyboardFace`. *Verify:* crate tests.

### Task 14: `kx-theme`
*Files:* `crates/kx-theme/**`, `tokens/*.toml` (moved), `src/contrast.rs`.
*Test first:* the moved 52 tests; legend contrast ≥ 4.5:1 outside the owner's listed exceptions; new tokens load. *Implement:* loader, look view, high-contrast palette, and a test that writes the QML look fixtures. *Verify:* crate tests.

### Task 15: Translations
*Files:* `crates/kx-ui/{Cargo.toml,src/lib.rs,src/i18n.rs,i18n/{en,fr,ar}.ftl}` (the crate starts here, without Qt), ADR-0017.
*Test first:* every key exists in all three files; Arabic "and N more" takes the right plural form; "auto" picks the Windows language, else English. *Implement:* Fluent loading. *Verify:* `cargo test -p kx-ui`.

### Task 16: `kx-ui` Qt build and tidy
*Files:* `kx-ui/{build.rs,src/bridges/*,src/ui_module.rs}`, `xtask/src/tidy/*`.
*Test first:* tidy accepts `unsafe` only in `kx-ui/src/bridges/` (new rule F3); the `ui` settings check refuses bad values. *Implement:* the QML module `KeyXtend.Ui`, bridges, the `ui` module. *Verify:* `cargo test -p xtask -p kx-ui`.

### Task 17: Move the QML
*Files:* `kx-ui/qml/*.qml` (20 files from the spike).
*Test first:* Qt Quick Tests find keys by accessible name; only latches and Caps are checkable; dimmed keys announce "Coming in a later version". *Implement:* the move plus fixes: `Icon.qml` without Qt's private module, grayscale text, a hover outline in high contrast. *Verify:* `qmltestrunner` on `kx-ui/qmltest`.

### Task 18: New QML parts
*Files:* `Notice.qml`, `TrayMenu.qml`, `LangCycle.qml`, `LangList.qml`, the theme button in `TopBar.qml`.
*Test first:* the banner queues and counts; the menu offers Show and Quit; List pages five rows. *Implement:* as named. *Verify:* `qmltestrunner`.

### Task 19: The app
*Files:* `apps/keyxtend/{Cargo.toml,build.rs,src/*}`.
*Test first:* argument parsing (`--show`, `--quit`); a quiet panic hook logs only the place. *Implement:* one running copy, log first, kernel, modules, ports, UI; uiAccess manifest in release. *Verify:* the app starts and types into `kx-target-window`.

### Task 20: Staging and dev install
*Files:* `xtask/src/devtools/{stage,shortcut}.rs`, root `Cargo.toml` metadata; `spike/stage-lib.ps1`, `spike/qt-kb/stage.ps1` (deleted).
*Test first:* the stage check fails on a TLS or network-information plugin; the shortcut path stays in the user's Start menu. *Implement:* `cargo xtask stage` (Qt from `QMAKE`), and `dev-install` adds and removes "KeyXtend (dev)". *Verify:* `cargo test -p xtask`; stage size under 80 MB.

### Task 21: CI
*Files:* `.github/workflows/ci.yml`.
*Test first:* none; the workflow is the test. *Implement:* the Windows job installs Qt 6.11.2 (pinned action), builds everything and runs the Qt Quick Tests; Ubuntu skips `kx-ui` and `keyxtend`; a G26 job. *Verify:* checks green on the pull request.

### Task 22: Spike clean-up check
*Files:* `spike/**`, `spike/spike.toml`, `spike/core/src/config.rs`.
*Test first:* the spike and gate tests pass on the product crates. *Implement:* confirm no P3 file is left in the spike; drop the P3 settings sections and config structs. *Verify:* both workspaces' tests; tidy 0/0.

### Task 23: Gates on the product
*Files:* `tools/kx-gates/src/{g1..g4,g25,close}.rs`.
*Test first:* keys are found by UI Automation id, so the top bar no longer shifts clicks. *Implement:* aim at `keyxtend`; `close` sends `--quit`. *Verify:* gate parse tests.

### Task 24: New gates
*Files:* `tools/kx-gates/src/{g8,g9,g10,g11,g13,g26}.rs`.
*Test first:* each gate's pass rule on recorded samples. *Implement:* budgets and weak mode (G8 also times resizing, start-up and a bus publish), window flags, screen-reader walk, DPI record, live-follow record, other languages. *Verify:* gate tests.

### Task 25: Docs
ARCHITECTURE.md; spec §3.1, §3.5, §3.6, §4.3; roadmap; move map; README; CONTRIBUTING (Qt set-up); `docs/security/{kx-platform-windows,kx-mod-keyboard,kx-ui}.md`; `kx-review` gains a check for `tracing` in input code; the "waiting modules" note moves to P13.

## Edge cases (each has a test)
1. Modifiers plus AltGr on AZERTY; AltGr on a layout without AltGr acts as Right Alt.
2. Arabic shifted harakat shown on ◌ and typed bare; لا types two code points.
3. A layout added or removed while running.
4. An app listed in `key_apps` gets key presses in its own layout.
5. A saved place on a missing or mixed-DPI screen returns on-screen at the same physical size.
6. Moving the keyboard off-screen is refused at the screen edge.
7. A key click while a modifier is held in "Stay held" mode keeps it held.
8. Dead key then space, then an unrelated letter, then a language switch.
9. Only one layout installed: the language key stays still and says so.
10. A second launch shows the running keyboard; `--quit` ends it.
11. A locked settings file; a log that cannot be replaced (runs without a file log and says so once).
12. A UI language other than EN, FR or AR falls back to English.
13. High contrast on: system colours only, with a hover outline.

## Security
- **Capabilities:** `keyboard` and `layouts` hold InjectInput; `ui` holds none.
- **Data:** typed text is transient and always `Redacted`; settings hold places and choices, no secrets.
- **Threats to test:** our own injections looping (tag); an app sending our command window `quit` (same-session only, recorded); a QML, token or font file swapped on disk (all compiled in); a network or TLS plugin staged (stage check); typed text in a log (redaction test).
- **Record:** three security notes via `kx-security-check`.

## Manual Windows check
1. `cargo xtask stage`, then `dev-install`; start KeyXtend from the Start menu.
2. Type EN, FR (AltGr) and AR (harakat, لا) into Notepad, Chrome, an admin Terminal and a signed-in Word; G1–G4 on the product.
3. Modifiers in both modes, Caps, the three language keys, theme button, sun/moon, sizes, move, resize, bubble.
4. × to tray, tray menu Show and Quit, start again from the Start menu; a damaged settings file shows the banner.
5. G10 with Narrator and NVDA; G11 at 100, 150 and 200 % scale; G13 dark mode, accent and high contrast live, and the frosted-glass choice.
6. G8 on this PC and in weak mode.
7. Report in `docs/test-reports/2026-09-29-p3-keyboard-core.md`.

## Done when
- The Definition of Done in `docs/roadmap.md` is met, and every note in `docs/items/P3.md` is handled or moved to its item.
- The installed test copy of the spike face is the owner's to remove.

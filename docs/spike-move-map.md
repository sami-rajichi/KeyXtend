# Spike move map

Where each tested spike part goes, and in which phase (ADR-0014). Paths are under `spike/` on the left and the product on the right. A phase plan may refine a line; it records the change here.

## Rules for every move

- Fit the part to the architecture: a module behind ports, settings in the schema, texts in translation files, values in theme tokens.
- OS calls (Win32, `unsafe`) go to `kx-platform-windows` behind a port; the module keeps only the logic. Rows below name both halves.
- Keep its tests passing in the new crate, and meet the Definition of Done.
- **Move:** the commit deletes the spike copy; spike code that still needs the part depends on the new crate.
- **Model:** a shared spike foundation stays until its last spike user has moved; the product crate is built from its pattern.

## P2 — Kernel, settings and test tools

| Spike | Product | Kind |
|---|---|---|
| `slint-kb/`, `tauri-kb/`; their lines in `spike.toml` `[keyboard]`, `core/src/config.rs`, `harness/src/{usage,main}.rs` (help and parse tests); Slint-only `window::click_through` | Removed (kept in git history); `worker_dir` (now in `kx-gates.toml`) repointed to the Qt stage for `g22bench` | — |
| `core/src/config.rs` (load beside the exe, refuse unknown keys, check at load) | `kx-settings` loader | Model |
| `core/src/clock.rs` | `Clock` and `Mono` in `kx-module-api`, because modules may use only that crate; `Platform` in `kx-platform` hands out the clock; the QPC clock in `kx-platform-windows` (P3) | Model |
| `core/src/{pct,ringstats}.rs` | `kx-test-support`; `ringstats` takes its unit constants from `kx-module-api`, and P4 deleted it | Move |
| `core/src/targetlog.rs`, `target-window/`, `spike.toml` `[target]` | `tools/kx-target-window`, the CI test window (spec §12); it reads `kx-target-window.toml` | Move |
| `harness/` whole, `harness.toml` | `tools/kx-gates`, the Windows gate runner; it reads `kx-gates.toml` and keeps testing the spike face (since P3, its last staged copy) until each phase aims its gates at the product | Move |

After P2, the harness rows below name files in `tools/kx-gates`. Gate logs go to `target/gate-logs`, and the Qt stage the gates use is `target/spike-stage/qt-kb`, the spike face's last stage, which nothing rebuilds since P3; gates aimed at the product use `target/stage/keyxtend` from `cargo xtask stage`, and P4's driving gates use the gate-input test build in `target/gate-input`.

## P3 — Keyboard core

The `qt-kb` build stopped at the start of P3 (ADR-0016): its Cargo files are deleted and the rest of `qt-kb/` stays as source until each part moves. `core/` and `worker/` keep building.

Only gates `g1`–`g4` and `g25`'s language part are aimed at the product in P3; the other gates that drive the spike face run only against its last staged or installed copy until their phase aims them at the product.

| Spike | Product |
|---|---|
| `core/src/{kbctl,kbctl/*,latch,kbgeom,kbgeom/*,kbview,place,sizer,legend,status}.rs` | `kx-mod-keyboard`: `kbgeom` is `geom` and `spec` (the rows in `board.toml`), `sizer` moved and `place` (last place, bottom centre, per-screen DPI) is new, `latch` and `legend` moved whole, the typing half of `kbctl` is `plan` (each click's route) and `run` (`Typing`), and `kbview` is `view`, the `KeyboardFace`'s JSON (`face`). The status line ended and the guard retries are `kx-ui` `guard`. The spike's `kbctl` is deleted; its `place.rs` keeps the stage-1 block for the tools row (P9 to P12) |
| `core/src/{langinfo,langkey}.rs` | `kx-platform-windows` `langinfo` (locale and registry facts), `front` (the window with the keyboard and its layout) and `layouts::ask` (the switch request); `kx-mod-layouts` names a layout (`[layouts] short`, and `label` for the space bar's parts), and the key steps with `kx_module_api::ports::step_to`. `langinfo.rs` is deleted; `langkey.rs` keeps only `installed_for` and `layout_for` for G22 (P11), as P6 moved the gates' other layout helpers to `tools/kx-gates` `langs.rs` |
| `core/src/layout.rs` | `kx-platform-windows` `layouts` (the installed list, `ToUnicodeEx` reading, Caps Lock and the scan-code helpers); `Follow` in `kx-mod-layouts` (`Layouts::refresh`); the board in `kx-mod-keyboard`'s `board.toml` and key names in `kx-ui`'s translations. The spike keeps `front_layout` for voice (P11) |
| `core/src/{inject,window,screen,backdrop,folders,com,uiaccess}.rs`, the extended-key table in `tools/kx-gates` `keys.rs` | `kx-platform-windows`: `folders`, `com`, `uiaccess`, `inject`, the window guard and queries (`window`), work areas, DPI and the pointer (`screen`), and DWM backdrops (`backdrop`); the accent-policy blur ended. The solid plate is `kx-ui` `plate`, the bubble's corner maths `kx-ui` `geom`, and the bottom-centre and corner maths `kx-mod-keyboard` `place`. The spike keeps thin helpers over the product in `sendkeys.rs`, `window.rs` and `screen.rs`, each with its home in a row below (P9 to P13) |
| `core/src/{sysui,sysui/*}.rs` | Split, and the spike file is deleted. The Win32 half is `kx-platform-windows` `sysui` (dark apps, the accent pair, the high-contrast colours, animations off, the UI language) with its watcher `watch`; the pure half is `kx-theme` `mode`, `hc` (the high-contrast palette) and `accent`. The `ui` module reads and follows the look through the `SystemLook` port |
| `core/src/theme.rs`, `core/src/theme/{common,extras,look,motion,palette,ringmove,shape,tests}.rs` | `kx-theme` (tokens, look, contrast; no Qt, so not `kx-ui`, ADR-0016), with their tests: WCAG maths in `contrast` (with `WCAG_AA` and `MIN_CONTRAST`, the floor of `[ui] min_contrast`), `Rgba` in `rgba`, `Mode` in `mode`, the accent in `accent`, and `ringmove`, since `motion.toml [ring]` is a token table (P4 keeps the ring's QML and bridge). The loader reads compiled-in tokens (`Themes::shipped`) |
| `core/src/paint.rs` | `kx-mod-keyboard` `view::paint`, which names palette tokens as text, so it needs no `kx-theme`; its "every theme has these tokens" test is in `apps/keyxtend`, the only product crate that sees both. `kx-theme`'s typed `Palette` and `Skirt` require `rec` and `rec_ink`, so every shipped theme has them |
| `core/src/facecfg.rs` | `kx-mod-keyboard`: key roles, latches and Caps Lock in `board.toml`, the bubble corner in `[keyboard] bubble_corner`, and the top-bar buttons in the face. Names and labels are in `kx-ui`'s translations, icon names in `kx-theme` `icons.toml` (with `palette` for the theme button), and the assets are compiled in. The spike file is deleted with `spike.toml [keys]`, `[bar]` and `[assets]`; G7 reads the frozen face's Settings key name from `kx-gates.toml [g7] opener` |
| `core/src/lookcfg.rs` | The `[ui]` settings of the `ui` module in `kx-ui` (`settings.rs`, `defaults.toml`), where one `follow_accent` switch replaced `accent_themes` and `poll_ms` joined from `[size]`. `ModeChoice` and `LookChoice` (theme id, mode, `follow_accent`, `reduced_motion`, `min_contrast`) are `kx-theme`'s look input, `Backdrop` is `kx-platform`'s `BackdropKind`, and the plate's share over a backdrop is the `[backdrop] plate` token. The spike file and `[look]` are deleted |
| `core/src/lookview.rs` | `kx-theme` `view`, called by the `ui` module, not the keyboard: `look_view(themes, choice, sys, row_px)` over `kx-platform`'s `SysLook` and the board's row height from the face, with the icons and the high-contrast `hover_line`; the Win32 reading is in `kx-platform-windows` |
| `core/src/note.rs` | Stays as a Model for `fill.rs` (P9), `typer.rs` (P11) and the tools row (P9 to P12); the product shows such notes as kernel notices in the banner |
| `themes.toml`, `shape.toml`, `motion.toml` | `kx-theme` tokens in `crates/kx-theme/tokens/`, compiled in, with the new `icons.toml`, the P3 sizes (notice, menu, language list, Cycle and List keys, `[dim]`, `[hc_outline]`), the notice and menu moves, and `accent` per theme. The theme names are `kx-ui`'s translations (`theme.<id>`), and the spike's stage script is deleted |
| `spike.toml` `[keyboard]`, `[layout]`, `[size]`, `[keys]`, `[bar]`, `[look]`, `[lang]` | Settings schema defaults, or the homes the rows above name: `[layout]`, the `[size]` presets, step and limits, `[keyboard] relabel_ms`, the latch and Caps lines and the bubble corner are `kx-mod-keyboard`'s (`board.toml`, `[keyboard]`), and `[size] poll_ms` is `[ui] poll_ms`. `[size]`, `[look]`, `[keys]`, `[bar]`, `[lang]`, `[assets]` and the guard lines ended; `[keyboard]` keeps the face's title and stage-1 block for the tools row (P9 to P12) |
| `qt-kb/src/main.rs` | `apps/keyxtend`: its own `main.rs` and `run.rs`, with the one running copy, the log, the kernel's thread and Qt's loop; the spike file is deleted |
| `qt-kb/src/{bridge,board}.rs`, `qt-kb/build.rs` | `kx-ui` bridges and build: `bridge.rs` is `bridges/keyboard.rs` and `board.rs` is `bridges/ui.rs`, both over the app's hub, and `build.rs` lists the QML files in one table. The spike's other bridges (`tools`, `voice`, `panel`; P4 deleted `ring`) stay as source for their phases and still name the old `bridge` helpers, whose homes are: `init`, `config` and `themes` → `kx_ui::install` and the `Hub` (each module's own settings, the look through `look_json`); `ms` → the `[ui]` timings in `settings_json`; `spot` → the `{x, y}` or `null` JSON of `bubble_at` and `cursor` |
| `qt-kb/qml/{main,KbWindow,Extras,Key,KeyText,CapFace,DPad,TopBar,Bubble,Glow,Catch,Mover,Corner,Chip,Icon,Tip,Tween,ColourTween,PopIn,Shade}.qml` | `kx-ui` QML views in `crates/kx-ui/qml/`, reading the hub's JSON: `main.qml` is part of kx-ui's root, the window's body is `Plate.qml`, `Key.qml`'s inner shading and LED are `Inset.qml` and `Led.qml`, and `Label.qml` (grayscale text) and `Hover.qml` (the high-contrast outline) are new. `Extras.qml` draws nothing P3 shows, so it stays whole as source, its parts listed in P4 and P9 to P12; the spike views import the moved parts from `crates/kx-ui/qml`. The P1 test strip's sample chips ended, `ThemeStrip.qml` stays for P13, and `Chip.qml` waits for word suggestions (P7) |
| `stage-lib.ps1`, `qt-kb/stage.ps1`, `assets/` | `cargo xtask stage` builds `keyxtend` in release and stages it in `target/stage/keyxtend` with only the Qt files `[workspace.metadata.stage]` lists; its check refuses TLS, network-information and QML debugging plugins, any import found neither in the stage nor in System32, and any library nothing imports. Both scripts are deleted, the assets are compiled in, and `dev-install` adds the "KeyXtend (dev)" Start-menu shortcut |
| `qt-kb/qmltest/tst_motion.qml`, `look-full.js`, `look-reduced.js`, `look-dolch.js` | `kx-ui` Qt Quick Tests in `crates/kx-ui/qmltest/`, fed by the real theme loader: `tst_motion.qml` reads kx-theme's look fixtures and the face's golden files, and the look files ended. Its caption and pill checks stay in the spike as `tst_extras.qml` (P10, P11) |
| Gates `g1`–`g4`, `g25` language part | Aimed at the product app |

- **Fonts and icons** moved from a local assets folder outside the repo into the repo's `assets/`, with their licences, checked by `kx-licence-check`, and are compiled into the binary (ADR-0016).
- **Open gates finished here:** G8 budgets, G9 window flags, G10 screen readers, G11 DPI, G13 live following and frosted glass, G26 other languages.

## P4 — Mouse assist

| Spike | Product |
|---|---|
| `core/src/{hold,hold/tests,holdcfg}.rs`, `spike.toml` `[hold]`, `[ring]` | `kx-mod-mouse`, settings schema. The spike copies are deleted; `Pt` stays in `core/src/pt.rs` for the spike's later parts and kx-gates |
| `qt-kb/qml/{Ring,RingFace}.qml`, the ring-test part of `qt-kb/qml/{Extras,ThemeStrip}.qml`, `qt-kb/src/ring.rs` | `kx-ui` overlay. G12 times the product overlay's frames with PresentMon, with no stats file or test feature, so `kx-test-support`'s `ringstats` is deleted. The ring's motion tokens (`ringmove`) are already in `kx-theme` |
| `tools/kx-gates` `{assist,hookhost,hookio,hookstate}.rs` | `kx-platform-windows` hooks |
| `qt-kb/qmltest/tst_ring.qml`, `look-ring-full.js`, `look-ring-reduced.js` | `kx-ui` Qt Quick Tests |
| Gates `g5`, `g12` (with `g12pts`), `g17`, `g18`, `hand` | Aimed at the product app |

## P5 to P12 — One feature each

| Phase | Spike | Product |
|---|---|---|
| P5 Scroll | `tools/kx-gates` `scroll.rs` routes (post, send, `act`), `g6track.rs`, `mouse::wheel` and `kx-gates.toml` `[g6]` | `kx-mod-scroll` logic, `kx-platform-windows` routes. The gates' copies are deleted; `scroll.rs` keeps the picture and one `ScrollPattern` step for G6's lag record |
| P5 Scroll | Gates `g6` (with `g6cases`, `g6judge`, `g6rec`, `g6rows`, `g6scene`, `g6spot`, `g6uia`), and the scroll steps of `g8`, `g9`, `g10` (`g8scroll`, `g9pad`, `g10pad`) | G6 drives the test build's pad, never Word; G8, G9 and G10 the product app's. Their settings are in `kx-gates-p5.toml` |
| P6 Shortcuts | Gate `g25` shortcut part: Ctrl+C, V and Z, Win+V and Alt+Tab sent by kx-gates itself | `kx-mod-keyboard` shortcuts layer, OS shortcut table in `kx-platform-windows`, and `kx-mod-power`. G25 now clicks the product's keys in Notepad (`g25layer`, `g25wins`, `g25watch`): the layer's Select all, Copy, Paste, Undo, Copy all and Switch app, then Win and V, then Power, whose dialog it only ever cancels; `[g25]` names the switcher's and the power dialog's windows, and its `alt_tab` and `win_v` and `[edit_keys] undo` ended |
| P6 Shortcuts | `core/src/langkey.rs` G25 helpers (`installed`, `layout_of`, `ask`, `step_from`), which G10 shares | `tools/kx-gates` `langs.rs` over `kx-platform-windows`, with their tests; G10, G22 and G25 use it, and the spike copies are deleted |
| P7 Prediction | `core/src/{uia,uia/act,uia/read}.rs` (UIA client, password flag), and `rects_of` from `core/src/uia/geom.rs` | `kx-platform-windows` `uia`, which the focused-box watch (`field`), the gates and the spike's pill use; a password box is the `FieldWatch` service's `Password` kind (ADR-0033), not an intercept event; gate `g21` imports the product's. The pill maths left in `uia/geom.rs` is now `core/src/geom.rs` |
| P7 Prediction | Gate `g21`'s field-kind check (`g21 chrome`, `g21 win32`), which stays as its first half | G21's product half `g21 keyxtend` (`g21prod`), new gate `g27` (`g27`, `g27lane`, `g27type`) and G8's prediction part `g8 predict` (`g8predict`, `g8words`), all on copies of the product they start (`owncopy`, with `ownset` keeping the owner's settings file). Their settings are in `kx-gates-p7.toml` |
| P8 Clipboard | `tools/kx-gates` `{clip,cliplisten,cliptext}.rs` | `kx-platform-windows` `clip`, `kx-mod-clipboard`; gate `g20` (with `clipkeep`, `winclip` staying in the tools). Moved (Task 10): `clip.rs` is `clip::open`, the Unicode read of `cliptext.rs` is `clip::read`, and the listener of `cliplisten.rs` is the watch in `clip::watch`; the spike copies are deleted. The gates use `clip::{open, read_text, write_text, marked, markers}`, and G20 hears through the product watch (`g20hear`) |
| P8 Clipboard | Gate `g20`'s spike run (`g20 notepad`, with `g20hear`), which stays as its first half | Moved (Task 19): G20 on the product `g20 keyxtend` (`g20prod`, `g20steps`, `g20keys`, `g20plan`, `clippanel`, `clipnames`, `clipcopy`, `clipown`), its worker check `g20 worker` (`workertok`) and G8's clipboard part `g8 clip` (`g8clip`, `clipfill`), on the test build with its own data folder. Their settings are in `kx-gates-p8.toml`; `clipkeep` and `winclip` stay in the tools |
| P9 Vault | `core/src/{hello,fill}.rs`, `core/src/sendkeys.rs` `text`, `qt-kb/src/tools.rs` tools-row and quick-fill part, `spike.toml` `[tools]` fill lines | `kx-mod-vault`, `kx-platform-windows` Hello; gate `g23` |
| P9 to P12 Tools row | `core/src/place.rs` (the stage-1 block and tools row), `core/src/window.rs` `guard_with_top` and `bring_back` (quick-fill and the snip), `core/src/note.rs`, the tools-row part of `qt-kb/qml/Extras.qml`, `spike.toml` `[keyboard]`; the frozen face's tool gates read them through `tools/kx-gates` `facetools` (G12, G19 pill, G22) | Each part moves with its tool's phase (P9 quick-fill, P10 pill, P11 mic, P12 snip), and each file goes with its last part |
| P10 Selection | `core/src/{selwatch,geom}.rs`, `qt-kb/qml/Pill.qml`, the pill parts of `qt-kb/qml/Extras.qml` and `qt-kb/qmltest/tst_extras.qml`, `core/src/sendkeys.rs` `combo` (the pill's Copy), `qt-kb/src/tools.rs` pill part, `spike.toml` `[tools]` pill lines | `kx-mod-selection`, `kx-platform-windows` UIA, `kx-ui`; gates `g19`, `g19pill` |
| P11 Voice | `core/src/{voice,voice/tests,voicecfg,voiceworker,typer}.rs`, `core/src/sendkeys.rs` `text_paced`, `core/src/layout.rs` (`front_layout`), `core/src/screen.rs` `bottom_centre` (the caption's spot), `qt-kb/qml/{Caption,Shimmer}.qml`, the caption parts of `qt-kb/qml/Extras.qml` and `qt-kb/qmltest/tst_extras.qml`, `qt-kb/src/voice.rs`, `spike.toml` `[voice*]` and the `[tools]` mic label | `kx-mod-voice`, `kx-ui`; process start in `kx-platform-windows` |
| P11 Voice | `core/src/weak.rs` (job objects) | `kx-platform-windows` |
| P11 Voice | `core/src/{voiceproto,lines}.rs` | `kx-ipc` |
| P11 Voice | `worker/` except `http.rs` | `apps/keyxtend-worker` |
| P11 Voice | `worker/src/http.rs` (WinHTTP) | `kx-platform-net`, used only by the worker; P11 adds it to tidy's banned-network list for `keyxtend` (ADR-0004) |
| P11 Voice | Gates `g22` (with `g22cfg`), `g22bench`, `g22type`, `wer` | Aimed at the product app |
| P11 Voice | `core/src/langkey.rs` G22 helpers (`installed_for`, `layout_for`; P6 moved `layout_of` to `langs.rs`) | Gate `g22`'s own code in `tools/kx-gates`, over `kx-platform-windows` |
| P12 Snip | `core/src/{capture,capture/tests,snip}.rs`, `core/src/screen.rs` `cursor`, the snip part of `qt-kb/qml/Extras.qml`, `qt-kb/qml/SnipOverlay.qml` (kx-ui already has an `Overlay`, the pointer overlay, so the snip's needs its own name), `qt-kb/src/tools.rs` snip part, `spike.toml` `[tools]` snip lines | `kx-mod-snip` logic, `kx-platform-windows` screen capture, `kx-ui`; gate `g24` |

## P13 — Themes, settings, languages

| Spike | Product |
|---|---|
| `qt-kb/qml/ThemeStrip.qml` | `kx-ui` quick settings strip, with its labels from `kx-ui`'s translations |
| `core/src/{panelcfg,pager,popspot}.rs`, `core/src/screen.rs` `pop_for` (the panel's spot), `core/src/window.rs` `spare` (the one window that may take focus), `qt-kb/qml/{Panel,PanelList,PopButton}.qml`, `qt-kb/src/panel.rs`, `spike.toml` `[panel]` | `kx-ui` Settings window and paged lists, `kx-settings`; gate `g7` |
| `qt-kb/qmltest/tst_panel.qml`, `look-panel-light.js`, `look-panel-dark.js` | `kx-ui` Qt Quick Tests |

**Owner's direction for P13 (2026-09-29):** Settings is a separate window with its own icon, following the keyboard's language and each theme's character as strongly as the keyboard does. The list panel is rebuilt to the full mock-up design.

## Last

Workspace files deleted last: `Cargo.toml`, `Cargo.lock`, `.cargo/`, `core/Cargo.toml`, `core/src/lib.rs` and `core/tests/smoke.rs`. The `qt-kb` Cargo files (`Cargo.toml`, `Cargo.lock`, `.gitignore`) were deleted in P3 (ADR-0016).

The models `core/src/config.rs` and `core/src/clock.rs` go at the same time.

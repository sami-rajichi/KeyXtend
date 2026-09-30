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
| `core/src/{pct,ringstats}.rs` | `kx-test-support`; `ringstats` takes its unit constants from `kx-module-api` | Move |
| `core/src/targetlog.rs`, `target-window/`, `spike.toml` `[target]` | `tools/kx-target-window`, the CI test window (spec §12); it reads `kx-target-window.toml` | Move |
| `harness/` whole, `harness.toml` | `tools/kx-gates`, the Windows gate runner; it reads `kx-gates.toml` and keeps testing the spike face until each phase aims its gates at the product | Move |

After P2, the harness rows below name files in `tools/kx-gates`. Gate logs go to `target/gate-logs`, and the Qt stage the gates use is `target/spike-stage/qt-kb`.

## P3 — Keyboard core

The `qt-kb` build stops at the start of P3 (ADR-0016): its Cargo files are deleted and the rest of `qt-kb/` stays as source until each part moves. `core/` and `worker/` keep building.

Only gates `g1`–`g4` and `g25`'s language part are aimed at the product in P3; the other gates that drive the spike face run only against its last staged or installed copy until their phase aims them at the product.

Task numbers below are those of the P3 plan (`docs/superpowers/plans/2026-09-29-phase-3-keyboard-core.md`).

| Spike | Product |
|---|---|
| `core/src/{kbctl,kbctl/*,latch,kbgeom,kbgeom/*,kbview,place,sizer,legend,status}.rs` | `kx-mod-keyboard`. Task 11 moved `kbgeom` (now `geom`), `kbgeom/spec` (now `spec`, with the rows in `board.toml`) and `sizer`, and added `place` (last place, bottom centre, per-screen DPI). Task 12 moved `latch` (now with the `release` and `stay` modes) and `legend` whole, and the typing half of `kbctl` (`kbctl/typing.rs` and its tests) into `plan` (each click's route) and `run` (`Typing`, over the injector, layout source and layouts ports); `kbview`'s `legend_mods` became `LegendMods::of`. The spike's `kbctl` keeps a `Typing` and only adds the language step, chips and the side keys' test actions. Task 13 moved `kbview` into `view`, the board and state JSON of the `KeyboardFace` (`face`); the spike's `kbctl` now draws it, dressed with its own text, Lucide icons, caps and working side keys. `status` (the status line and guard retries) goes to the `ui` module (Task 16), whose banner shows the notice keys the face returns; Task 16 moved the guard retries to `kx-ui` `guard`, and the status line ended. The spike's `place.rs` keeps only the stage-1 block and tools-row maths on the crate's `Area`, for gates G2 and G3 (Task 23), the tools row (P9–P12) and the Qt bridge (Task 16) |
| `core/src/{langinfo,langkey}.rs` | `kx-platform-windows`: the locale and registry facts (`langinfo`), the window with the keyboard and its layout (`front`) and the switch request (`layouts::ask`) moved in Task 8. What stays is built on them. Task 10 moved the naming to `kx-mod-layouts`: `short` is its `[layouts] short` setting and `space_label` its `label` (the space bar's parts), which the spike's `info` now calls; Task 15 ended the joiner in `spike.toml [lang]`, as `kx-ui`'s `lang.space` text joins the parts. Task 13 ended `LangInfo` (the spike's `label` returns the product's `LangLabel`) and the spike's stepping (`next_layout`, `neighbours`): the key steps with `kx_module_api::ports::step_to`, the rule of `Layouts::step` and the Carousel. What stays asks the app in front to switch (`ask_step`, `ask`), for the spike face and gates G22 and G25 |
| `core/src/layout.rs` | `kx-platform-windows` `layouts` (the installed list, `ToUnicodeEx` reading, Caps Lock) moved in Task 8, with the spike's scan-code helpers from `core/src/lib.rs`. What stays reads through it: `Follow` moved to `kx-mod-layouts` in Task 10 (`Layouts::refresh` uses it), and the spike keeps its Win32 half, `other_front`, which hides our own window in front, until `kbctl` moves. Task 12 ended `scan`, which only typing used, and added `OtherFront`, the layout source typing reads, so its stale-table check sees the app `other_front` sees. Task 11 ended `rows`, `rows_json`, `KeyCap`, `label` and `key_name`: the board is `kx-mod-keyboard`'s `board.toml`, and key names come from translations (Task 15); `width` went to `place.rs` with the stage-1 block |
| `core/src/{inject,window,screen,backdrop,folders,com,uiaccess}.rs`, the extended-key table in `tools/kx-gates` `keys.rs` | `kx-platform-windows`. `folders`, `com` and `uiaccess` moved in Task 6 and `inject` in Task 7; `core/src/sendkeys.rs` keeps spike helpers built on it until their phases (paced typing P11, shortcut keys P9); Task 12 ended its `chord` and `tap_scan`, which only `kbctl` used, as `kx-mod-keyboard` sends through the `InputInjector` port. Task 9 moved the window guard and window queries (`window`), work areas, DPI and the pointer (`screen`), and DWM backdrops (`backdrop`); the accent-policy blur ends. What stays is built on them: `screen.rs` keeps the start and bubble spots for `kx-mod-keyboard` placement (Task 13), the pop spot (P13) and the voice pill spot (P11); since Task 11 their bottom-centre and corner maths is `kx-mod-keyboard` `place`, and `screen.rs` only reads the work area and DPI; `window.rs` keeps the spared title and `guard_with_top` for the `ui` module (Task 16) and `bring_back` for quick-fill (P9); `backdrop.rs` keeps the solid plate for `kx-ui` (Task 16). Task 16 moved the solid plate to `kx-ui` `plate` and the bubble's corner maths to `kx-ui` `geom` (`Corner` is now a `kx-module-api` port type), ending `bubble_for`; `guard_with_top` waits, as the `WindowGuard` port has no top window yet |
| `core/src/{sysui,sysui/*}.rs` | Split. Task 9 moved the Win32 half to `kx-platform-windows`: dark apps, the accent pair, the high-contrast colours, animations off and the UI language (`sysui`), with its watcher (`sysui/watch.rs` became `watch`). Task 14 moved the pure half to `kx-theme`: `mode_for` (`mode`), `contrast_palette` and `contrast_common` (`hc`, now built from a theme palette in three parts, with dimmed keys in `GrayText` and the `Highlight` hover outline) and the accent shade (`accent`). `kx-theme` reads `kx-platform`'s `SysLook`, so the spike's own look types and `from_port` ended; the spike keeps only `read` and `watch` over `WinLook` for its face until the `ui` module takes them (Task 16). Task 16: the `ui` module reads and follows the look through the `SystemLook` port |
| `core/src/theme.rs`, `core/src/theme/{common,extras,look,motion,palette,ringmove,shape,tests}.rs` | `kx-theme` (tokens, look, contrast; no Qt, so not `kx-ui`, ADR-0016). Moved in Task 14 with their tests: WCAG maths to `contrast` (with `WCAG_AA` and `MIN_CONTRAST`, the floor of `[ui] min_contrast`), `Rgba` to `rgba`, `Mode` to `mode`, the accent to `accent`; `ringmove` came too, since `motion.toml [ring]` is a token table (P4 keeps the ring's QML and bridge). The loader reads compiled-in tokens (`Themes::shipped`), so the spike's `config::read_settings` ended |
| `core/src/paint.rs` | `kx-mod-keyboard` `view::paint` (moved in Task 13); it names palette tokens as text, so it needs no `kx-theme`. Its "every theme has these tokens" test stays in the spike's `kbctl` tests, with the spike's recording `REC`, until it moves to `apps/keyxtend` (Task 19), the only product crate that sees both |
| `core/src/facecfg.rs` | `kx-mod-keyboard` (Tasks 11 and 13): key roles, latches and side-key actions in `board.toml`, the bubble corner in `[keyboard]`, the top-bar buttons in the face. Names and labels go to translations (Task 15), icon names to `kx-theme` `icons.toml` (Task 14 wrote it, with `palette` for the new theme button; Task 13 ended the spike's `[keys]` and `[bar]` icon lines, as the spike reads `icons.toml` too, and gave the test panel its own `PanelButton`); `[assets]` ends, as assets are compiled in (Task 16). Task 11 moved the roles, latches and Caps Lock to `board.toml` and the bubble corner to `[keyboard] bubble_corner`; the side block's role is "later", so the side-key test actions stay in `spike.toml [keys]` for the spike face only. Task 15 wrote the product's key names and labels and the bar names in `kx-ui`'s translations; the spike keeps its English `[keys]` and `[bar]` texts for its own face and gate G7, which finds the Settings key by name, until those move |
| `core/src/lookcfg.rs` | The `[ui]` settings of the `ui` module in `kx-ui` (Task 16). Task 14 moved `ModeChoice` to `kx-theme` and made `LookChoice` the look view's input (theme id, mode, `follow_accent`, `reduced_motion`, `min_contrast`), so `kx-theme` needs nothing from `kx-ui`; the spike's `LookConfig::choice` builds one, taking the accent for a theme its `accent_themes` lists. `Backdrop` reads as `kx-platform`'s `BackdropKind` since Task 9. Task 16 wrote the `[ui]` settings (`settings.rs`, `defaults.toml`); `accent_themes` became one `follow_accent` switch, `[size] poll_ms` joined them, and the spike's contrast check now uses `MIN_CONTRAST`. Task 17 ended `plate_alpha`: the share of the plate kept over a backdrop is kx-theme's `[backdrop] plate` token |
| `core/src/lookview.rs` | `kx-theme` `view` (moved in Task 14), called by the `ui` module, not the keyboard: `look_view(themes, choice, sys)` over `kx-platform`'s `SysLook`, now also with the icons and the high-contrast `hover_line`; the Win32 reading moved to `kx-platform-windows` in Task 9 |
| `core/src/note.rs` | Stays in the spike (Model) for `fill.rs` (P9) and `typer.rs` (P11); the product shows such notes as kernel notices in the banner (Task 18) |
| `themes.toml`, `shape.toml`, `motion.toml` | `kx-theme` tokens in `crates/kx-theme/tokens/`, compiled in (Task 14), with the new `icons.toml`, the P3 sizes (notice, menu, language list, Cycle and List keys, `[dim]`, `[hc_outline]`), the notice and menu moves, and `accent` per theme; `spike/stage-lib.ps1` no longer stages them. Task 15 moved the theme names to `kx-ui`'s translations (`theme.<id>`) |
| `spike.toml` `[keyboard]`, `[layout]`, `[size]`, `[keys]`, `[bar]`, `[look]`, `[lang]` | Settings schema defaults, or the homes the rows above name; `[assets]` ends (ADR-0016). Task 11 moved `[layout]`, the `[size]` presets, step and limits, `[keyboard] relabel_ms`, the `[keys]` latch and Caps lines and `[bar] bubble_corner` to `kx-mod-keyboard` (`board.toml` and `[keyboard]`); `[size] poll_ms` moved to `[ui] poll_ms` in Task 16. Task 15 ended `[lang]` |
| `qt-kb/src/main.rs` | `apps/keyxtend` |
| `qt-kb/src/{bridge,board}.rs`, `qt-kb/build.rs` | `kx-ui` bridges and build. Moved in Task 16: `bridge.rs` is `bridges/keyboard.rs` and `board.rs` is `bridges/ui.rs`, both over the app's hub instead of the spike core; `build.rs` lists the QML files in one table. The spike's other bridges (`tools`, `voice`, `panel`, `ring`) stay as source for their phases. They and the spike's `main.rs` still name the old `bridge` helpers, whose product homes are: `init`, `config` and `themes` → `kx_ui::install` and the `Hub` (each module's own settings, the look through `look_json`); `ms` → the `[ui]` timings in `settings_json`; `spot` → the `{x, y}` or `null` JSON of `bubble_at` and `cursor` |
| `qt-kb/qml/{main,KbWindow,Extras,Key,KeyText,CapFace,DPad,TopBar,Bubble,Glow,Catch,Mover,Corner,Chip,Icon,Tip,Tween,ColourTween,PopIn,Shade}.qml` | `kx-ui` QML views. Task 17 moved them to `crates/kx-ui/qml/`, reading the hub's JSON: `main.qml` joined kx-ui's root, the window's body became `Plate.qml`, and `Key.qml` gave its inner shading and LED to `Inset.qml` and `Led.qml`; `Label.qml` (grayscale text) and `Hover.qml` (the high-contrast outline) are new. `Extras.qml` draws nothing P3 shows, so it stays whole as source: the ring test goes with P4, the tools row with P9–P12, the pill with P10, the caption with P11 and the snip overlay with P12. The spike views that use the moved parts import `crates/kx-ui/qml`. The P1 test strip and its sample chips end; `Chip.qml` waits for word suggestions (P7) |
| `stage-lib.ps1`, `qt-kb/stage.ps1`, `assets/` | `cargo xtask stage` (Task 20), with a check that no Qt network or TLS plugin is staged |
| `qt-kb/qmltest/tst_motion.qml`, `look-full.js`, `look-reduced.js`, `look-dolch.js` | `kx-ui` Qt Quick Tests, fed by the real theme loader. Task 17 moved `tst_motion.qml` to `crates/kx-ui/qmltest/`, where it reads kx-theme's look fixtures and the face's golden files, and the look files ended. Its caption and pill checks stay in the spike as `tst_extras.qml` for P10 and P11 |
| Gates `g1`–`g4`, `g25` language part | Aimed at the product app |

- **Fonts and icons** are staged today from a local assets folder outside the repo. P3 moves them into the repo's `assets/` with their licences, checked by `kx-licence-check`, and compiles them into the binary (ADR-0016).
- **Open gates finished here:** G8 budgets, G9 window flags, G10 screen readers, G11 DPI, G13 live following and frosted glass, G26 other languages.

## P4 — Mouse assist

| Spike | Product |
|---|---|
| `core/src/{hold,hold/tests,holdcfg}.rs`, `spike.toml` `[hold]`, `[ring]` | `kx-mod-mouse`, settings schema |
| `qt-kb/qml/{Ring,RingFace}.qml`, `qt-kb/src/ring.rs` | `kx-ui` overlay; frame stats only behind a test feature, so no shipped crate depends on `kx-test-support`. The ring's motion tokens (`ringmove`) moved to `kx-theme` in P3 Task 14 |
| `tools/kx-gates` `{assist,hookhost,hookio,hookstate}.rs` | `kx-platform-windows` hooks |
| `qt-kb/qmltest/tst_ring.qml`, `look-ring-full.js`, `look-ring-reduced.js` | `kx-ui` Qt Quick Tests |
| Gates `g5`, `g12` (with `g12pts`), `g17`, `g18`, `hand` | Aimed at the product app |

## P5 to P12 — One feature each

| Phase | Spike | Product |
|---|---|---|
| P5 Scroll | `tools/kx-gates` `scroll.rs` routes | `kx-mod-scroll` logic, `kx-platform-windows` routes; gate `g6` |
| P6 Shortcuts | Gate `g25` shortcut part | `kx-mod-keyboard` shortcuts layer, OS shortcut table in `kx-platform-windows` |
| P7 Prediction | `core/src/{uia,uia/act,uia/read}.rs` (UIA client, password flag) | `kx-platform-windows` UIA; the password intercept event in `kx-module-api`; gate `g21` |
| P8 Clipboard | `tools/kx-gates` `{clip,cliplisten,cliptext}.rs` | `kx-platform-windows` clipboard, `kx-mod-clipboard`; gate `g20` (with `clipkeep`, `winclip` staying in the tools) |
| P9 Vault | `core/src/{hello,fill}.rs`, `qt-kb/src/tools.rs` tools-row and quick-fill part, `core/src/sendkeys.rs` `combo`, `spike.toml` `[tools]` fill lines | `kx-mod-vault`, `kx-platform-windows` Hello; gate `g23` |
| P10 Selection | `core/src/{selwatch,uia/geom}.rs`, `qt-kb/qml/Pill.qml`, `qt-kb/src/tools.rs` pill part, `spike.toml` `[tools]` pill lines | `kx-mod-selection`, `kx-platform-windows` UIA, `kx-ui`; gates `g19`, `g19pill` |
| P11 Voice | `core/src/{voice,voice/tests,voicecfg,voiceworker,typer}.rs`, `core/src/sendkeys.rs` `text_paced`, `qt-kb/qml/{Caption,Shimmer}.qml`, `qt-kb/src/voice.rs`, `spike.toml` `[voice*]` and the `[tools]` mic label | `kx-mod-voice`, `kx-ui`; process start in `kx-platform-windows` |
| P11 Voice | `core/src/weak.rs` (job objects) | `kx-platform-windows` |
| P11 Voice | `core/src/{voiceproto,lines}.rs` | `kx-ipc` |
| P11 Voice | `worker/` except `http.rs` | `apps/keyxtend-worker` |
| P11 Voice | `worker/src/http.rs` (WinHTTP) | `kx-platform-net`, used only by the worker; P11 adds it to tidy's banned-network list for `keyxtend` (ADR-0004) |
| P11 Voice | Gates `g22` (with `g22cfg`), `g22bench`, `g22type`, `wer` | Aimed at the product app |
| P12 Snip | `core/src/{capture,capture/tests,snip}.rs`, `qt-kb/qml/Overlay.qml`, `qt-kb/src/tools.rs` snip part, `spike.toml` `[tools]` snip lines | `kx-mod-snip` logic, `kx-platform-windows` screen capture, `kx-ui`; gate `g24` |

## P13 — Themes, settings, languages

| Spike | Product |
|---|---|
| `qt-kb/qml/ThemeStrip.qml` | `kx-ui` quick settings strip |
| `core/src/{panelcfg,pager,popspot}.rs`, `qt-kb/qml/{Panel,PanelList,PopButton}.qml`, `qt-kb/src/panel.rs`, `spike.toml` `[panel]` | `kx-ui` Settings window and paged lists, `kx-settings`; gate `g7` |
| `qt-kb/qmltest/tst_panel.qml`, `look-panel-light.js`, `look-panel-dark.js` | `kx-ui` Qt Quick Tests |

**Owner's direction for P13 (2026-09-29):** Settings is a separate window with its own icon, following the keyboard's language and each theme's character as strongly as the keyboard does. The list panel is rebuilt to the full mock-up design.

## Last

Workspace files deleted last: `Cargo.toml`, `Cargo.lock`, `.cargo/`, `core/Cargo.toml`, `core/src/lib.rs` and `core/tests/smoke.rs`. The `qt-kb` Cargo files (`Cargo.toml`, `Cargo.lock`, `.gitignore`) were deleted in P3 (ADR-0016).

The models `core/src/config.rs` and `core/src/clock.rs` go at the same time.

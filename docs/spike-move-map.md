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

| Spike | Product |
|---|---|
| `core/src/{kbctl,kbctl/*,latch,kbgeom,kbgeom/*,kbview,place,sizer,legend,status}.rs` | `kx-mod-keyboard` |
| `core/src/{langinfo,langkey}.rs` | `kx-mod-keyboard` logic; layout queries and switching in `kx-platform-windows` |
| `core/src/layout.rs` | `kx-mod-layouts` logic; `ToUnicodeEx` reading in `kx-platform-windows` |
| `core/src/{inject,window,screen,sysui,sysui/*,backdrop,folders,com,uiaccess}.rs`, the extended-key table in `tools/kx-gates` `keys.rs` | `kx-platform-windows` |
| `core/src/theme.rs`, `core/src/theme/{common,extras,look,motion,palette,shape,tests}.rs`, `core/src/{paint,lookview,lookcfg,facecfg,note}.rs` | `kx-ui` (theme loader, look, view settings) |
| `themes.toml`, `shape.toml`, `motion.toml` | Theme tokens |
| `spike.toml` `[keyboard]`, `[layout]`, `[size]`, `[keys]`, `[bar]`, `[look]`, `[lang]`, `[assets]` | Settings schema defaults |
| `qt-kb/src/main.rs` | `apps/keyxtend` |
| `qt-kb/src/{bridge,board}.rs`, `qt-kb/build.rs` | `kx-ui` bridges and build |
| `qt-kb/qml/{main,KbWindow,Extras,Key,KeyText,CapFace,DPad,TopBar,Bubble,Glow,Catch,Mover,Corner,Chip,Icon,Tip,Tween,ColourTween,PopIn,Shade}.qml` | `kx-ui` QML views |
| `stage-lib.ps1`, `qt-kb/stage.ps1`, `assets/` | `cargo xtask dev-install` staging, with a check that no Qt network or TLS plugin is staged |
| `qt-kb/qmltest/tst_motion.qml`, `look-full.js`, `look-reduced.js`, `look-dolch.js` | `kx-ui` Qt Quick Tests, fed by the real theme loader |
| Gates `g1`–`g4`, `g25` language part | Aimed at the product app |

- **Fonts and icons** are staged today from `D:\dev\assets`, outside git. P3 decides, with `kx-licence-check`, whether they live in the repo or are fetched by hash at build time.
- **Open gates finished here:** G8 budgets, G9 window flags, G10 screen readers, G11 DPI, G13 live following and frosted glass, G26 other languages.

## P4 — Mouse assist

| Spike | Product |
|---|---|
| `core/src/{hold,hold/tests,holdcfg}.rs`, `spike.toml` `[hold]`, `[ring]` | `kx-mod-mouse`, settings schema |
| `core/src/theme/ringmove.rs`, `qt-kb/qml/{Ring,RingFace}.qml`, `qt-kb/src/ring.rs` | `kx-ui` overlay; frame stats only behind a test feature, so no shipped crate depends on `kx-test-support` |
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
| P9 Vault | `core/src/{hello,fill}.rs`, `qt-kb/src/tools.rs` tools-row and quick-fill part, `spike.toml` `[tools]` fill lines | `kx-mod-vault`, `kx-platform-windows` Hello; gate `g23` |
| P10 Selection | `core/src/{selwatch,uia/geom}.rs`, `qt-kb/qml/Pill.qml`, `qt-kb/src/tools.rs` pill part, `spike.toml` `[tools]` pill lines | `kx-mod-selection`, `kx-platform-windows` UIA, `kx-ui`; gates `g19`, `g19pill` |
| P11 Voice | `core/src/{voice,voice/tests,voicecfg,voiceworker,typer}.rs`, `qt-kb/qml/{Caption,Shimmer}.qml`, `qt-kb/src/voice.rs`, `spike.toml` `[voice*]` and the `[tools]` mic label | `kx-mod-voice`, `kx-ui`; process start in `kx-platform-windows` |
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

Workspace files deleted last: `Cargo.toml`, `Cargo.lock`, `.cargo/`, `core/Cargo.toml`, `core/src/lib.rs`, `core/tests/smoke.rs`, `qt-kb/Cargo.toml`, `qt-kb/Cargo.lock` and `qt-kb/.gitignore`.

The models `core/src/config.rs` and `core/src/clock.rs` go at the same time.

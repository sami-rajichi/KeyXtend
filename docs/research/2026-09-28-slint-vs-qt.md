# Slint 1.18.1 vs Qt 6.11 + cxx-qt 0.10: sourced facts (2026-09-28)

Desk research for ADR-0013, gathered before P1 stage 2. Numbers in brackets point to the sources below; "unverified" means no primary source was found.

## Versions
- Slint 1.18.1 (21 Sep 2026) is the latest release [1]. Qt 6.11.0 shipped 23 Mar 2026 [17]; Qt 6.12 LTS is due 30 Sep 2026 [44].

## Deal-breaker risks found
- **Slint, Arabic:** RTL mirroring (#2294, open since Feb 2023) and bidi text editing (#7841, open since Mar 2025) have no linked PR [28].
- **Slint, themes on Windows:** inset shadows and spread need the Skia renderer [3]. Skia on Windows defaults to wgpu/D3D12 in 1.18, where transparent windows render black (#13572, open) [9] and idle memory is about 155 MB instead of 19.5 MB (#13470, closed "not planned") [10]. Skia-OpenGL uses about 46 MB [10]; its transparency on NVIDIA is unverified [12b]. The software renderer has no drop shadows or rotation [7][8].
- **Slint, macOS:** built on winit 0.30; a non-activating NSPanel exists only in winit 0.31 betas [30], and Slint's move is on a feature branch (#13499, tracking #11243) [31]. The fallback is a runtime class swap, as tauri-nspanel does [32].
- **Both, GNOME Wayland:** no layer-shell; third-party on-screen keyboards need a GNOME Shell extension [39].

## Comparison
| Topic | Slint 1.18.1 | Qt Quick 6.11 |
|---|---|---|
| Blur of own content | none (#612 open) [5] | `MultiEffect` blur [14] |
| Blur behind the window | DWM calls only (#4121) [13] | DWM calls only [26] |
| Shadows | one drop shadow per item (#11295); spread and inset Skia-only [3][4] | `RectangularShadow`, `MultiEffect`; inset needs a shader [15] |
| Gradients | linear, radial, conic [2] | linear, radial, conical [17] |
| Custom shaders | none; PRs closed unmerged [6] | `ShaderEffect` (not on the software backend) [16] |
| Masks | rounded `clip` only [5] | `MultiEffect` masks, layer masks [14][17] |
| Animation | `animate`, states, transitions, springs, path animation [2][19] | Behaviors, States, Animators on the render thread, particles, Lottie [17][18] |
| Styles | fluent, material, cupertino, cosmic, qt, native [20] | Basic, Fusion, Material, Universal, FluentWinUI3, macOS, iOS [25] |
| Dark mode, live | `Palette.color-scheme`; Linux live changes unreliable (#4392) [21][23] | `QStyleHints::colorScheme` with a change signal [24] |
| Accent colour | system accent since PR #11229 (Apr 2026); Windows live follow partly unverified [22] | `QPalette::Accent`, `SystemPalette.accent` [24] |
| High contrast | no upstream API found (unverified) | `QAccessibilityHints::contrastPreference` (6.10) [24] |
| Mica | works on FemtoVG and software; broken on Skia-wgpu [2][9] | needs `DwmSetWindowAttribute`; one glitch report [27] |
| RTL | hand-built | `LayoutMirroring`, bidi text inputs [29] |
| macOS no-focus panel | needs winit 0.31 or a class swap | `Qt::Tool` is a panel; `WindowDoesNotAcceptFocus`; keep-visible property [33] |
| Linux on top | X11 always-on-top broken (#6691); layer-shell only via early `layer-shika` [36][37] | X11 hints; LayerShellQt on KDE [34][38] |
| Screen readers | AccessKit on all three systems [35] | native on all three |
| Existing keyboards | one in-app example [41] | plasma-keyboard, Maliit [40] |
| Memory, small app | 14–51 MB by renderer on Windows (2023); 155 MB on 1.18 Skia-wgpu [10][42] | about 35 MB (2023); 6.11 figure unverified [42] |
| Binary / deploy | 2.6–3.6 MB binary [42] | 13–14 MB binary; about 100 MB raw deploy before trimming [42][43] |
| Licence | GPLv3, royalty-free or commercial [45] | LGPLv3; GPL-only modules fit a GPLv3 app [45] |
| Maturity | four minor releases in 2026; LibrePCB 2.0 moved from Qt [1][47] | mature; cxx-qt 0.10 says "early development" [46] |

## Sources
[1] https://crates.io/crates/slint/versions · [2] https://github.com/slint-ui/slint/blob/master/CHANGELOG.md · [3] https://docs.slint.dev/latest/docs/slint/reference/elements/rectangle/ · [4] https://github.com/slint-ui/slint/issues/11295 · [5] https://github.com/slint-ui/slint/issues/612, /issues/2066, /issues/13502 · [6] https://github.com/slint-ui/slint/pull/10874, /pull/11191 · [7] https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/ · [8] https://github.com/slint-ui/slint/issues/4177 · [9] https://github.com/slint-ui/slint/issues/13572 · [10] https://github.com/slint-ui/slint/issues/13470 · [12b] https://github.com/Secrtect/slint-windows-mica-template · [13] https://github.com/slint-ui/slint/issues/4121 · [14] https://doc.qt.io/qt-6/qml-qtquick-effects-multieffect.html · [15] https://doc.qt.io/qt-6/qml-qtquick-effects-rectangularshadow.html · [16] https://doc.qt.io/qt-6/qml-qtquick-shadereffect.html · [17] https://www.qt.io/blog/qt-6.11-released · [18] https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html · [19] https://slint.dev/blog/slint-1.18-released · [20] https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/ · [21] https://docs.slint.dev/latest/docs/slint/reference/std-widgets/globals/palette/ · [22] https://github.com/slint-ui/slint/pull/11229 · [23] https://github.com/slint-ui/slint/issues/4392 · [24] https://doc.qt.io/qt-6/qstylehints.html, https://doc.qt.io/qt-6/qaccessibilityhints.html · [25] https://doc.qt.io/qt-6/qtquickcontrols-fluentwinui3.html · [26] https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type · [27] https://forum.qt.io/topic/163927 · [28] https://github.com/slint-ui/slint/issues/2294, /issues/7841 · [29] https://doc.qt.io/qt-6/qml-qtquick-layoutmirroring.html · [30] https://github.com/rust-windowing/winit/issues/3894 · [31] https://github.com/slint-ui/slint/issues/11243, /pull/13499 · [32] https://github.com/ahkohd/tauri-nspanel · [33] https://code.qt.io/cgit/qt/qtbase.git/plain/src/plugins/platforms/cocoa/qcocoawindow.mm?h=6.11 · [34] https://raw.githubusercontent.com/qt/qtbase/6.11/src/plugins/platforms/xcb/qxcbwindow.cpp · [35] https://github.com/AccessKit/accesskit · [36] https://github.com/slint-ui/slint/issues/6691 · [37] https://crates.io/crates/layer-shika · [38] https://github.com/KDE/layer-shell-qt · [39] https://discourse.gnome.org/t/supporting-third-party-accessibility-software-on-gnome-wayland/38214 · [40] https://github.com/KDE/plasma-keyboard · [41] https://github.com/slint-ui/slint/blob/master/examples/virtual_keyboard/README.md · [42] https://github.com/slint-ui/slint/discussions/3376 · [43] https://forum.qt.io/topic/162307/windows-deployment-extra-files · [44] https://wiki.qt.io/Qt_6.12_Release · [45] https://doc.qt.io/qt-6/licensing.html · [46] https://github.com/KDAB/cxx-qt · [47] https://librepcb.org/blog/2026-01-28_release_2.0.0/

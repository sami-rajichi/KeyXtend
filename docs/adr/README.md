# Architecture decision records

Never rewrite an accepted ADR. Supersede it with a new one.

| # | Decision | Status |
|---|---|---|
| 0001 | [Record architecture decisions](0001-record-architecture-decisions.md) | Accepted |
| 0002 | [Rust for all code, Slint for all windows](0002-rust-and-slint.md) | Superseded by ADR-0013 |
| 0003 | [The keyboard process itself holds uiAccess](0003-uiaccess-keyboard-process.md) | Accepted |
| 0004 | [Untrusted data and network access live in worker processes](0004-worker-processes.md) | Accepted |
| 0005 | [Micro-kernel with independent "Lego" modules](0005-micro-kernel-modules.md) | Accepted |
| 0006 | [Licence: GPL-3.0-or-later for the app](0006-licence-gpl3.md) | Accepted for now (2026-09-26). Revisit before v1.0. Slint notes superseded by ADR-0013. |
| 0007 | [AI features are local-first and free](0007-local-first-free-ai.md) | Accepted |
| 0008 | [Every UI works without scrolling, dragging or right-click](0008-paged-no-scroll-ui.md) | Accepted |
| 0009 | [Characters as Unicode, shortcuts as virtual keys](0009-input-injection-strategy.md) | Accepted. Verified by gate G1 (ADR-0013). |
| 0010 | [Sensitive data is encrypted at rest with an OS-protected key](0010-encryption-at-rest.md) | Accepted |
| 0011 | [Choose the UI toolkit by a measured test round](0011-toolkit-test-round.md) | Accepted (process). Result in ADR-0013; its "No leftovers" rule for code is superseded by ADR-0014. |
| 0012 | [Installer: Inno Setup 7](0012-installer-inno-setup.md) | Accepted |
| 0013 | [Qt 6 Quick with a Rust core for every window](0013-qt-quick-for-every-window.md) | Accepted (2026-09-29). Supersedes ADR-0002 and ADR-0006's Slint notes. Partly superseded by ADR-0016 (theme loader in `kx-theme`). |
| 0014 | [Keep the tested spike code and move it into the product](0014-keep-and-move-the-spike-code.md) | Accepted (2026-09-29). Its build rule for `qt-kb` is superseded by ADR-0016. |
| 0015 | [Settings and logs live in one data folder](0015-settings-and-logs-in-one-data-folder.md) | Accepted (2026-09-29) |
| 0016 | [The product app replaces the spike face](0016-product-app-replaces-the-spike-face.md) | Accepted (2026-09-29). Supersedes ADR-0014's build rule for `qt-kb` and ADR-0013's rule that `kx-ui` holds the theme loader. Its UI ports gain `Tray` in ADR-0019. Its CI line as built: Linux leaves out the app and builds `kx-ui` without Qt. |
| 0017 | [Translations: Fluent files compiled in](0017-translations-fluent-compiled-in.md) | Accepted (2026-09-30). Its language choice for the keyboard's text is superseded by ADR-0020, and its bidi rule and Arabic list join by ADR-0022. |
| 0018 | [One set-once hub for the QML objects](0018-set-once-hub-for-qml-objects.md) | Accepted (2026-09-30) |
| 0019 | [The tray icon: a port over the taskbar, and our own QML menu](0019-tray-icon-port-and-qml-menu.md) | Accepted (2026-09-30). Adds the `Tray` port to ADR-0016's UI ports. |
| 0020 | [The keyboard's text follows the keyboard's language](0020-keyboard-text-follows-the-keyboard-language.md) | Accepted (2026-10-01). Supersedes ADR-0017's language choice for the keyboard's text. |
| 0021 | [One graphics device for every window](0021-one-graphics-device-for-every-window.md) | Accepted (2026-10-10). Adds how ADR-0013's windows draw; sets spec §10's idle memory to 100 MB. Its decisions 6 and 7 gain ADR-0028's after-use reading and buffer pool limit. |
| 0022 | [Texts carry no bidi marks](0022-texts-carry-no-bidi-marks.md) | Accepted (2026-10-10). Supersedes ADR-0017's bidi rule and Arabic list join. |
| 0023 | [Give freed heap pages back after a look change](0023-give-freed-heap-pages-back-after-a-look-change.md) | Accepted (2026-10-11). Adds to ADR-0021 how memory stays flat over G8's churn; its decision 6 stands. |
| 0024 | [Controls act on the press they got](0024-controls-act-on-the-press-they-got.md) | Accepted (2026-10-11) |
| 0025 | [A pressed key draws first and presents without vsync](0025-a-pressed-key-draws-first-and-presents-without-vsync.md) | Accepted (2026-10-12). Changes spec §10's key light target. Its `present_vsync = false` is superseded by ADR-0026. |
| 0026 | [Frames keep the refresh wait; a press still draws first](0026-frames-keep-the-refresh-wait.md) | Accepted (2026-10-12). Supersedes ADR-0025's `present_vsync = false`. |
| 0027 | [The hold engine decides in the hook, behind a port](0027-the-hold-engine-decides-in-the-hook-behind-a-port.md) | Accepted (2026-10-13). Changes spec §5.1's hook discipline; as built, our keyboard's Esc also reaches the engine, only to cancel a carry, the keyboard hook also reads key releases for the host's record of held input, the health check reinstalls only after two lagging checks in a row, counts our own input as seen and skips while another desktop is in front, and the hand try reads the owner's running copy, not the test build. `REPLAY` sits in the low 32 bits of `dwExtraInfo` beside each act's 16-bit id, as Windows hands a mouse hook only those, and echoes heard inside our own send go to an inbox the host lends that send. |
| 0028 | [The renderer keeps no spare buffers, and G8 also reads after use](0028-the-renderer-keeps-no-spare-buffers.md) | Accepted (2026-10-15). Adds to ADR-0021's decisions 6 and 7. |

ADR-0011 names "ADR-0012" for the toolkit result; that number went to the installer decision, so the toolkit result is ADR-0013.

# 13. Qt 6 Quick with a Rust core for every window

- **Date:** 2026-09-29
- **Status:** Accepted. Supersedes ADR-0002 and ADR-0006's Slint notes. Records the result of the test round in ADR-0011.

## Context

ADR-0011 set a measured test round (roadmap Phase 1, gates G1–G26) between Qt 6 Quick with a Rust core (cxx-qt) and Slint 1.18, with a quick Tauri check. Its rule: a candidate must pass every gate, and if both pass, Qt wins for its macOS and Linux window support and its Arabic editing.

- Stage 1 ran the deal-breakers on both toolkits; both passed. Tauri failed G3.
- From stage 2 the owner chose to test Qt first and keep Slint as the fallback, to be retested only if Qt failed a must-pass gate.
- Qt failed none of the gates that ran. The owner approved its look in all three themes, light and dark, and closed the round on 2026-09-29 with the remaining gates moved into later phases (below).

## Decision

- **Qt 6 Quick (QML) draws every window; Rust holds all logic.** The bridge is cxx-qt; versions stay pinned (Qt 6.11.2, cxx-qt 0.10.0) and every upgrade is reviewed.
- **Slint and Tauri are dropped.** Their spike code is not carried forward (ADR-0014).
- **QML only draws and forwards clicks.** Rust decides; each view gets one bridge object with read-only settings and a small state it redraws from.
- **QML is built into the binary** as a resource module and never loaded from disk, since the keyboard runs with uiAccess.
- **Bridge code is the one `unsafe` exception outside `kx-platform-*`.** cxx-qt needs an `unsafe extern "C++"` block in each bridge, so `kx-ui` may hold these blocks in `src/bridges/` only, each with a `// SAFETY:` comment. Tidy learns this exception when `kx-ui` is created (P3).
- **Window rules** (replacing ADR-0002's Slint rules):
  - no-focus windows are `Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus`;
  - click-through windows (shadows, ring, caption bar) add `Qt.WindowTransparentForInput`;
  - the Windows adapter guards every window except Settings: it restores `WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW` in `WM_STYLECHANGING` and answers `MA_NOACTIVATE`;
  - only the full Settings window is a normal, focus-taking window, and it gives focus back when it closes;
  - never call `requestActivate()` on a no-focus window.
- **The pointer ring is a Qt window**, not a native layered window: G12 showed no lost clicks at the display's full rate.
- **If Qt fails a deferred must-pass gate later**, a new ADR weighs a fix against a return to Slint, whose face stays in the history of `spike/toolkits`.

## Results (development PC, Windows 11, 2026-09-27 to 2026-09-29)

| Gate | Result |
|---|---|
| G1 Typing (core) | Test window, Chrome, Terminal and Word received all 1000 characters (Word's AutoCorrect curls quotes). Notepad 993–997: it drops a few in fast bursts. |
| G2 No focus | Qt: focus and caret kept 1000/1000, 1000/1000 characters; click to character p50 4.2 ms, p99 13.1 ms. Slint: focus 1000/1000, 999/1000 characters; the one miss came while the owner touched the mouse. |
| G3 Top band | Qt and Slint stay above Start, Search and Task Manager. **Tauri fails:** WebView2 does not start under uiAccess (0x80070057). |
| G4 Admin windows | Qt 200/200 into an admin Terminal. Nothing beyond uiAccess is needed. |
| G5 Hold engine (core) | 20/20 in four rounds, plain and admin; hook p99 0.066 ms and 0.053 ms. |
| G6 Scroll (core) | All three routes scroll Notepad, Chrome and Explorer up and down; sideways only in Chrome. |
| G7 Arabic UI | Qt: joined legends, harakat, a mirrored panel, and an Arabic field whose caret moves only between letters. |
| G12 Overlay | Qt: 200/200 clicks under the ring reached the app; 167 fps on a 165 Hz screen, p99 7.3 ms. |
| G13 Themes | Partial: the owner's look check passed for Native, ET66 and Dolch, light and dark. Live following and the two frosted-glass tries were not run. |
| G17, G18 (core) | Grab moves a file and selects text; Esc cancels; Shift+click and Ctrl+click work. |
| G19 Selection | Pill at most 1 px off, focus kept, shown 136–579 ms after the selection; Copy copies. |
| G20, G21 (core) | Clipboard: every copy heard, marked copies skipped, an older copy pasted back. Password boxes flagged in Chrome and Win32. |
| G22 Voice | Qt: caption on top without focus; words 0.8–1.3 s after Mic stop. Local base model 0.14× real time (weak mode 0.36×); fine for English only. |
| G23 Quick-fill | Only the "Hello unavailable" path was tested: nothing typed, no prompt. |
| G24 Snip | Qt: overlay above everything in about 107 ms; the saved region differs by 0 px. |
| G25 Language, shortcuts | Layout cycling and Ctrl+C/V/Z, Win+V, Alt+Tab work; injected Win+V needs uiAccess. |
| Size | Staged test build 55.4 MiB unpacked: Qt 46.0, keyboard 5.0, voice worker 2.0, fonts 2.4 (43.6 MiB earlier in the round, before the fonts and more Qt Quick parts). No network or TLS plugins; `Qt6Network.dll` stays because Qt Qml links it. |

- **Word** was not tested in G6, G17, G19 or G20: a fresh Word process showed a subscription dialog.
- **Moved to later phases:** G8, G9, G10, G11, G13 (live following, frosted glass) and G26 to P3; G14 to P14; G15 and G16 to P15, or G15 earlier once the Qt keyboard builds on macOS and Ubuntu. G15 needs the owner's yes to push.

## Toolkit findings

- Qt `Tool` windows needed no focus workarounds. Slint needed winit hooks, a guard that forced `SWP_NOACTIVATE`, and a retry to lift new windows.
- Click-through is one Qt flag; Slint needed layered-window bits and a guard to keep them.
- A `Window` declared inside another becomes its transient child (Qt 6.7+), so owned windows stay above their owner.
- To fix in P3: ClearType colour fringes on thin text; no hover outline in high contrast; `Icon.qml` uses Qt's internal `QtQuick.Controls.impl`; re-measure resize time (165 → 43 ms in a debug build).

## Consequences

- `kx-ui` holds the QML module, its bridge objects and the theme loader; logic crates stay toolkit-free.
- Qt is LGPL-3.0: it ships as separate DLLs, with its licence texts and a note on how to swap them. ADR-0006's Slint notes no longer apply.
- UI tests use Qt Quick Test (`qmltestrunner`) and find controls by accessible name.
- Update `ARCHITECTURE.md`, `CLAUDE.md`, the spec (§3.5, §3.6, §5.3, §10, §11, §12 and the open decisions), `THIRD_PARTY.md`, `CONTRIBUTING.md` and the skills that name Slint.

## Alternatives

- **Slint 1.18:** passed stage 1, but needs more window workarounds, has open Arabic editing (#7841) and mirroring (#2294) issues, and no NSPanel or layer-shell yet.
- **Tauri:** rejected; WebView2 fails under uiAccess.

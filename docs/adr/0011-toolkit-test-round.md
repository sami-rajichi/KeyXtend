# 11. Choose the UI toolkit by a measured test round

- **Date:** 2026-09-26
- **Status:** Accepted (the process). Result in ADR-0013 (Qt). The "No leftovers" rule for code is superseded by ADR-0014.

## Context

The product owner asked for a re-check with macOS and Linux in mind, and for proof of weight and speed before building. Three research passes (2026-09-26) found:

- **Window behaviour.**
  - Qt 6 has first-class flags on all three systems: `WindowDoesNotAcceptFocus` (Windows), an NSPanel for `Qt::Tool` windows (macOS), X11 hints, and LayerShellQt (Wayland).
  - Slint depends on winit, which has no NSPanel and no layer-shell (ADR-0002, Consequences).
- **Arabic.**
  - Qt edits bidirectional text and mirrors layouts.
  - Slint bidi editing (#7841) and mirroring (#2294) are still open.
- **Keyboard precedents.** Maliit (the Ubuntu Touch keyboard, with an Arabic layout) and KDE's plasma-keyboard are built with Qt Quick. No Slint keyboard was found.
- **Visuals.** Every mock-up visual can be drawn by both toolkits.
  - The frosted glass is limited by the OS: Windows may turn it solid on inactive windows, whatever the toolkit.
- **OS walls that no toolkit removes.**
  - GNOME Wayland has no public way to keep a window on top without focus, catch clicks or watch the clipboard.
  - The owner's policy: desktop built-ins first; the GNOME extension and the mouse helper are opt-in.
- **Qt costs.**
  - Bigger: an estimated 30–80 MB download and 40–80 MB of memory, not yet measured.
  - Two languages: Rust, plus QML through cxx-qt 0.10.
  - LGPLv3 obligations.
  - A QML engine inside the uiAccess process, so QML is compiled ahead of time and loaded only from built-in resources.
- **Tauri is rejected on evidence.**
  - WebView2 fails in uiAccess apps for standard users since KB5044273 ([WebView2Feedback #4884](https://github.com/MicrosoftEdge/WebView2Feedback/issues/4884), closed "not planned").
  - `focusable: false` is reported broken on macOS ([tauri #14102](https://github.com/tauri-apps/tauri/issues/14102), open).
  - An empty Tauri app uses about 204–317 MB of memory on Windows, across several processes ([benchmark](https://github.com/Elanis/web-to-desktop-framework-comparison)).
  - A browser engine inside the keyboard's trust zone conflicts with ADR-0004.

## Decision

- **Candidates.**
  - Phase 1 builds the same small test keyboard in **Qt 6 Quick with a Rust core (cxx-qt)** and in **Slint 1.18**.
  - Tauri runs only the deal-breaker gates: G2, G3 and the memory part of G8.
- **Gates and pass marks:** roadmap Phase 1 and spec §10.
- **Machines.**
  - The development PC, normally and in **weak mode**: the test keyboard is held to 2 cores at a low CPU rate, 4 GB and software rendering.
  - macOS and Ubuntu on GitHub's hosted machines, once the owner approves the repo.
  - A Linux virtual PC on D: running GNOME and KDE.
- **Rule.**
  - A candidate must pass every gate.
  - If both pass, Qt wins, for its macOS and Linux window support and its Arabic editing.
  - If Qt fails G3, G4 or the budgets and Slint passes, ADR-0002 stands.
- **No leftovers.** Spike code, screenshots, logs and virtual PCs are deleted afterwards. Only the numbers are kept, below.

## Results

Filled in after the test round.

## Consequences

- The logic crates stay toolkit-free. Only `kx-ui` and the window adapters depend on the result.
- After the result, update `ARCHITECTURE.md`, spec §11, `THIRD_PARTY.md` and the `kx-slint-theme` skill.

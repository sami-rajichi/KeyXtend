# 2. Rust for all code, Slint for all windows

- **Date:** 2026-09-26
- **Status:** Superseded by ADR-0013 (2026-09-29): Qt 6 Quick draws every window.

## Context

The requirements behind this choice:

- The keyboard window must always be on top, above the Start menu, which needs uiAccess (ADR-0003).
- It must never take focus.
- It must be small, fast and cross-platform.
- It must show Arabic correctly.
- Future camera and AI features must stay possible.

**WebView2 fails in uiAccess processes** (MicrosoftEdge/WebView2Feedback#4884, closed "not planned"). That rules out Tauri, Electron and Wails for the keyboard window.

## Decision

- **Rust** for all logic, platform code and workers.
- **Slint 1.18.x** for every window. Pin the version exactly (`=1.18.1`) and review every minor upgrade.
- **Renderer:** FemtoVG on OpenGL (the default) or Skia on OpenGL.
  - Never Skia on DX12: transparency renders black (#13572), and memory is about 155 MB instead of about 20 MB (#13470).
- Window-level behaviour is owned by the Windows platform adapter:
  - It uses the HWND from Slint's `raw-window-handle-06` / `unstable-winit-030` features.
  - It subclasses the window:
    - In `WM_STYLECHANGING`, it puts `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST` back. winit rewrites the whole extended style whenever a winit flag changes.
    - It returns `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`.
  - Windows are created with `with_active(false)` and `with_skip_taskbar(true)`.
  - We **never** call Slint's `bring_to_front()`, because it calls `SetForegroundWindow`.
  - Size and position are set only through Slint's API, because Slint re-applies its own values (#11001).
- **Pointer ring, burst and badge:** drawn by the platform adapter as a native layered, click-through window (`UpdateLayeredWindow` with a tiny-skia bitmap), not by Slint. This avoids Slint's open transparency and click-through gaps (#6021, #13572).
- **UI scale:** a global `scale` property in `.slint`. Slint has no runtime scale-factor API; `SLINT_SCALE_FACTOR` is read once.

## Evidence (verified 2026-09-26 from source and issues; nothing compiled yet)

- No Windows showstopper found. uiAccess compatibility is **unknown**: no reports either way. Slint draws in-process, so there is no reason to expect the WebView2 problem, but this is spike gate G3.
- **Arabic:**
  - Parley shapes Arabic and reorders mixed text. Key legends and captions work, with explicit `horizontal-alignment: right`, because `start` always means left.
  - **Bidi text editing in `TextInput` is officially broken** (#7841), and there is **no RTL layout mirroring** (#2294).
- **Accessibility:** AccessKit (UIA / AX / AT-SPI) is on by default.
- **Other capabilities:** built-in tray icon since 1.17; `SharedPixelBuffer` for live camera frames.
- **Measured by others:** about 19.5 MB RAM with FemtoVG. A real Slint + Skia app ships 10.7–12.6 MB release assets.
- **Maturity:** SixtyFPS GmbH maintains Slint. It has about 24k stars and a minor release every 2–4 months. LibrePCB 2.0 ships on it.

## Consequences

- **Our own UI avoids editable Arabic text fields where possible.**
  - Settings uses pickers and steppers.
  - Where text entry is unavoidable (vault entry names), Arabic editing has the known caret bug until upstream fixes it.
  - RTL mirroring of the settings window for the Arabic UI is done by hand, with a `Dir.rtl` global that swaps order and alignment.
- **macOS (later):** winit 0.30 cannot create an `NSPanel`. Either swap the class at runtime (as tauri-nspanel does) or wait for Slint on winit 0.31 (`with_panel`, Slint #11243). Budget for this in the macOS phase.
- **Linux:**
  - winit has no layer-shell. On KDE and wlroots we use a layer-shell crate (layer-shika).
  - **GNOME Wayland cannot host an always-on-top, non-activating overlay with any toolkit** (Mutter has no wlr-layer-shell). There we document the limit and use the X11 session or GNOME's own on-screen keyboard settings.
- The logic crates never depend on Slint. If Slint fails G2, G3 or G7, only `kx-ui` and the window adapter change.

## Alternatives

- **Fallback: Qt 6 via cxx-qt.**
  - Qt 6.11 has `WindowDoesNotAcceptFocus`, `WindowTransparentForInput` and `WindowStaysOnTopHint` flags, plus full RTL mirroring.
  - But cxx-qt is only at 0.10.0.
  - LGPL obligations: users must be able to relink, we must provide Qt source or a written offer and the notices; some Qt modules are GPL-only.
- **Rejected:**
  - Tauri, Electron and Wails (WebView2/Chromium under uiAccess; heavy).
  - WinUI 3 (Windows-only; uiAccess unknown).
  - egui, iced and GPUI: RTL/bidi is still open in all three, and they share winit's no-activate limits.
  - Flutter (heavy; needs native plugins for window control).

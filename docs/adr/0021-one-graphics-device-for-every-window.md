# 21. One graphics device for every window

- **Date:** 2026-10-10
- **Status:** Accepted. Adds to ADR-0013 how its Qt Quick windows draw; changes spec §10's idle memory target from 80 MB to 100 MB.

## Context

- The installed keyboard used 215 MB of private memory right after start, and about 500 MB after a day of tips and menus. Spec §10 asks for 80 MB at idle, with no growth.
- Qt Quick gives each shown window its own Direct3D 11 device and keeps it while the window is hidden. On the development PC, each device costs about 26 MB, 25 driver threads and 1,360 handles. We own 12 windows: the keyboard, the shadows, the tips, the menus, the bubble and the tray picture.
- `FontLoader` keeps every bundled font for the life of the process. Qt makes one face per named style of a variable font, so the six files cost 17.5 MB even in Native, which uses none of them.
- The measurements, options and sources are in `docs/research/2026-10-10-window-memory.md`.

## Decision

Decided by the owner on 2026-10-10.

1. **One device.** The app creates one QRhi on the GUI thread before the QML loads, on the adapter the app pins (ADR-0013's minimum-power rule). Every `QQuickWindow` adopts it through `QQuickGraphicsDevice::fromRhi` before its first frame, Loader windows included. Each window keeps its own swap chain.
2. **The basic render loop.** A QRhi is used on one thread only, so every window draws on the GUI thread (`QSG_RENDER_LOOP=basic`). Animations run at 60 frames per second, which spec §10 and gate G12 ask for.
3. **Fonts for the current theme only.** The app loads the theme's font files from Rust-held data before the QML, and drops them on a theme switch. QML no longer uses `FontLoader`.
4. **No depth buffer** (`QSG_NO_DEPTH_BUFFER=1`). No item clips with a stencil today; a theme that needs one must lift this.
5. **Device loss.** When the device is lost (a driver update, a GPU reset or sleep), the app makes a new device and rebuilds its windows from the Rust state; the user sees the keyboard come back where it was.
6. **The budget.** Spec §10's idle target becomes 100 MB of private working set and 140 MB of private commit, with no growth over 8 hours. G8 logs both, samples after a fixed warm-up and never trims the working set.
7. **The glue.** One small C++ header beside `cpp/harden.h` with its cxx bridge in `kx-ui/src/bridges/` (ADR-0013's `unsafe` rule). Rust sets the two Qt variables after `harden_qt`, as it sets the adapter pin.

## Consequences

- Measured on the development PC with the real `App.qml` over test stand-ins: idle 215 → 127 MB with the shared device alone, and about 105 MB with the fonts and the depth buffer too. Threads fall from 87 to 36 and handles from 4,471 to 1,767. Showing and hiding pop-ups 60 times leaves memory flat.
- The keyboard's windows, flags, guard, topmost and owners stay as ADR-0013 says; shadows stay separate click-through windows.
- Every frame of every window renders on the GUI thread. G2 (click to character), G8 frame times and the owner's motion check run again. Future low-level hooks run on their own threads.
- `rhi/qrhi.h` comes from Qt's versioned include path and has limited compatibility promises. The glue is pinned to Qt 6.11.2 and checked at every Qt upgrade.
- A destroyed window's buffers are reused, not given back; the growth test stays flat.
- Weak mode's software drawing uses the same path with the WARP adapter.
- Later, as P13 ideas: slimmer keys (about 86 MB) and an optional "low memory" mode that draws without the graphics chip (about 70 MB).

## Alternatives

- **Keep one device per window:** 215 MB at start and growth with each pop-up shown; refused.
- **Create pop-ups only while shown, or release graphics on hide:** 45–55 ms for each show, and one handle leaks per device made, so tips grow memory all day; refused.
- **Draw shadows inside their windows:** the shadow's margin would take clicks, since a window region also cuts the drawing; refused.
- **Share the D3D11 device and context across threads (`fromDeviceAndContext`):** the immediate context is not thread-safe; refused.
- **Software drawing for everyone:** about 70 MB and the 80 MB target holds, but the processor draws every frame and animations slow when several windows move; kept only as a P13 idea for an opt-in mode.
- **Keep the 80 MB target:** not reachable with the graphics chip on the development PC (the lowest measured is 85.6 MB); the owner chose 100 MB now and slimmer keys later.

# Window memory: options measured for P3

- **Date:** 2026-10-10. **Input for:** ADR-0021 (one graphics device for every window).
- **Short answer:** one shared graphics device for every window is the fix. It takes the app from about 215 MB to about 127 MB at idle and stops the growth each new pop-up causes. With theme-only fonts and two Qt switches it reaches about 105 MB. A slimmer key brings it to about 86 MB. 80 MB was only reached with software rendering (WARP, 69 MB).

## How this was measured
- Qt 6.11.2 on this PC's AMD iGPU (DXGI index 0), display scale 125 %, Native light theme.
- A throwaway C++ harness (`mt.exe`, built with MSVC; deleted) loads QML like `qml.exe`, can share one QRhi across all windows, and reads its own private working set (PWS), private commit, threads and handles.
- "App" means the real `App.qml` over the `qmltest` stand-ins (`FakeUi`, `FakeKeyboard`), sampled 5 s after start. The Rust side of the real app is not included.
- Noise between runs is about ±4 MB. Figures are PWS / commit in MB unless marked.

## Where the memory goes today
| Part (App, one shared device, no bundled fonts: 114 MB) | MB |
|---|---|
| Process and Qt libraries | 6 |
| One D3D11 device on the AMD driver | 25 |
| QML objects of all 12 windows (mostly the 70 keys) | 42 |
| Drawing the keyboard, its shadow and the tray picture | 41 |
| Today, on top: one more device per shown window | 26 each, 25 threads, 1,360 handles |
| Today, on top: all six bundled font files | 17.5 |

- **Keys:** with 0 keys the App is at 54 MB, so the keys cost about 60 MB (0.85 MB each). `KeyText` alone costs about 44 MB. Its per-icon `MultiEffect` costs about 9 MB, and the lock-glow `RectangularShadow` on keys and lights about 7 MB.
- **Fonts:** Qt makes one DirectWrite face per named instance of a variable font, so the cost is about "instances × file size": Noto Sans Arabic 8.3 MB (9 instances), IBM Plex Sans 4.4 (7), Rubik 3.0 (7), the three static Plex Arabic files 1.5. That is 17.5 MB in total, measured the same with and without sharing. The 33 MB measured in fix round E was not reproduced.
- **Window buffers:** swap-chain buffers barely show in PWS or commit. The depth-stencil buffer does: about 12 MB for a full-screen move catcher, 3 MB for the keyboard.

## The options, ranked
| # | Option | Idle saving (measured) | Risk | Effort | Fit with ADR-0013 |
|---|---|---|---|---|---|
| 1 | One shared QRhi for all windows, basic render loop | **−88 MB** (215 → 127); after the tray menu showed: 294 → 129. Threads 87 → 36, handles 4,471 → 1,767 | Medium: device loss, 60 fps animations, GUI-thread rendering | M | Full: flags, guard, topmost and owners are unchanged |
| 2 | Theme-only fonts, loaded in C++ before QML; ship static weights | Native −17.5; ET66 about −11; Dolch about −6 (more with static weights) | Low | S–M | Full |
| 3 | Key diet: build each key's parts by its kind; tint icons without one `MultiEffect` each; glow only on lit keys | −18 to −28 (105 → 86 at the floor) | Medium: QML restructure, look check | M–L | Full |
| 4 | `QSG_NO_DEPTH_BUFFER=1` | −6 at idle; −12 for each full-screen catcher once shown | Low–medium: stencil clips (none today) | XS | Full |
| 5 | Our own D3D11 device with `PREVENT_INTERNAL_THREADING_OPTIMIZATIONS` under #1 | −5 MB, −2 threads, −280 handles | Low | S | Full |
| 6 | Software device (WARP) shared, as an optional low-memory mode | 85.6 → 69 at the floor, 13 threads | High: CPU rendering, 50 fps with 4 animating windows | S | Full |
| 7 | Create pop-ups only while shown (Loader) | Without #1: frees 26 MB a window, but leaks (below). With #1: about −1.3 MB a window | Medium | M | Full |
| 8 | Release graphics on hide (`setPersistentGraphics(false)`) | Same as 7 without #1, same leak | Medium | S | Full |
| 9 | Draw shadows inside their window | With #1: 1–3 MB a window, only the depth buffer | High: the margin takes clicks | M | Breaks click-through |
| 10 | Tray picture without a window | With #1: about 0 | Low | S | Full |

**Rejected after measuring:**
- D3D12: 171 MB for 2 windows; the App is at 369 MB.
- OpenGL: 363 MB commit (fix round E).
- The basic render loop alone: no gain.
- `QT_D3D_NO_FLIP`: drops DirectComposition, so see-through windows lose alpha.
- `QT_D3D_MAX_FRAME_LATENCY`, `QSG_ATLAS_WIDTH/HEIGHT`, QtRendering text: no change beyond noise once #1 is in.
- Trimming the working set: PWS falls to 2 MB, but commit stays the same. It only games G8.

## Answers to the five questions
**1. Sharing one device.**
- Qt 6.11 supports one QRhi for many windows: `QQuickGraphicsDevice::fromRhi` (Qt 6.6+).
  - `QSGRhiSupport::createRhi` hands the adopted QRhi back as not owned.
  - The basic loop then gives each window its own swap chain (`rhi->newSwapChain()`) and its own render context.
  - A test with a keyboard, its shadow and 6 pop-ups rendered correctly, see-through included: 72 MB and 37 threads, against 255 MB and 185 threads with one device each.
- **Thread rule:** a QRhi is used on one thread only (QRhi docs, "Threading"). The threaded loop renders each window on its own thread, so sharing needs `QSG_RENDER_LOOP=basic`, where every window renders on the GUI thread.
- **`fromDeviceAndContext`** would put several QRhi state trackers on one immediate context. The D3D11 immediate context is not thread-safe. Untested and not advised.
- **`fromAdapter` and `fromRhiAdapter`** only choose the adapter: still one device per window, no saving.
- **The C++ glue** is one small header beside `cpp/harden.h`, with a cxx bridge in `src/bridges/`, `unsafe` with `// SAFETY:` as ADR-0013 allows:
  - After `QGuiApplication` exists and before the QML loads, it creates the QRhi once on the GUI thread with `QRhi::create(QRhi::D3D11, …)`. Qt still honours the `QT_D3D_ADAPTER_INDEX` pin.
  - It installs one app-wide event filter. On `QPlatformSurfaceEvent::SurfaceCreated` of any `QQuickWindow`, the filter calls `setGraphicsDevice(fromRhi(rhi))`, which works for Loader windows too. This must happen before a window's first expose; the harness proved this hook point works.
  - It deletes the QRhi after the last window at quit.
  - Rust sets `QSG_RENDER_LOOP=basic` through the existing `set_env`, after `harden_qt`, like the adapter pin.
  - The build needs Qt's `QtGui/6.11.2/QtGui` include path for `rhi/qrhi.h`. That API is public, but Qt gives it only limited compatibility guarantees, so it stays pinned to 6.11.2.
  - Option 5 adds `D3D11CreateDevice` on the pinned adapter and `QRhiD3D11NativeHandles`.

**2. Create on show, or keep.**

| Variant (synthetic keyboard, shadow, 6 pop-ups) | Pop-ups shown | After hide | First show of a window | Show again |
|---|---|---|---|---|
| Today: one device per window | 255 MB | 255 | about 60 ms each | 10–17 ms |
| + release on hide (8) | 289 | 118 | — | 45–55 ms each |
| + Loader (7) | 289 | 118 | about 45 ms | about 45 ms |
| Shared (1), kept | 72 | 72 | 36–41 ms, then +4 ms each | 7–14 ms |
| Shared (1) + Loader | 72 | 66 | 40–60 ms | 40–60 ms |

- **Growth test:** 3 pop-ups shown and hidden 60 times beside the keyboard.
  - Shared, kept or Loader: flat (46.6 → 47.2 MB, handles flat).
  - One device per window with 7 or 8: 70 → 83 MB, and one handle leaks per device made (180 devices, +180 handles).
  - Tips show on every hover, so 7 and 8 would grow all day.
- **Verdict:** with #1, keep every pop-up and leave Loader only where it already is.

**3. Fewer windows.**
- `SetWindowRgn` clips drawing, so a shadow inside the region also takes clicks. Outside it, the shadow is not drawn.
- `HTTRANSPARENT` passes clicks only to windows of the same thread. Per-pixel hit testing applies only to `UpdateLayeredWindow` windows, and a shadow's pixels are not zero alpha anyway.
- Switching `WS_EX_TRANSPARENT` as the pointer moves is racy and fights the guard.
- With #1 a shadow window costs only its depth buffer (gone with #4), so keep the shadow windows.
- **Tray picture:** with #1 it costs about nothing. `QPainter` with `QSvgRenderer` could draw it without a window, but the badge would then be drawn twice (QML and C++). Not worth it.

**4. Per-device knobs.**
- `BUFFER_COUNT` is fixed at 2 in `qrhid3d11.cpp`.
- Flip model is required for alpha: DirectComposition is used only when `QT_D3D_NO_FLIP` is unset.
- Qt passes only `D3D11_CREATE_DEVICE_DEBUG`. Other flags need our own device (option 5). With `SINGLETHREADED` it was −2 MB; with both flags −7 MB, not worth the thread risk.
- AMD's 25 driver threads per device stay whatever the flags.
- Variables to set after the hardening clear: `QSG_RENDER_LOOP=basic` and `QSG_NO_DEPTH_BUFFER=1`, beside the existing `QT_D3D_ADAPTER_INDEX`.

**5. Fonts.**
- Yes, they can load for the current theme only.
- `FontLoader` keeps every font in a process-wide cache forever (`qquickfontloader.cpp`), so theme switches pile up.
- Load each theme's files in C++ with `QFontDatabase::addApplicationFontFromData`, before the QML, and `removeApplicationFont` on a switch.
- Native needs no bundled font, since Segoe UI is found first.
- Shipping the families' official static files of the used weights instead of variable fonts cuts the cost per instance. They need `kx-licence-check`, and only unmodified files are used, so no Reserved Font Name issue.

## Recommendation
- **For the P3 ADR:** #1 + #2 + #4, with #5 optional.
  - **Expected idle (Native, harness):** about 105 MB PWS and 139 MB commit, 35 threads, about 1,750 handles. Flat under show and hide churn.
  - **Other themes:** ET66 about +6 MB and Dolch about +11 MB, or an estimated +2–4 with static weights (not measured).
- **Next, as its own item:** #3, the key diet. Expected about 86 MB PWS and 121 MB commit (floor measured: 85.6–87).
- **80 MB is not reachable with the GPU on this PC.**
  - The lowest shown with hardware rendering is 85.6 MB.
  - The real app also carries its Rust side, which was not measured here.
  - Only WARP (#6) went below it: 69 MB.
- **Owner decision needed**, either:
  - (a) set G8 to about 100 MB PWS and 140 MB commit, with no growth over 8 h; or
  - (b) fund #3, then re-measure, keeping #6 as an opt-in "low memory" setting.
- **In both cases, G8 should:**
  - log commit as well as PWS;
  - sample after a fixed warm-up;
  - never trim the working set.

## Risks
1. **Device loss** (driver update, TDR, sleep with a removed GPU).
   - With an adopted QRhi, the basic loop does not destroy it, and `createRhi` hands back the same lost one (`qsgrenderloop.cpp`, `handleDeviceLoss`).
   - The app must detect `isDeviceLost()`, then make a new QRhi and rebuild the windows. Rust holds all state, so a UI reload is possible, but it is new code and untested.
2. **One thread draws everything, at 60 fps.**
   - The basic loop drives animations from a 16 ms timer (Qt scene graph docs). We measured 62 fps per window against 124 fps threaded on this panel.
   - Every frame now renders on the GUI thread.
   - Re-run G2 (click to character), G12 (ring at display rate) and the owner's motion check.
   - Future low-level hooks must run on their own thread.
3. **The pinned and semi-public API, and memory that never comes back.**
   - `rhi/qrhi.h` needs the versioned private include path. Review it on every Qt upgrade.
   - With a shared device, a destroyed window's depth buffer is not given back; it is reused, with no growth over 30 cycles. #4 removes most of it.
   - The 80 MB budget still needs the owner's call.
- **Also:** `QSG_NO_DEPTH_BUFFER` needs a look check of every theme. Today no rotated item clips, so no stencil is needed.

## Sources
- No Qt bug matched "shared QRhi across windows on the basic loop". The evidence is Qt's 6.11 sources, the docs and the runs above.
- QQuickGraphicsDevice (`fromRhi` since 6.6, `fromRhiAdapter` since 6.10, `fromDeviceAndContext`): https://doc.qt.io/qt-6/qquickgraphicsdevice.html
- `QQuickWindow::setGraphicsDevice` (only before the scene graph starts) and `setPersistentGraphics`: https://doc.qt.io/qt-6/qquickwindow.html#setGraphicsDevice
- QRhi threading ("all usage must be limited to that one single thread"): https://doc.qt.io/qt-6/qrhi.html#threading
- Render loops and animation drivers (basic loop: 16 ms timer): https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html
- Depth buffer and `QSG_NO_DEPTH_BUFFER`: https://doc.qt.io/qt-6/qquickgraphicsconfiguration.html#setDepthBufferFor2D
- `createRhi` returns an adopted QRhi as not owned: https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/quick/scenegraph/qsgrhisupport.cpp?h=6.11
- Basic loop: per-window swap chain, `ownRhi`, device loss, no animation driver: https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/quick/scenegraph/qsgrenderloop.cpp?h=6.11
- Threaded loop releases the device on hide only when not persistent: https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/quick/scenegraph/qsgthreadedrenderloop.cpp?h=6.11
- D3D11 backend (device flags, `QT_D3D_NO_FLIP`, `BUFFER_COUNT`, DirectComposition): https://code.qt.io/cgit/qt/qtbase.git/tree/src/gui/rhi/qrhid3d11.cpp?h=6.11
- FontLoader's process-wide cache: https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/quick/util/qquickfontloader.cpp?h=6.11
- One DirectWrite face per font-set entry: https://code.qt.io/cgit/qt/qtbase.git/tree/src/gui/text/windows/qwindowsfontdatabasebase.cpp?h=6.11
- D3D11 device flags: https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_create_device_flag
- `SetWindowRgn` (nothing outside the region is drawn): https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowrgn
- `WM_NCHITTEST` and `HTTRANSPARENT` (same thread only): https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nchittest
- Layered-window hit testing: https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#layered-windows

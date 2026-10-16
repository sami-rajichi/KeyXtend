# 29. The scroll pad glides by wheel steps and reads by UI Automation

- **Date:** 2026-10-16
- **Status:** Accepted.

## Context

- Spec §5.2 listed three Windows routes in this order: UI Automation's `ScrollPattern`, a posted wheel message, then a `SendInput` wheel. P1's gate G6 found that all three scroll Notepad, Chrome and Explorer while the pointer is elsewhere, and that sideways scrolling worked only in Chrome (ADR-0013).
- The owner chose a smooth glide over notch steps (2026-10-16). `ScrollPattern.Scroll` moves only in fixed steps, and each call waits on the target app's UI thread.
- Windows documents wheel deltas below 120 (`WHEEL_DELTA`) for finer wheels: apps either scroll part of a line or add the deltas up to 120 ([WM_MOUSEWHEEL](https://learn.microsoft.com/windows/win32/inputdev/wm-mousewheel)). A posted message never waits on the app.
- The target is the last point the pointer was at outside the keyboard (§5.2). P4's hook runs only while a mouse mode is on (ADR-0027), so nothing knows that point when scrolling starts.
- The owner wants every feature built once for Windows, Linux and macOS (2026-10-16). Modules never call the OS (ADR-0005).

## Decision

Decided with the owner's approval of P5's design on 2026-10-16.

1. **A port.** `ScrollTarget` in `kx-module-api`, gated by its own `Scroll` capability, gives `kx-mod-scroll` three OS-neutral calls:
   - find the target now: the point, whether the pointer is on it, the box to outline, and whether it is one of our windows;
   - turn it by a number of lines (fractional) in one of four directions;
   - tell whether it can still move that way: yes, no, or unknown.

   The module holds every rule: taps, speeds, stops and timing, all on a fake clock in tests. Windows fills the port now; macOS (`CGEvent` scroll events, Accessibility for boxes) and Linux (XTest or the portal, AT-SPI) fill it in P15.
2. **Windows moves by wheel steps.** Each tick turns the target by a small wheel delta, converted from lines with the user's wheel setting (`SPI_GETWHEELSCROLLLINES`, `SPI_GETWHEELSCROLLCHARS`):
   - posted as `WM_MOUSEWHEEL` or `WM_MOUSEHWHEEL` to the window at the target point while the pointer is elsewhere;
   - sent as a `SendInput` wheel while the pointer itself is on the target;
   - not posted to a window Windows reports as hung, so steps never pile up.
3. **UI Automation reads, and moves only as a fallback.** On its own COM thread, with a short timeout, it finds the scrollable element at the point (walking up to the first `ScrollPattern`), its box for the outline, and its scroll percent for the end. When the percent stays still through `[scroll] check_ms` of wheel steps short of the end, that target moves by `ScrollPattern` line steps instead.
4. **The point before the keyboard.** When the pointer enters the keyboard window, the window guard reads Windows' recent pointer history (`GetMouseMovePointsEx`) and keeps the last point outside the window. If the history misses, the entry point is pushed out past the nearest edge. Nothing watches the pointer while it is elsewhere.
5. **The hook runs while scrolling.** The mouse module keeps its hook on while the pad scrolls (adds to ADR-0027's decision 4). That way the badge follows the pointer, and a press outside our windows stops the scrolling.
6. **The outline is four thin windows.** Each edge is a click-through Qt window just under the keyboard, so it costs a few kilobytes however large the target is (ADR-0021's one device).

## Consequences

- Spec §5.2 lists the routes in this order, and the roadmap and `ARCHITECTURE.md` follow.
- Apps that add deltas up to 120 still jump a notch's lines at a time; apps that scroll part of a line glide.
- The badge and click-to-stop need the mouse module. With it off, the pad still scrolls, outlines and stops by its own rules.
- On Wayland, where no global pointer history or posted event exists, P15 scrolls only what is under the pointer; the move map records that wall.
- G6 runs on the product and judges sideways scrolling where the content overflows sideways.

## Alternatives

- **UI Automation first (the old order):** steps instead of a glide, and every step waits on the app.
- **`SetScrollPercent` for a glide:** its precision depends on the app, some round to whole lines, and it skips the app's own smooth scrolling.
- **An always-on hook or pointer polling:** it would watch the mouse when nothing needs it.
- **One window covering the whole target for the outline:** its buffers cost megabytes on a full-screen target.

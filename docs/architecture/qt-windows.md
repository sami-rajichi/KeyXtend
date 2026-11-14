# Qt window rules (ADR-0013)

Moved from [ARCHITECTURE.md](../../ARCHITECTURE.md), which keeps the overview, the code map and the invariants.

- Pin Qt 6.11.2 and cxx-qt 0.10.0; review every upgrade.
- QML, fonts and icons are compiled into the binary as Qt resources; they are never loaded from disk.
- Keyboard windows are `Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus`; click-through windows add `Qt.WindowTransparentForInput`.
- The Windows adapter guards every window we create except Settings:
  - In `WM_STYLECHANGING`, it puts back `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`, and refuses click-through on a window that took clicks when first guarded.
  - In `WM_WINDOWPOSCHANGED`, it lifts a window that left the topmost band straight back, as `HWND_NOTOPMOST` never reaches `WM_STYLECHANGING`. After such a drop by another program, the keyboard comes back above a shown banner or list until a guard run names that pop-up as `top`.
  - It returns `MA_NOACTIVATE` for `WM_MOUSEACTIVATE`.
  - Every window it covers must start topmost (Qt's `WindowStaysOnTopHint`); guarding a plain hidden window can leave it below the topmost band.
- Never call `requestActivate()` on a no-focus window. Only the full Settings window takes focus, and it gives it back when it closes.
- Owned windows (shadows, bubble, tips, caption, ring) are declared inside their owner, so Qt keeps them above it. The selection strip sits inside its own shadow, which has no owner (`transientParent: null`), so it also shows by the bubble.
- The scroll outline's four edge windows have no owner (`transientParent: null`) and join the guard's quiet run, so `lay_under` can put them just under the keyboard each time a set shows or the keyboard shows again. They exist only while there is a box.
- A tip also lifts itself alone to the front as it shows (`SWP_NOOWNERZORDER`), since the guard may lift a banner or the list after its owner. A shown tip hides as a guard run starts, or the guard would lift its owner with it.
- A hidden window shows again at its old place, so a pop-up's guard run names it as `top` to land in front.
- Arabic flips text only: no window or control mirrors, and Arabic text reads right to left in the theme's Arabic font (ADR-0020).
- Every window draws on one shared graphics device, on the GUI thread (Qt's basic render loop) and without a depth buffer; a lost device, or a window that cannot draw, brings the windows back over the same hub, with the keyboard shown, closed or minimised as the user left it, up to `[ui] device_rebuilds` times, and Qt's loop runs between tries (ADR-0021).
- Only the shown theme's bundled fonts stay in Qt's font list: the hub loads them from the resources before the QML and again on each theme switch (ADR-0021).

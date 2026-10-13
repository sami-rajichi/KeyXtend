# 19. The tray icon: a port over the taskbar, and our own QML menu

- **Date:** 2026-09-30
- **Status:** Accepted. Adds the `Tray` port to the UI ports that ADR-0016 lists; ADR-0016 itself stays as accepted.

## Context

A hidden keyboard needs a way back, and the app needs a Quit, so it shows an icon in the taskbar's notification area (P3 plan, Task 18).

- Qt's tray icon (`QSystemTrayIcon`) lives in Qt Widgets: it needs the Widgets DLL and a `QApplication`, and its native menu takes the focus and opens on a right click.
- Every window of ours is Qt Quick and never takes the focus (ADR-0013), and no action may need a right click (ADR-0008).
- `kx-ui` reaches the OS only through the UI ports in `kx-platform` (ADR-0016).

## Decision

- **A `Tray` port** in `kx-platform`: the picture's side in pixels, `show` with a picture, a tooltip and a listener, and a `TrayIcon` whose `set` changes them and whose drop removes the icon. Events are a click with where the menu opens, and "another window came to the front".
- **The Windows adapter** (`WinTray`) calls `Shell_NotifyIconW` (version 4) from a hidden window's own thread. It adds, changes and deletes the icon from that thread's queue, never holding a lock during a taskbar call, and adds it again when the taskbar restarts. `kx-platform-fake` has a fake for tests.
- **The picture** is drawn by the UI in the theme (`TrayPicture.qml`) at the taskbar's size for its screen's DPI, and drawn again when the look or the scale changes.
- **The menu** is our own QML window (`TrayMenu.qml`): themed, non-activating, opened by a left click, Enter or Space, or the menu key. It closes on a pick, a second click on the icon, a press on any of our windows, and a new front window once `[ui] tray_grace_ms` has passed since it opened.

## Consequences

- No Qt Widgets DLL, and the menu looks and behaves like our other windows.
- The app must remove the icon before it ends (`Hub::tray_hide`), or a dead icon stays until the pointer passes over it.
- Windows 11 puts a new icon in the hidden icons at first; closing that pop-up can bring the last app to the front, which the grace time covers.
- A later Linux or macOS adapter implements the same port.

## Alternatives

- **`QSystemTrayIcon`:** the Widgets DLL, a native menu that takes the focus, and right-click habits; refused.
- **No tray icon, only the bubble:** a keyboard hidden with Close would have no way back and no Quit; refused.

# Platform notes

Moved from [ARCHITECTURE.md](../../ARCHITECTURE.md), which keeps the overview, the code map and the invariants.

| Concern | Windows (v1) | macOS (later) | Linux (later) |
|---|---|---|---|
| Top-most, no focus | uiAccess + `WS_EX_NOACTIVATE/TOPMOST/TOOLWINDOW` (guarded) | Qt `Tool` windows are non-activating `NSPanel`s | X11: `_NET_WM_STATE_ABOVE`; Wayland: LayerShellQt on KDE/wlroots; **not possible on GNOME Wayland** |
| Type | `SendInput` Unicode + VK | `CGEventPost` (Accessibility permission) | X11 XTest; Wayland RemoteDesktop portal / libei (asks the user once) |
| Mouse hook | `WH_MOUSE_LL` | `CGEventTap` | X11 XInput2; GNOME Wayland has no global hook, so we switch on GNOME's own "simulated secondary click" |
| Scroll | Wheel deltas posted, or `SendInput` while the pointer is on the target; UIA reads the box and end, and steps as the fallback (ADR-0029) | `CGEventCreateScrollWheelEvent`; Accessibility for boxes | XTest buttons 4–7 / portal; AT-SPI for boxes; Wayland only under the pointer |
| Secrets | DPAPI, tied to the signed-in account; no prompt (ADR-0010, ADR-0038) | Keychain | Secret Service |
| Focused box (P7) | UI Automation focus handler, called on UI Automation's threads; reads on the watch's COM thread; kinds from `field.toml` | Accessibility (`AXUIElement`) | AT-SPI |
| Word packs (P7) | A read-shared file, mapped `PAGE_READONLY` | A read-only `mmap` from a folder only the system writes, else a copy | The same as macOS |
| Clipboard (P8) | `AddClipboardFormatListener` on a message-only window, markers from `clip.toml`; Open folder through the running Explorer | `NSPasteboard`'s change count, polled; Finder | XFixes on X11, data-control on Wayland where the compositor has it; the desktop's file manager |
| Backup places (P9) | Cloud folders from `places.toml`, `DRIVE_REMOVABLE` drives and the folder list; a temp file, then a rename | (P15) | (P15) |
| Selection (P10) | UI Automation focus and text-selection events, line boxes with no text; the link through the running Explorer's `ShellExecute` | Accessibility (`AXSelectedTextRange`, `AXBoundsForRange`) (P15) | AT-SPI (`Text.GetSelection`, `GetRangeExtents`) (P15) |
| Hidden run (P9) | A probe file looked for in each package's `LocalCache\Local`, from `redirect.toml` | None known; P15 checks the App Sandbox | None known |
| Power dialog | `Shell.Application.ShutdownWindows` | loginwindow prompt | session-manager prompt |
| Shortcuts | `SendInput` from `shortcuts.toml`; Lock PC by `LockWorkStation` (ADR-0031) | (P15) | (P15) |
| Speech (P11) | The worker's `voice` job: the microphone through `cpal` (WASAPI), and `transcribe.cpp` with its processor parts and Vulkan card part loaded from the worker's own folder (ADR-0044) | The same jobs; the card part is Metal (P15) | The same jobs; the card part is Vulkan (P15) |
| Session lock and sleep (P11) | A hidden window on its own thread hears `WM_WTSSESSION_CHANGE` and `WM_POWERBROADCAST`, mapped by `session.toml` | Its own lock and sleep signals (P15) | Its own lock and sleep signals (P15) |
| Models folder (P11) | The fixed drives `places.toml` lists, a new folder made with protected rights in one step, SHA-256 by CNG, and Open through the running Explorer's `explore` verb (ADR-0046) | Its own file manager, under the same checks (P15) | Its own file manager, under the same checks (P15) |

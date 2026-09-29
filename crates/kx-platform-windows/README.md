# kx-platform-windows

The Windows adapter: the only product crate that calls Win32 (ARCHITECTURE.md invariant 3). It implements the ports from `kx-platform` and `kx-module-api` on Windows; the app picks it, and modules only ever see the ports.

The crate builds on every system. OS code is Windows-only, and the pure parts (QPC ticks to microseconds, the folder rule, the injection plan and its key tables) are tested everywhere.

## What it provides

| Part | Port | What it does |
|---|---|---|
| `WindowsPlatform` | `kx_platform::Platform` | Hands out the clock and the folders; `new` fails when Windows refuses either. `injector()` hands out the input injector for the app to register under `InputInjectorKey`. |
| `QpcClock` | `kx_module_api::Clock` | Microseconds since Windows started, from the performance counter; the time base `kx-gates` and `kx-target-window` log. |
| `dirs` | `kx_platform::AppDirs` | `exe_dir` is the running program's folder; `user_dir` is `APP_DIR_NAME` inside the Local AppData known folder, never read from environment variables. |
| `WinInjector` (`inject`) | `kx_module_api::ports::InputInjector` | Taps, chords and text for the app in front, in one `SendInput` batch each, every event tagged `keys::TAG`. Text goes as Unicode, or as key presses in the front app's layout (`VkKeyScanEx`). After a partial send it releases the keys still down. |
| `strokes` | none | The injection plan as plain key events (combos, Unicode units, what is still down), built and tested on every system. Its debug output never shows a key. |
| `keys` | none | The key tables: the tag, the virtual keys Windows sends as extended, and the `VkKeyScanEx` modifier bits and keys. |
| `front` | none | The app in front: its keyboard layout and its program's file name. |
| `inject::{send, send_keys, input, vk_key}` | none | The one `SendInput` path, shared with the test tools and the spike. |
| `com::Com` | none | COM start-up for one thread, undone when dropped, for UI Automation and the Windows Runtime. |
| `uiaccess::active` | none | Whether this process really runs with uiAccess. |

Later P3 tasks add layout reading and the window and system parts.

## Capabilities and settings

The adapter holds no capability of its own: the kernel grants capabilities to the modules that use its ports, and the injector's service key needs `InjectInput`. It has no settings; the app folder name is `kx_platform::APP_DIR_NAME`, and the apps that get key presses are the keyboard module's `key_apps` setting.

## Security

See `docs/security/kx-platform-windows.md`. Every `unsafe` block has a scoped `#[allow(unsafe_code, reason = "...")]` and a `// SAFETY:` comment. The injector never logs, and its errors carry counts, never text.

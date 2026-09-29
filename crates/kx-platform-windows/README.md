# kx-platform-windows

The Windows adapter: the only product crate that calls Win32 (ARCHITECTURE.md invariant 3). It implements the ports from `kx-platform` and `kx-module-api` on Windows; the app picks it, and modules only ever see the ports.

The crate builds on every system. OS code is Windows-only, and the pure parts (QPC ticks to microseconds, the folder rule, the injection plan and its key tables, the layout reading logic and layout ids) are tested everywhere.

## What it provides

| Part | Port | What it does |
|---|---|---|
| `WindowsPlatform` | `kx_platform::Platform` | Hands out the clock and the folders; `new` fails when Windows refuses either. `injector()` and `layouts()` hand out the input injector and the layout source for the app to register under `InputInjectorKey` and `LayoutSourceKey`. |
| `QpcClock` | `kx_module_api::Clock` | Microseconds since Windows started, from the performance counter; the time base `kx-gates` and `kx-target-window` log. |
| `dirs` | `kx_platform::AppDirs` | `exe_dir` is the running program's folder; `user_dir` is `APP_DIR_NAME` inside the Local AppData known folder, never read from environment variables. |
| `WinInjector` (`inject`) | `kx_module_api::ports::InputInjector` | Taps, chords and text for the app in front, in one `SendInput` batch each, every event tagged `keys::TAG`. Text goes as Unicode, or as key presses in the front app's layout (`VkKeyScanEx`, checked with the layout source's `ToUnicodeEx` under the current Caps Lock; Space follows a dead key), or not at all. After a partial send it tries to release the keys still down, with a mask key before Alt or Win. |
| `WinLayouts` (`layouts`) | `kx_module_api::ports::LayoutSource` | Every installed layout (Windows gives the count first, so there is no cap), what each key types per modifier state with `ToUnicodeEx` (which never changes the keyboard state), Caps Lock, and the layout of the window with the keyboard. `request` posts `WM_INPUTLANGCHANGEREQUEST` to that window and never switches anything itself; an uninstalled layout types nothing. The logic runs over a fake OS in tests. |
| `langinfo` | none | A layout's language facts: the ISO code and native name from the locale, right to left from its reading layout, and the layout's English name from the registry (a variant is found by its `Layout Id`). Reading layout ids and Windows' answers is pure. |
| `strokes` | none | The injection plan as plain key events (combos, Unicode units, what is still down), built and tested on every system. Its debug output never shows a key. |
| `keys` | none | The key tables: the tag, the virtual keys Windows sends as extended, the `VkKeyScanEx` modifier bits and keys, and the keys `ToUnicodeEx` reads held (Caps Lock, `AltGr`, left and right modifiers). |
| `front` | none | The app in front: its keyboard layout, the window that has its keyboard (`focus`, `focus_of`, `thread_layout`) and its program's file name. |
| `inject::{send, send_keys, input, vk_key}` | none | The one `SendInput` path, shared with the test tools and the spike. |
| `layouts::{id, hkl, ask, read_typed, is_printable}` | none | Layout ids to and from Windows handles, the switch request to a given window, and how a `ToUnicodeEx` answer reads, shared with the test tools and the spike. |
| `com::Com` | none | COM start-up for one thread, undone when dropped, for UI Automation and the Windows Runtime. |
| `uiaccess::active` | none | Whether this process really runs with uiAccess. |

Later P3 tasks add the window and system parts.

## Capabilities and settings

The adapter holds no capability of its own: the kernel grants capabilities to the modules that use its ports, and the injector's and layout source's service keys need `InjectInput`. It has no settings; the app folder name is `kx_platform::APP_DIR_NAME`, and the apps that get key presses are the keyboard module's `key_apps` setting.

## Security

See `docs/security/kx-platform-windows.md`. Every `unsafe` block has a scoped `#[allow(unsafe_code, reason = "...")]` and a `// SAFETY:` comment. The injector and the layout source never log; the injector's errors carry counts, never text, and a switch error names no layout.

# kx-platform-windows

The Windows adapter: the only product crate that calls Win32 (ARCHITECTURE.md invariant 3). It implements the ports from `kx-platform` and `kx-module-api` on Windows; the app picks it, and modules only ever see the ports.

The crate builds on every system. OS code is Windows-only, and the pure maths (QPC ticks to microseconds, the folder rule) is tested everywhere.

## What it provides

| Part | Port | What it does |
|---|---|---|
| `WindowsPlatform` | `kx_platform::Platform` | Hands out the clock and the folders; `new` fails when Windows refuses either. |
| `QpcClock` | `kx_module_api::Clock` | Microseconds since Windows started, from the performance counter; the time base `kx-gates` and `kx-target-window` log. |
| `dirs` | `kx_platform::AppDirs` | `exe_dir` is the running program's folder; `user_dir` is `APP_DIR_NAME` inside the Local AppData known folder, never read from environment variables. |
| `com::Com` | none | COM start-up for one thread, undone when dropped, for UI Automation and the Windows Runtime. |
| `uiaccess::active` | none | Whether this process really runs with uiAccess. |

Later P3 tasks add input injection, layout reading and the window and system parts.

## Capabilities and settings

The adapter holds no capability of its own: the kernel grants capabilities to the modules that use its ports. It has no settings; the app folder name is `kx_platform::APP_DIR_NAME`.

## Security

See `docs/security/kx-platform-windows.md`. Every `unsafe` block has a scoped `#[allow(unsafe_code, reason = "...")]` and a `// SAFETY:` comment.

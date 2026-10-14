# kx-mod-mouse

The mouse module: Right-click by holding still, Grab, and modifier + click (spec §5.1). Its hold engine runs on the platform's pointer hook, which decides each mouse event on the hook's thread (ADR-0027), so the module never talks to the OS.

## Services

| Service | Direction | Use |
|---|---|---|
| `PointerHook` (`PointerHookKey`) | needs | Runs the hold engine on a system-wide pointer hook, only while a mode, a latched modifier or a carry needs it. |

It provides no service; it talks to the keyboard and the UI over the bus (`kx_module_api::ports` assist events).

## Capabilities

| Capability | Why |
|---|---|
| `PointerHook` | It gates the pointer hook the engine runs on. |
| `InjectInput` | The engine sends clicks and modifier keys, and it gates the toggle and latch events the module hears and sends. |

## Settings (`[mouse]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `rclick` | `false` | Right-click mode; the module writes it on each toggle, so it survives a restart. Grab is never saved. |
| `hold_ms` | `1500` | A still press right-clicks after this, in ms; within `HOLD_MS` (0.5 s to 3.0 s, spec §5.1). |
| `still_px` | `7` | Moves up to this many logical pixels, scaled by the screen's DPI, still count as still; within `STILL_PX`. |
| `ring_after_ms` | `150` | The ring shows this long after the press, in ms; within `RING_AFTER_MS`, and always below the shortest hold. |
| `health_ms` | `1000` | How often the health check runs while the hook runs, in ms; within `HEALTH_MS`. |
| `health_slack_ms` | `500` | How long the hooks may miss input before they are put back, in ms; within `HEALTH_SLACK_MS`. |
| `echo_ms` | `250` | How long input waits behind our own before it goes on anyway, in ms; within `ECHO_MS`. |

The ranges are named constants in `src/settings.rs`; the defaults live only in `defaults.toml`. `MouseSettings::hook_config` hands the last three to the hook's host.

## Public API

`MouseModule`, `ID`, `MANIFEST`, `MouseSettings` (with `from_defaults` and `hook_config`), its `SPEC`, and the pure hold engine: `Engine`, `EngineCfg` (built `From<&MouseSettings>`), `Note`, `ESC`, `MAX_LATCHES` and `MAX_ACTS` (the room the host reserves for one call's input). `Actor` and `Msg` are the runtime the module's thread runs, open so tests drive it without a thread; `STOPPED_LINE` is the message of the line it logs with the hook's numbers at each stop, which kx-gates reads.

## Security

See `docs/security/kx-mod-mouse.md`. The module never logs a point, a key or a scan code.

# kx-mod-scroll

The scroll module: the round pad scrolls what the user last pointed at, gliding at two speeds until a stop (spec §5.2). It moves targets only through the platform's `ScrollTarget` port (ADR-0029), so the module never talks to the OS.

## Services

| Service | Direction | Use |
|---|---|---|
| `ScrollTarget` (`ScrollTargetKey`) | needs | Finds the target where the user pointed, turns it by fractions of lines and says whether it can go further. |

It provides no service; it talks to the keyboard and the UI over the bus (`kx_module_api::ports` scroll events).

## Capabilities

| Capability | Why |
|---|---|
| `Scroll` | It gates the scroll target, which moves other apps' windows. |

The pad's events (`ScrollTap`, `ScrollStop`, `ScrollPeek`, `ScrollAsk`) and the mouse module's `PointerPressed` need no capability.

## Settings (`[scroll]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `slow` | `3` | The first tap's speed, in lines a second; within `SPEED` (1 to 60). |
| `fast` | `10` | The second tap's speed, in lines a second; within `SPEED`, and always above `slow`. |
| `limit_s` | `120` | Scrolling stops this many seconds after the last pad tap; `0` means no limit, any other value is within `LIMIT_S` (10 s to 30 min). |
| `stop_at_end` | `true` | Scrolling stops when the target reaches its end. |
| `stop_on_press` | `true` | Scrolling stops at a press outside our windows. |
| `tick_ms` | `16` | How often the glide steps while scrolling, in ms; within `TICK_MS`, and always below `check_ms`, so a check spans more than one tick. |
| `check_ms` | `250` | How long a target may stay still under our steps, short of its end, before the platform tries another way, in ms; within `CHECK_MS`. |
| `read_ms` | `300` | The longest one read of a target may take, in ms; within `READ_MS`. |
| `edge_px` | `8` | How far past the keyboard's nearest edge the target point goes when the pointer's path is unknown, in physical pixels; within `EDGE_PX`. |

The ranges are named constants in `src/settings.rs`; the defaults live only in `defaults.toml`. `ScrollSettings::scroll_config` hands the last three to the platform.

## Public API

`ScrollModule`, `ID`, `MANIFEST`, `ScrollSettings` (with `from_defaults` and `scroll_config`), its `SPEC`, and the pure rules: the tap machine `Pad` with its `Change`, the glide `Glide` with `GlideCfg`, and the stop rules `Stops` (both built `From<&ScrollSettings>`) with their `Halt`.

## Security

See `docs/security/kx-mod-scroll.md`. The module never logs a point or a target's box.

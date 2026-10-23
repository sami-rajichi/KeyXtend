# kx-mod-power

The power module: a click on the Power key opens the system's own power dialog, where the user picks what to do (spec §8.5, ADR-0031). It reaches the OS only through the platform's `PowerDialog` port, and never shuts down, restarts or confirms anything itself.

## Services

| Service | Direction | Use |
|---|---|---|
| `PowerDialog` (`PowerDialogKey`) | needs | Opens the system's power dialog. |

It provides no service; it talks to the keyboard and the UI over the bus (`kx_module_api::ports` power events).

## Capabilities

| Capability | Why |
|---|---|
| `Power` | It gates the power dialog; only this module holds it. |

The events it hears (`PowerAsked`, `PowerAsk`) and sends (`PowerChanged`, the `notice.power-failed` notice) need no capability.

## Settings

None: the module has nothing to configure, so its manifest owns no section.

## Public API

`PowerModule`, `ID` and `MANIFEST`. The runtime is the `Actor`, driven by `Msg` on the `kx-power` thread; each click opens the dialog on its own short `kx-power-open` thread, through a `Spawn` that tests replace to run each `Job` in place. A stop never waits on an open, and an answer that comes after it tells nothing.

## Security

See `docs/security/kx-mod-power.md`.

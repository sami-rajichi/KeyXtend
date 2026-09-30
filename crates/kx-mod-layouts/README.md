# kx-mod-layouts

The layouts module: it reads the keyboard layouts installed in the OS and provides the `Layouts` service, which gives the keyboard the table of the layout it shows, the language list and the language switch. It talks to the OS only through the `LayoutSource` port, so it builds and tests on every system over `kx-platform-fake`.

## Services

| Service | Direction | Use |
|---|---|---|
| `LayoutSource` (`LayoutSourceKey`) | Requires | The installed layouts, the layout of the app in front, what each key types, the language facts, and the switch request. |
| `Layouts` (`LayoutsKey`) | Provides | `current` (the table of the layout shown), `list` (every installed layout's label, in the OS order), `refresh`, `step` and `pick`. |

## Capabilities

| Capability | Why |
|---|---|
| `InjectInput` | It gates the layout source, whose switch request acts on the app in front, and the layouts service this module provides. |

## Settings (`[layouts]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `short` | `{ ar = "ع" }` | What the language key shows for a language, by its ISO 639 code (lower-case letters); a language not listed shows its code in capitals. A short name may not be blank. |

The defaults live only in `defaults.toml`.

## Security

See `docs/security/kx-mod-layouts.md`.

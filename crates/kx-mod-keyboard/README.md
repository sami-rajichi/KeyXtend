# kx-mod-keyboard

The keyboard module: the board the user clicks, its size and its place on the screen, and later typing, sticky modifiers and the language key. It talks to the OS only through ports, so it builds and tests on every system.

## Services

| Service | Direction | Use |
|---|---|---|
| None yet | — | The `KeyboardFace` it provides and the `Layouts` and `InputInjector` it needs arrive with typing and the language key (P3 Tasks 12 and 13). |

## Capabilities

| Capability | Why |
|---|---|
| None yet | It only reads its settings. Typing will need `InjectInput`, which gates `Layouts` and `InputInjector`. |

## Settings (`[keyboard]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `modifiers` | `"release"` | Latched modifiers let go after the next key (`"release"`) or stay until clicked again (`"stay"`). |
| `lang_key` | `"carousel"` | How the language key shows the layouts: `"carousel"`, `"cycle"` or `"list"`. |
| `size` | `1.0` | The keyboard's size as a factor of the size-1 board; the module writes it when the user resizes. |
| `presets` | `[1.0, 1.25, 1.5]` | The S, M and L sizes; at least one, each between `size_min` and `size_max`. |
| `size_step` | `0.05` | Change per smaller or bigger click; above 0. |
| `size_min`, `size_max` | `0.8`, `1.8` | The smallest and largest sizes; `size_min` is above 0 and below `size_max`. |
| `relabel_ms` | `500` | How often the labels follow the layout of the app in front, in ms; above 0. |
| `key_apps` | `[]` | Apps that get text as key presses in their own layout, by file name; no blank names. |
| `x`, `y`, `placed` | `0`, `0`, `false` | Where the user last put the keyboard, in physical pixels; the module writes them, and they hold only once `placed` is true. |
| `bubble_corner` | `"right"` | The bottom corner of the keyboard's screen where the minimise bubble goes: `"right"` or `"left"`. |

Every size is a finite number. The defaults live only in `defaults.toml`.

## Public API

`KeyboardModule`, `ID`, `MANIFEST`, `SPEC`, `KeyboardSettings` (with `from_defaults` and `place`), `Modifiers`, `LangKey` and `Corner`.

## Security

See `docs/security/kx-mod-keyboard.md`. The module never logs.

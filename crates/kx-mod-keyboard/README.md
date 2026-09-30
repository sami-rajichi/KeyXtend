# kx-mod-keyboard

The keyboard module: the board the user clicks, its size and its place on the screen, and later typing, sticky modifiers and the language key. It talks to the OS only through ports, so it builds and tests on every system.

## Services

| Service | Direction | Use |
|---|---|---|
| None yet | — | The `KeyboardFace` it provides and the `Layouts` and `InputInjector` it needs arrive with typing and the language key (P3 Tasks 12 and 13). |

## Capabilities

| Capability | Why |
|---|---|
| None yet | It reads and writes only its own settings. Typing will need `InjectInput`, which gates `Layouts` and `InputInjector`. |

## The board

`board.toml` holds the grid sizes, the five main rows (key ids, widths in key units, kinds, scan codes in hex and roles), the side block and the D-pad. It is compiled in and checked when read: every row spans the grid, every key is whole grid columns, ids are unique and each role fits its key. Key names, labels and icons are not in the board; translations (Task 15) and `kx-theme` (Task 14) give them by key id.

| Role | A click |
|---|---|
| `char` | Types the character of the layout shown; only the keys of a `chars` run have it. |
| `send` | Sends its scan code with the latched modifiers. |
| `caps` | Sends Caps Lock and lights while it is on. |
| `latch` | Latches Shift, Ctrl, Alt, AltGr or Win. |
| `lang` | Steps through or picks the layouts. |
| `later` | Nothing: the side block and D-pad come in a later version and are drawn dimmed. |

## Geometry, size and place

- `board` lays the board out at any size as boxes in logical pixels (`Area`), with the plate, D-pad and top bar.
- `Limits` clamps a size to `size_min..=size_max`, steps it by `size_step` and names the presets; `Sizer` applies them and saves `size`.
- `place` keeps the keyboard wholly on one screen in physical pixels, sized at that screen's DPI. It starts at the saved place, else at the bottom centre of the primary work area; a place off every screen goes there too. `Placer` saves `x`, `y` and `placed` when the user moves the keyboard.

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

Every size is a finite number. The defaults live only in `defaults.toml`. The module writes `size`, `x`, `y` and `placed` only through its own settings handle, and only when they change.

## Public API

- `KeyboardModule`, `ID`, `MANIFEST`, `SPEC`, `KeyboardSettings` (with `from_defaults` and `place`), `Modifiers`, `LangKey`, `Corner`.
- `spec`: `BoardSpec` (`shipped`, `read`, `columns`, `row_keys`, `char_scans`), `BoardError`, `KeyKind`, `Latch`, `Role`, `KeySpec`, `CharSpec`, `RowItem`, `RowKey`, `SideSpec`, `DpadSpec`, `char_id`.
- `geom`: `board`, `Board`, `KeyBox`, `Area`.
- `sizer`: `Limits`, `Resize`, `Sizer`, `SizeError`.
- `place`: `Screen`, `Placer`, `inside`, `bottom_centre`, `home`, `fit`, `start`, `bubble`.

## Security

See `docs/security/kx-mod-keyboard.md`. The module never logs.

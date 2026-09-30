# kx-mod-keyboard

The keyboard module: the board the user clicks, its size and its place on the screen, typing with sticky modifiers, and later the language key. It talks to the OS only through ports, so it builds and tests on every system.

## Services

| Service | Direction | Use |
|---|---|---|
| `InputInjector` | needs | Sends each click's keys or text to the app in front, and names that app for `key_apps`. |
| `LayoutSource` | needs | Says whether Caps Lock is on, and whether a key typed anything while a dead key waits. |
| `Layouts` | needs | Its current table says what each key types in the layout shown. |
| `KeyboardFace` | provides, later | Arrives with the language key (P3 Task 13); it holds the ports and a `Typing`. |

## Capabilities

| Capability | Why |
|---|---|
| `InjectInput` | It gates all three services typing needs; the module sends input only through `InputInjector`. |

## The board

`board.toml` holds the grid sizes, the five main rows (key ids, widths in key units, kinds, scan codes in hex and roles; 48 character keys), the side block and the D-pad. Row 4 is ISO, with the key 0x56 (FR `< >`, AR `|`) after a short left Shift, as on real French keyboards. It is compiled in and checked when read: every row spans the grid, every key is whole grid columns, ids are unique and each role fits its key. A character key's id is `c-` and its scan code, such as `c-0x002C`, so adding or moving a key renames no other key. Key names, labels and icons are not in the board; translations (Task 15) and `kx-theme` (Task 14) give them by key id.

| Role | A click |
|---|---|
| `char` | Types the character of the layout shown; only the keys of a `chars` run have it. |
| `send` | Sends its scan code with the latched modifiers. |
| `caps` | Sends Caps Lock and lights while it is on. |
| `latch` | Latches Shift, Ctrl, Alt, `AltGr` or Win. |
| `lang` | Steps through or picks the layouts. |
| `later` | Nothing: the side block and D-pad come in a later version and are drawn dimmed. |

## Geometry, size and place

- `board` lays the board out at any size as boxes in logical pixels (`Area`), with the plate, D-pad and top bar.
- `Limits` clamps a size to `size_min..=size_max`, steps it by `size_step` and names the presets; `Sizer` applies them and saves `size`.
- `place` keeps the keyboard wholly on one screen in physical pixels, sized at that screen's DPI. The screens are the ports' `Screen`, as `FaceAction::Placed` carries them. It starts at the saved place, else at the bottom centre of the primary work area; a place off every screen goes there too. `Placer` saves `x`, `y` and `placed` when the user moves the keyboard.

## Typing

`Typing::tap(io, id, back)` plans one click from the key's role, the current layout table, the latches, Caps Lock, a waiting dead key and the app in front (`plan`), then sends it (`run`). Characters go as Unicode text, so the result never depends on the app's layout (ADR-0009); keys that need the app's own layout go as scan codes.

| Click | Route | Latches | Dead-key wait |
|---|---|---|---|
| A character key with Ctrl, Alt or Win latched, or `AltGr` on a layout without `AltGr` characters (Right Alt) | Chord: every latch held around its scan code | Used | Kept only if the key typed nothing |
| A character key whose character is a dead key | Tap, or a chord of the latches its legend counts | Used | Starts |
| The character key after a dead key | Tap, or a chord of the latches its legend counts, so the app adds the accent | Used | Ends |
| A character key that types nothing in the layout shown | Its scan code, as above | Used | Kept only if the key typed nothing |
| Any other character key | Its text as Unicode, or as key presses in the app's layout (`TextRoute::Keys`) when the app is in `key_apps`; Shift, Caps Lock and `AltGr` are already in the text | Used, none held | None |
| A `send` key | Tap, or a chord with every latch held | Used | Kept only if the key typed nothing |
| Caps Lock | Tap | Kept | Kept |
| A modifier | Nothing; it latches or lets go | Toggled | Kept |
| The language key, a `later` key | Nothing; `Tapped::Lang` or `Tapped::Later` for the face | Kept | Kept |

"Used" means `release` lets every latch go and `stay` keeps them. `key_apps` names match the app's file name without case; every UWP app reports `ApplicationFrameHost.exe`. A send that fails changes nothing: the latches and the wait stay. Legends come from the same table and latches (`legend`, `LegendMods::of`), so a key types what it shows.

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
| `key_apps` | `[]` | Apps that get text as key presses in their own layout, by file name, matched without case; no blank names. |
| `x`, `y`, `placed` | `0`, `0`, `false` | Where the user last put the keyboard, in physical pixels; the module writes them, and they hold only once `placed` is true. |
| `bubble_corner` | `"right"` | The bottom corner of the keyboard's screen where the minimise bubble goes: `"right"` or `"left"`. |

Every size is a finite number. The defaults live only in `defaults.toml`. The module writes `size`, `x`, `y` and `placed` only through its own settings handle, and only when they change.

## Public API

- `KeyboardModule`, `ID`, `MANIFEST`, `SPEC`, `KeyboardSettings` (with `from_defaults` and `place`), `Modifiers`, `LangKey`, `Corner`.
- `spec`: `BoardSpec` (`shipped`, `read`, `columns`, `row_keys`, `char_scans`, `roles`), `BoardError` (`kind`, and `From` into a start error), `KeyKind`, `Latch`, `Role`, `KeySpec`, `CharSpec`, `RowItem`, `RowKey`, `SideSpec`, `DpadSpec`, `char_id`.
- `latch`: `Latches` (`new`, `mode`, `toggle`, `is_on`, `held`, `used`).
- `legend`: `Legend`, `LegendMods` (`of`), `legend`, `typed`, `slot`, `shown`, `is_arabic`, `is_haraka`, `is_mark`.
- `plan`: `plan`, `Plan`, `Route` (its debug output hides keys and text), `Wait`, `Now`, `text_route`.
- `run`: `Typing` (`new`, `tap`, `latches`, `waiting`), `Tapped`, `Io`, `Ports` (`from_cx`, `io`).
- `geom`: `board`, `Board`, `KeyBox`, `Area`.
- `sizer`: `Limits`, `Resize`, `Sizer`, `SizeError`.
- `place`: `Placer`, `inside`, `bottom_centre`, `home`, `fit`, `start`, `bubble`.

## Security

See `docs/security/kx-mod-keyboard.md`. The module never logs; typed text travels only as `Redacted<String>`.

# kx-mod-keyboard

The keyboard module: the board the user clicks, its size and its place on the screen, typing with sticky modifiers, the language key, and the face the UI draws. It talks to the OS only through ports, so it builds and tests on every system.

## Services

| Service | Direction | Use |
|---|---|---|
| `InputInjector` | needs | Sends each click's keys or text to the app in front, and names that app for `key_apps`. |
| `LayoutSource` | needs | Says whether Caps Lock is on, and whether a key typed anything while a dead key waits. |
| `Layouts` | needs | Its current table says what each key types in the layout shown. |
| `KeyboardFace` | provides | The board and state JSON the UI draws, and each click it forwards; see "The face". |

## Capabilities

| Capability | Why |
|---|---|
| `InjectInput` | It gates all three services typing needs; the module sends input only through `InputInjector`. |

## The board

`board.toml` holds the grid sizes, the five main rows (key ids, widths in key units, kinds, scan codes in hex and roles; 48 character keys), the side block and the D-pad. Row 4 is ISO, with the key 0x56 (FR `< >`, AR `|`) after a short left Shift, as on real French keyboards. It is compiled in and checked when read: every row spans the grid, every key is whole grid columns, ids are unique and each role fits its key. A character key's id is `c-` and its scan code, such as `c-0x002C`, so adding or moving a key renames no other key. Key names, labels and icons are not in the board; translations (Task 15) and `kx-theme` (Task 14) give them by key id. A key's `nav` says what it does in an open list.

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
| A character key missing from the table shown | Tap, or a chord with every latch held around its scan code | Used | Kept only if the key typed nothing |
| Any character key sent as a scan code while the app in front has another installed layout | Nothing is sent; `Tapped::Stale`, so the face refreshes the layouts and redraws, and the user clicks again | Kept | Kept |
| Any other character key | Its text as Unicode, or as key presses in the app's layout (`TextRoute::Keys`) when the app is in `key_apps`; Shift, Caps Lock and `AltGr` are already in the text | Used, none held | None |
| A `send` key | Tap, or a chord with every latch held | Used | Kept only if the key typed nothing |
| Caps Lock | Tap | Kept | Kept |
| A modifier | Nothing; it latches or lets go | Toggled | Kept |
| The language key, a `later` key | Nothing; `Tapped::Lang` or `Tapped::Later` for the face | Kept | Kept |

"Used" means `release` lets every latch go and `stay` keeps them; `Typing::update` takes a new `modifiers` or `key_apps` and keeps what is latched. `key_apps` names match the app's file name without case; every UWP app reports `ApplicationFrameHost.exe`. A send refused whole changes nothing. One the system took in part moves the latches and the wait as if it all went, returns the error for a notice and is never retried. Legends come from the same table and latches (`legend`, `LegendMods::of`), so a key types what it shows.

## The face

`Face` is the `KeyboardFace` service the UI draws. The UI calls it from one thread, which also runs the relabel timer, so `Layouts::refresh` is only ever called from there. The module keeps its face across a settings restart, so the latches survive; `Face::renew` takes the new settings (`Typing::update`, `Sizer`, `Placer`).

`board_json()` changes only with the size or the language key's design:

- `size`, `preset`, `presets`, `relabel_ms`, `lang_key`, `bubble_corner`, and the `plate`, `bar` and `dpad` boxes (`x y w h`, logical pixels).
- `keys`, each `id kind row side x y w h icon label name checkable dim tip`. The box is the key's cell; the UI draws the cap half the theme's `gap_px` (times the size) inside it.
- `icon` is an `icons.toml` key (`keys.<id>`, `lang.cycle`, `lang.list`); the UI shows `label` when the theme has no such icon. `name`, `label` and `tip` are translation keys (`key.<id>`, `label.<id>`, `tip.later`).
- A `later` key and the D-pad come with `dim: true` and the tip "Coming in a later version".

`state_json()` changes as keys are used:

- `keys` by id: `main second third main_ar second_ar third_ar name marks on paint`. Marks with no letter before them sit on a dotted circle (◌) in every legend and in `name`. `marks` names the harakat a key types by translation key (`mark.fatha`), for readers; `paint` names the `fill`, `ink` and `skirt` palette tokens (`dim` for a later key).
- `lang`: `prev`, `cur` and `next` as `{short rtl arabic}` (null when the neighbour is the current layout), `at` and `count` for the Cycle's dots, the space bar's `space` parts, and `turn` (-1, 0 or 1) for the Carousel's slide.
- `list`: null, or `{rows, hi, keys, page_back, page_on}` while the List is open; each row is `{short name rtl arabic current icon}`, and the current row's `icon` is `lang.current`. `page_back` and `page_on` name the board keys with the moves `PAGE_BACK` and `PAGE_ON` (null when the board has none), which the list's page buttons send as `panel_key`; the UI dims a button on the first or last page.
- `at`: null, or where the window goes in physical pixels.

`lang()` names the language of the layout shown (`LangLabel::iso`), which the UI's keyboard texts follow (ADR-0020); it only reads, so any thread may ask.

`act(action)` says what to redraw in a `FaceReply`, with a notice key for the banner above the keyboard:

| Action | Does | Reply |
|---|---|---|
| `tap {id, back}` | Types, latches, or clicks the language key | State after typing, a latch or a List click. Nothing for a Carousel or Cycle step (the switch redraws on `relabel`), a later-version key, or a list key while the list is open. A failed send adds `notice.keys-not-sent`, and a refused step `notice.lang-not-switched`. A stale click sends nothing, refreshes the layouts once and redraws; the user clicks again. |
| `relabel` | Refreshes the layouts | State when the layout changed. |
| `panel_key {key, rows}` | Drives the open list with a board key that has a `nav`, at `rows` rows per page (`[ui] page_rows`) | State when it moved. |
| `lang_pick {index}` | Picks a list row and closes the list | State; `NoChoice` past the list. |
| `smaller`, `bigger`, `preset`, `resize_begin`, `resize_at`, `resize_end` | Sizes the keyboard; the size is saved in one write | Board; `notice.settings-unsaved` when it could not be saved. |
| `placed {rect, screens}` | Fits the window to the screens the UI reports; a move is saved | State when the place changed. |

The language key has three designs, by `lang_key`:

- **Carousel:** the back part steps back, the rest steps on; the names slide the way it stepped once the switch lands, and a state read in the change event already has the slide.
- **Cycle:** every click steps on; `at` lights one dot of `count`, and the names never slide (`turn` stays 0).
- **List:** a click opens the list on the current row, and another closes it. While it is open, the board's `nav` keys (Esc, Enter and the arrows) come as `panel_key` and never reach the app; every other key, Space too, types as usual. Up and Down move one row, Left and Right one page, Enter picks and Esc closes.

A step or pick the app refuses shows `notice.lang-not-switched`. Steps and the Carousel's neighbours wrap with `kx_module_api::ports::step_to`, the rule `Layouts::step` uses.

The golden files in `tests/golden/` pin both JSONs for EN, FR, AR, AR with `AltGr` latched and the open list. `KX_BLESS=1` rewrites them after a deliberate change; the QML fixtures (Task 17) are made from them.

## Settings (`[keyboard]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `modifiers` | `"release"` | Latched modifiers let go after the next key (`"release"`) or stay until clicked again (`"stay"`). |
| `lang_key` | `"carousel"` | How the language key shows the layouts: `"carousel"`, `"cycle"` or `"list"`. |
| `size` | `1.0` | The keyboard's size as a factor of the size-1 board; the module writes it when the user resizes. |
| `presets` | `[1.0, 1.25, 1.5]` | The S, M and L sizes: one to the face port's `MAX_PRESETS` (one per size button the top bar names), each between `size_min` and `size_max`. |
| `size_step` | `0.05` | Change per smaller or bigger click; above 0. |
| `size_min`, `size_max` | `0.8`, `1.8` | The smallest and largest sizes; `size_min` is above 0 and below `size_max`. |
| `relabel_ms` | `500` | How often the labels follow the layout of the app in front, in ms; within `RELABEL_MS` in `settings.rs`, so a hand-edited value cannot stall them. |
| `key_apps` | `[]` | Apps that get text as key presses in their own layout, by file name, matched without case; no blank names. |
| `x`, `y`, `placed` | `0`, `0`, `false` | Where the user last put the keyboard, in physical pixels; the module writes them, and they hold only once `placed` is true. |
| `bubble_corner` | `"right"` | The bottom corner of the keyboard's screen where the minimise bubble goes: `"right"` or `"left"`. |

Every size is a finite number. The defaults live only in `defaults.toml`. The module writes `size`, `x`, `y` and `placed` only through its own settings handle, and only when they change.

## Public API

- `KeyboardModule` (keeps its face across restarts), `ID`, `MANIFEST`, `SPEC`, `KeyboardSettings` (with `from_defaults` and `place`), `Modifiers`, `LangKey`, `Corner`.
- `face`: `Face` (`new`, `renew`), the `KeyboardFace` service.
- `view`: `board_view`, `state_view`, `BoardView`, `KeyView`, `DpadView`, `StateView`, `KeyState`, `StateInput`, `Paint`, `paint`, `DIM`, and the name prefixes `NAME_PREFIX`, `LABEL_PREFIX`, `ICON_PREFIX`, `LATER_TIP`, `DPAD_NAME`, `CYCLE_ICON`, `LIST_ICON`.
- `langkey`: `click`, `LangClick`, `lang_view`, `LangView`, `LangName`, `Turn`.
- `panel`: `LangList`, `Moved`, `navs`, `list_view`, `ListView`, `LangRow`, `CURRENT_ICON`.
- `spec`: `BoardSpec` (`shipped`, `read`, `columns`, `row_keys`, `char_scans`, `roles`), `BoardError` (`kind`, and `From` into a start error), `KeyKind`, `Latch`, `Role`, `KeySpec` (with `nav`), `Nav`, `CharSpec`, `RowItem`, `RowKey`, `SideSpec`, `DpadSpec`, `char_id`.
- `latch`: `Latches` (`new`, `mode`, `set_mode`, `toggle`, `is_on`, `held`, `used`).
- `legend`: `Legend`, `LegendMods` (`of`), `legend`, `typed`, `slot`, `shown`, `is_arabic`, `is_haraka`, `is_mark`, `mark_names`, `MARK_PREFIX`, `HARAKA_NAMES`.
- `plan`: `plan`, `Plan`, `Route` (its debug output hides keys and text), `Wait`, `Now`, `text_route`.
- `run`: `Typing` (`new`, `update`, `tap`, `latches`, `waiting`), `Tapped`, `TapError`, `Io`, `Ports` (`from_cx`, `io`).
- `geom`: `board`, `Board`, `KeyBox`, `Area`.
- `sizer`: `Limits`, `Resize`, `Sizer`, `SizeError`.
- `place`: `Placer`, `inside`, `bottom_centre`, `home`, `screen_of`, `fit`, `start`; the bubble's spot is `kx-ui`'s, as it needs theme sizes.

## Security

See `docs/security/kx-mod-keyboard.md`. The module never logs; typed text travels only as `Redacted<String>`.

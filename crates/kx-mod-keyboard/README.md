# kx-mod-keyboard

The keyboard module: the board the user clicks, its size and its place on the screen, typing with sticky modifiers, the language key, and the face the UI draws. It talks to the OS only through ports, so it builds and tests on every system.

## Services

| Service | Direction | Use |
|---|---|---|
| `InputInjector` | needs | Sends each click's keys or text to the app in front, and names that app for `key_apps`. |
| `LayoutSource` | needs | Says whether Caps Lock is on, and whether a key typed anything while a dead key waits. |
| `Layouts` | needs | Its current table says what each key types in the layout shown. |
| `KeyboardFace` | provides | The board and state JSON the UI draws, the board's row the theme's drawn check measures caps by, and each click it forwards; see "The face". |

## Capabilities

| Capability | Why |
|---|---|
| `InjectInput` | It gates all three services typing needs, and the latch and toggle events of mouse assist; the module sends input only through `InputInjector`. |

## The board

`board.toml` holds the grid sizes, the five main rows (key ids, widths in key units, kinds, scan codes in hex and roles; 48 character keys), the side block and the D-pad. Row 4 is ISO, with the key 0x56 (FR `< >`, AR `|`) after a short left Shift, as on real French keyboards. It is compiled in and checked when read: every row spans the grid, every key is whole grid columns, ids are unique and each role fits its key. A character key's id is `c-` and its scan code, such as `c-0x002C`, so adding or moving a key renames no other key. Key names, labels and icons are not in the board; the `kx-ui` translations (`i18n/*.ftl`) and `kx-theme` (`tokens/icons.toml`) give them by key id. A key's `nav` says what it does in an open list. A `send` key with `repeat = true` repeats while held (Backspace, Delete, Enter, Space and the arrows), as every character key does; `BoardSpec::repeating` lists them, and any other key with the flag is refused.

| Role | A click |
|---|---|
| `char` | Types the character of the layout shown; only the keys of a `chars` run have it. |
| `send` | Sends its scan code with the latched modifiers. |
| `caps` | Sends Caps Lock and lights while it is on. |
| `latch` | Latches Shift, Ctrl, Alt, `AltGr` or Win. |
| `lang` | Steps through or picks the layouts. |
| `assist` | Asks the mouse module to flip Right-click (`rclick`) or Grab (`grab`); see "Mouse assist on the bus". |
| `later` | Nothing: the other side keys come in a later version and are drawn dimmed. |

## Geometry, size and place

- `board` lays the board out at any size as boxes in logical pixels (`Area`), with the plate, D-pad and top bar.
- `Limits` clamps a size to `size_min..=size_max`, steps it by `size_step` and names the presets; `Sizer` applies them and saves `size`.
- `place` keeps the keyboard wholly on one screen in physical pixels, sized at that screen's DPI. The screens are the ports' `Screen`, as `FaceAction::Placed` carries them. It starts at the saved place, else at the bottom centre of the primary work area; a place off every screen goes there too. `Placer` saves `x`, `y` and `placed` when the user moves the keyboard.
- The keyboard always fits the screen it is on, so the size buttons at the top bar's right never leave it. `room` gives the largest size that fits, `Sizer::fit` caps the size there without saving (below `size_min` if it must), and `Placer::keep` moves the window back inside after a size change, saving nothing.

## Typing

`Typing::tap(io, id, back)` plans one click from the key's role, the current layout table, the latches, Caps Lock, a waiting dead key and the app in front (`plan`), then sends it (`run`). Characters go as Unicode text, so the result never depends on the app's layout (ADR-0009); keys that need the app's own layout go as scan codes.

| Click | Route | Latches | Dead-key wait |
|---|---|---|---|
| A character key with Ctrl, Alt or Win latched, or `AltGr` on a layout without `AltGr` characters (Right Alt) | Chord: every latch held around its scan code | Used | Kept only if the key typed nothing |
| A character key whose character is a dead key | Tap, or a chord of the latches its legend counts | Used | Starts |
| The character key after a dead key | Tap, or a chord of the latches its legend counts, so the app adds the accent | Used | Ends |
| A character key that types nothing in the layout shown, with neither Shift nor `AltGr` latched | Its scan code, as above | Used | Kept only if the key typed nothing |
| An empty key: a character key that types nothing with the latched Shift or `AltGr`, and no Ctrl, Alt or Win | Nothing; `Tapped::Empty`, as Windows' own keyboard does | Kept | Kept |
| A character key missing from the table shown | Tap, or a chord with every latch held around its scan code | Used | Kept only if the key typed nothing |
| Any character key sent as a scan code while the app in front has another installed layout | Nothing is sent; `Tapped::Stale`, so the face refreshes the layouts and redraws, and the user clicks again | Kept | Kept |
| Any other character key | Its text as Unicode, or as key presses in the app's layout (`TextRoute::Keys`) when the app is in `key_apps`; Shift, Caps Lock and `AltGr` are already in the text | Used, none held | None |
| A `send` key | Tap, or a chord with every latch held | Used | Kept only if the key typed nothing |
| Caps Lock | Tap | Kept | Kept |
| A modifier | Nothing; it latches or lets go | Toggled | Kept |
| The language key, a `later` key, a mouse-assist key | Nothing; `Tapped::Lang`, `Tapped::Later` or `Tapped::Assist` for the face | Kept | Kept |
| An id not on the board | Nothing; `TapError::UnknownKey`, which the face answers as `FaceError::UnknownKey` | Kept | Kept |

"Used" means `release` lets every latch go and `stay` keeps them; `Typing::update` takes a new `modifiers` or `key_apps` and keeps what is latched. `key_apps` names match the app's file name without case; every UWP app reports `ApplicationFrameHost.exe`. A send refused whole changes nothing. One the system took in part moves the latches and the wait as if it all went, returns the error for a notice and is never retried. Legends come from the same table and latches (`legend`, `LegendMods::of`), so a key types what it shows. Caps Lock that types nothing falls back to the same state without it; Shift or `AltGr` never fall back, so such a key shows no main or second legend and types nothing (`is_empty`, one rule for the plan and the view). A Ctrl, Alt, Win or Right Alt latch makes no key empty, as its click sends the shortcut. A key whose base is an Arabic letter shows no `AltGr` corner, which would crowd the letter; it shows its `AltGr` character when `AltGr` is latched. A layout cell holds only visible text: the adapter reads a control or U+0000 as empty.

## The face

`Face` is the `KeyboardFace` service the UI draws. The UI calls it from one thread, which also runs the relabel timer, so `Layouts::refresh` is only ever called from there. The module keeps its face across a settings restart, so the latches survive; `Face::renew` takes the new settings (`Typing::update`, `Sizer`, `Placer`) and drops a held key, ending its burst. A held key's times come from the kernel's clock (`Ports::clock`), so tests move a fake clock and never sleep.

`board_json()` changes only with the size or the settings, such as the language key's design or key repeat:

- `size`, `preset`, `presets`, `relabel_ms`, `repeat_delay_ms`, `repeat_every_ms`, `lang_key`, `bubble_corner`, and the `plate`, `bar` and `dpad` boxes (`x y w h`, logical pixels); `dpad` also has its `name`, `key.dpad`.
- `keys`, each `id kind row side x y w h icon label name checkable dim tip repeat`. The box is the key's cell; the UI draws the cap half the theme's `gap_px` (times the size) inside it. `repeat` marks a key that repeats while held, so the UI sends `press` and `let_go` for it; with `repeat` off in the settings, no key has it.
- `icon` is an `icons.toml` key (`keys.<id>`, `lang.cycle`, `lang.list`); the UI shows `label` when the theme has no such icon. `name`, `label` and `tip` are translation keys (`key.<id>`, `label.<id>`, `tip.later`).
- A `later` key comes with `dim: true` and the tip "Coming in a later version". `checkable` marks the latches, Caps Lock and the mouse-assist keys.

`state_json(rows)` changes as keys are used; `rows` is the UI's page size (`[ui] page_rows`):

- `keys` by id: `main second third main_ar second_ar third_ar main_mark second_mark third_mark name marks blank on dim paint tip`. Marks with no letter before them sit on a dotted circle (◌) in every legend and in `name`; `*_mark` flags such a legend, which the UI draws in the theme's mark font. `marks` names the harakat a key types by translation key (`mark.fatha`), for readers, and `blank` is `key.empty` while an empty key shows nothing; `dim` is true while the key does nothing: a later key, or a mouse-assist key while the mouse module is not live; `paint` names the `fill`, `ink` and `skirt` palette tokens (`dim` paint for such a key). `tip` is a translation key the key says after its name: `tip.one-lang` on the language key while one layout, or none, is installed, and `tip.rclick`, `tip.grab` or `tip.mouse-off` on the mouse-assist keys.
- `lang`: `prev`, `cur` and `next` as `{short rtl arabic}` (null when the neighbour is the current layout), `at` and `count` for the Cycle's dots, the space bar's `space` parts, and `turn` (-1, 0 or 1) for the Carousel's slide.
- `list`: null, or `{rows, hi, page, keys, page_back, page_on}` while the List is open; each row is `{short name rtl arabic current icon}`, and the current row's `icon` is `lang.current`. `page` is `{at count back on}`: the page holding `hi` from 0, how many pages, and whether one lies before or after it; `LangList::page` is the only page maths, so the UI and its page turns read it. `page_back` and `page_on` name the board keys with the moves `PAGE_BACK` and `PAGE_ON` (null when the board has none), which the list's page buttons send as `panel_key`; the UI dims a button when `page` says no page lies that way.
- `at`: null, or where the window goes in physical pixels.
- `dpad`: `{dim tip dir fast}`. While the scroll module is not live it is `dim` with `tip.scroll-off`, `dir` null and `fast` false; while live, `tip` is null, `dir` is the way it scrolls (`up`, `down`, `left`, `right`, or null while still) and `fast` its speed.

`lang()` names the language of the layout shown (`LangLabel::iso`), which the UI's keyboard texts follow (ADR-0020); it only reads, so any thread may ask.

`act(action)` says what to redraw in a `FaceReply`, with a notice key for the banner above the keyboard and, for a press or a tick, whether the key is still held (`holding`), so the UI keeps its beat only while it is:

| Action | Does | Reply |
|---|---|---|
| `tap {id, back}` | Types, latches, or clicks the language key; a burst left open ends first, so its latches never ride on the click | State after typing, a latch or a List click. Nothing for a Carousel or Cycle step (the switch redraws on `relabel`), a later-version key, a mouse-assist key (it publishes `AssistToggle` while the mouse module is live), or a list key while the list is open. A failed send adds `notice.keys-not-sent`, and a refused step `notice.lang-not-switched`. A stale click sends nothing, refreshes the layouts once and redraws; the user clicks again. |
| `press {id}` | The pointer went down on a key (spec §4.6): it types now, as `tap` with the front part. It holds, with the latches kept down until its burst ends, when `repeat` is on, no list is open, no Ctrl, Alt or Win (or `AltGr` acting as Right Alt) acts on it, it is not a dead key (`Typing::repeats`) and it typed; else it acts once. While the list is open, a key that drives it waits for `panel_key` | As `tap`, with `holding` when it holds. `UnknownKey` off the board, which changes nothing. |
| `tick` | The hold timer: types the held key at `repeat_delay_ms`, then once every `repeat_every_ms`, with the latches kept down (`Typing::hold`); a late tick types once. A repeat that is stale, empty, fails, or turns into a shortcut or dead key ends the burst | `holding` while the key is still held, due or not. A repeat that went out redraws the state only when its burst ended, as the latches stay latched until then; else as a tap: a stale refresh, or `notice.keys-not-sent` once. |
| `let_go {id}` | The pointer let go, wherever it is: the key typed at the press, so nothing more types, and the burst's latches let go (`Typing::release`); a let-go of a key not held changes nothing | State when a burst ended. `UnknownKey` off the board, which changes nothing. |
| `leave` | Qt took the press away, or the keyboard hid: the burst ends and nothing more types | State when a burst ended. |
| `relabel` | Refreshes the layouts | State when the layout changed, Caps Lock changed outside the keyboard, or the bus changed what the state shows since it was last read. |
| `panel_key {key, rows}` | Drives the open list with a board key that has a `nav`, at `rows` rows per page (`[ui] page_rows`) | State when it moved. |
| `pad {dir}` | An arrow publishes `ScrollTap`, and Stop (`dir` null) `ScrollStop`, while the scroll module is live; else nothing | Nothing; the module's answer redraws on `relabel`. |
| `pad_hover {on}` | The pointer went onto the pad or off it: each change publishes `ScrollPeek` while the scroll module is live | Nothing. |
| `lang_pick {index}` | Picks a list row and closes the list | State; `NoChoice` past the list. |
| `smaller`, `bigger`, `preset`, `resize_begin`, `resize_at`, `resize_end` | Sizes the keyboard within the room on its screen; the size is saved in one write | Board, and state when the window moved to stay on its screen; `notice.settings-unsaved` when it could not be saved. |
| `placed {rect, screens}` | Fits the window and its size to the screens the UI reports; a move is saved | Board when the size had to fit its screen, state when the place changed. |

The language key has three designs, by `lang_key`:

- **Carousel:** the back part steps back, the rest steps on; the names slide the way it stepped once the switch lands, and a state read in the change event already has the slide.
- **Cycle:** every click steps on; `at` lights one dot of `count`, and the names never slide (`turn` stays 0).
- **List:** a click opens the list on the current row, and another closes it. Opening it ends a held key's burst. While it is open, the board's `nav` keys (Esc, Enter and the arrows) come as `panel_key` and never reach the app; every other key, Space too, types as usual. Up and Down move one row, Left and Right one page, Enter picks and Esc closes.

With one layout installed, a click asks no step and opens no list in any design, but still closes a list left open, and the key says so through its state's `tip`. A step or pick the app refuses shows `notice.lang-not-switched`. Steps and the Carousel's neighbours wrap with `kx_module_api::ports::step_to`, the rule `Layouts::step` uses.

The golden files in `tests/golden/` pin both JSONs for EN, FR, AR, AR with Shift or `AltGr` latched, and the open list. `KX_BLESS=1` rewrites them after a deliberate change; the `kx-ui` QML test fixtures are made from them.

## Mouse assist on the bus

The keyboard never depends on the mouse module; they talk only through `kx_module_api::ports::assist` events, so each starts without the other.

- **Side keys:** `rclick` and `grab` light from `AssistChanged` (Grab while armed or carrying). Until the mouse module says it is `live`, or after it stops, they are `dim` with `tip.mouse-off`, and a click does nothing.
- **Toggles:** a live click publishes `AssistToggle`; it sends nothing to the app and keeps the latches.
- **Start:** the module subscribes, then publishes its latched set and `AssistAsk`, so neither side keeps what it missed while the other was stopped.
- **Latches:** every change of the latched set publishes `LatchesChanged` with `Latches::held`, in its fixed order; `LatchesAsk` publishes it again. `LatchesUsed` (a click used them) lets them go as `modifiers` says, publishing only a change.
- **Senders:** the sets go out one sender at a time, newest last, from the UI thread or the mouse module's; a thread that finds another sending leaves its request to it.
- **Redraw:** a change from the bus marks the state unshown, so the next `relabel` answers state until the UI reads it; a click's own changes redraw at once, as before.

## The scroll pad on the bus

The keyboard never depends on the scroll module either; they talk only through `kx_module_api::ports::scroll` events.

- **State:** the pad follows `ScrollChanged`; a change marks the state unshown, as the mouse module's does. The keyboard asks for it with `ScrollAsk` at start.
- **Clicks and hover:** `pad` and `pad_hover` publish as above; the UI never publishes them itself. A module that comes live while the pointer rests on the pad hears `ScrollPeek` at once, and one that stops forgets it.
- **One sender:** peeks go out one thread at a time through their own `Outbox` (the latched set has another), each re-reading the pointer and the module's run, so the last peek the module hears is where the pointer is, even after a quick restart.
- **Our list:** the keyboard ignores `ScrollPage`; `kx-ui`'s hub turns it into the list's own page-button `panel_key`, as only the hub knows `[ui] page_rows`.

Handlers run on the sender's thread and take the face's lock briefly; nothing is published while it is held, so a handler may call back into the face.

## Settings (`[keyboard]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `modifiers` | `"release"` | Latched modifiers let go after the next key (`"release"`) or stay until clicked again (`"stay"`). |
| `lang_key` | `"carousel"` | How the language key shows the layouts: `"carousel"`, `"cycle"` or `"list"`. |
| `size` | `1.0` | The keyboard's size as a factor of the size-1 board; the module writes it when the user resizes. |
| `presets` | `[1.0, 1.25, 1.5]` | The S, M and L sizes, rising: one to the face port's `MAX_PRESETS` (one per size button the top bar names), each between `size_min` and `size_max`. |
| `size_step` | `0.05` | Change per smaller or bigger click; above 0, no wider than `size_max - size_min`, and with a finite reciprocal. |
| `size_min`, `size_max` | `0.8`, `1.8` | The smallest and largest sizes; `size_min` is below `size_max`, and both stay within `SIZE_FLOOR` (every key cell 24 px across) and `SIZE_CEILING` in `settings.rs`. |
| `relabel_ms` | `500` | How often the labels follow the layout of the app in front, in ms; within `RELABEL_MS` in `settings.rs`, so a hand-edited value can neither stall them nor poll for nothing. |
| `repeat` | `true` | Held typing keys repeat (spec §4.6); with `false`, a held key types once, at the press. |
| `repeat_delay_ms` | `800` | How long a key is held before it repeats, in ms; within `REPEAT_DELAY_MS` in `settings.rs`, so a slow click never repeats. |
| `repeat_every_ms` | `100` | The time between repeats, in ms; within `REPEAT_EVERY_MS` in `settings.rs`. |
| `key_apps` | `["scrcpy.exe"]` | Apps that get text as key presses in their own layout, by file name, matched without case; no blank names. scrcpy reads letters only as key presses. |
| `x`, `y`, `placed` | `0`, `0`, `false` | Where the user last put the keyboard, in physical pixels; the module writes them, and they hold only once `placed` is true. |
| `bubble_corner` | `"right"` | The bottom corner of the keyboard's screen where the minimise bubble goes: `"right"` or `"left"`. |

Every size is a finite number. The defaults live only in `defaults.toml`. The module writes `size`, `x`, `y` and `placed` only through its own settings handle, and only when they change.

## Public API

- `KeyboardModule` (keeps its face across restarts), `ID`, `MANIFEST`, `SPEC`, `KeyboardSettings` (with `from_defaults` and `place`), `Modifiers`, `LangKey`, `Corner`.
- `face`: `Face` (`new`, `renew`), the `KeyboardFace` service.
- `view`: `board_view`, `state_view`, `BoardView`, `KeyView`, `DpadView`, `StateView`, `KeyState`, `StateInput`, `PadState`, `pad_state`, `Paint`, `paint`, `DIM`, and the name prefixes `NAME_PREFIX`, `LABEL_PREFIX`, `ICON_PREFIX`, `LATER_TIP`, `LONE_TIP`, `MOUSE_OFF_TIP`, `SCROLL_OFF_TIP`, `RCLICK_TIP`, `GRAB_TIP`, `DPAD_NAME`, `EMPTY_NAME`, `CYCLE_ICON`, `LIST_ICON`.
- `langkey`: `click`, `lone`, `LangClick`, `lang_view`, `LangView`, `LangName`, `Turn`.
- `panel`: `LangList`, `Moved`, `navs`, `list_view`, `ListView`, `LangRow`, `CURRENT_ICON`.
- `spec`: `BoardSpec` (`shipped`, `read`, `columns`, `row_keys`, `char_scans`, `roles`, `repeating`), `BoardError` (`kind`, and `From` into a start error), `KeyKind`, `Latch`, `Assist` (into `Toggle`), `Role`, `KeySpec` (with `nav` and `repeat`), `Nav`, `CharSpec`, `RowItem`, `RowKey`, `SideSpec`, `DpadSpec`, `char_id`.
- `latch`: `Latches` (`new`, `mode`, `set_mode`, `toggle`, `is_on`, `held`, `used`); its debug output shows only the mode, and `LegendMods`, `KeyState` and `StateView` hide latches and Caps Lock too.
- `legend`: `Legend`, `LegendMods` (`of`), `legend`, `typed`, `slot`, `is_empty`, `shown`, `is_haraka`, `is_mark`, `mark_names`, `MARK_PREFIX`, `HARAKA_NAMES`; a legend takes the Arabic font by `kx_module_api::is_arabic`, the rule the layouts module names languages by.
- `plan`: `plan`, `Plan`, `Route` (its debug output hides keys and text), `Wait`, `Now`, `text_route`.
- `repeat`: `Repeat` (`new`, `press`, `tick`, `leave`, `held`), the hold of one key, timed by the kernel's clock (`Ports::clock`); its debug output shows only its phase.
- `run`: `Typing` (`new`, `update`, `tap`, `latches`, `clicked`, `waiting`, and for a held key `repeats`, `hold`, `release`), `Tapped`, `TapError`, `Io`, `Ports` (`from_cx`, `io`, with the kernel's `clock` and `bus`).
- `geom`: `board`, `Board`, `KeyBox`, `Area`.
- `sizer`: `Limits`, `Resize`, `Sizer` (with `fit`), `SizeError`.
- `place`: `Placer` (with `keep`), `inside`, `bottom_centre`, `home`, `screen_of`, `landing`, `room`, `fit`, `start`; the bubble's spot is `kx-ui`'s, as it needs theme sizes.

## Security

See `docs/security/kx-mod-keyboard.md`. The module never logs; typed text travels only as `Redacted<String>`.

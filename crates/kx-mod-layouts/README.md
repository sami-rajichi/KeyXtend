# kx-mod-layouts

The layouts module: it reads the keyboard layouts installed in the OS and provides the `Layouts` service, which gives the keyboard the table of the layout it shows, the language list and the language switch. It talks to the OS only through the `LayoutSource` port, so it builds and tests on every system over `kx-platform-fake` and its golden EN, FR and AR tables.

## Services

| Service | Direction | Use |
|---|---|---|
| `LayoutSource` (`LayoutSourceKey`) | Requires | The installed layouts, the layout of the app in front, what each key types, the language facts, and the switch request. |
| `Layouts` (`LayoutsKey`) | Provides | `current`, `list`, `refresh`, `step` and `pick`, below. |
| `LayoutChanged` (event) | Publishes | The keyboard now shows another installed layout, with its language code and direction; sent by `refresh` and `tell`, never by start-up. |
| `LayoutAsk` (event) | Hears | A module that started late asks for the shown layout; `tell` sends it again, or nothing while none is installed. |

## How it behaves

- **Shown layout (`Follow`).** The app in front's layout while it is installed; else the one shown, while still installed; else the first installed. A layout in front that is not installed, or no app in front, keeps the one shown.
- **`current`.** The shown layout's `LayoutTable`, read once per layout and cached until that layout is uninstalled. It holds only keys that type something, among the scan codes in `scans.toml` (compiled in); an empty table while nothing is installed.
- **`list`.** One `LangLabel` per installed layout, in the source's order (the OS order, as the Windows language bar steps), each layout once.
- **`refresh`.** Reads the installed list and the app in front again; true when either the list or the shown layout changed. The keyboard face calls it from its one thread, which runs the relabel timer and the clicks, so the events stay in order; an event's id is a hint, and listeners read `current`. `refresh` and `tell` tell one change at a time, so a `LayoutChanged` handler must not publish `LayoutAsk`.
- **`step` and `pick`.** They only ask the app in front to switch (`LayoutSource::request`); `current` changes when a later `refresh` sees the switch. `step(by)` wraps both ways along the list from the shown layout (`kx_module_api::ports::step_to`, which the language key also uses), and asks nothing when it lands on the shown layout (`by` 0, or one layout installed); `pick` refuses a layout not in the list with `NotInstalled` and asks nothing. The source's errors come back unchanged, and the shown layout stays.
- **Labels (`label`).** `short` comes from the `short` setting by ISO code, else the code in capitals; `name` holds the language's own name, then the layout's, leaving out an unknown one (the UI joins them); `iso` is the ISO code in lower case, which the keyboard's texts follow (ADR-0020); `rtl` comes from the facts; `arabic` is true when the language's own name holds a letter from the Unicode blocks of Arabic letters (`kx_module_api::is_arabic`, the keyboard's legend rule too), so no language list is kept.

## Capabilities

| Capability | Why |
|---|---|
| `InjectInput` | It gates the layout source, whose switch request acts on the app in front, and the layouts service this module provides. |

## Settings (`[layouts]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `short` | `{ ar = "ع" }` | What the language key shows for a language, by its ISO 639 code (two or three lower-case letters, `ISO_LETTERS`); a language not listed shows its code in capitals. A short name may not be blank, and holds at most `MAX_SHORT_CHARS` characters so it fits on the key. A user's map replaces the default one. |

The defaults live only in `defaults.toml`. The space bar's joiner is not a setting: the UI joins the name parts with a translated text.

## Public API

`LayoutsModule`, `ID`, `MANIFEST`, `LayoutsSettings` (with `from_defaults`), `SPEC`, `LayoutsService`, `Follow`, `label`, `scans` and `ScanTableError`, and the pure `logic` modules (`follow`, `label`, `order`).

## Security

See `docs/security/kx-mod-layouts.md`. The module never logs, and it holds no typed text: only what keys type in each layout and the language names.

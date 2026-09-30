# kx-ui

The UI: the texts in English, French and Arabic, the `[ui]` settings, the `ui` module and the hub it shares with the app, and, with the `qt` feature (on by default), the QML module `KeyXtend.Ui` and its cxx-qt bridges (ADR-0013). It depends only on `kx-module-api`, `kx-platform` and `kx-theme`, plus Qt and Fluent.

## Build

The `qt` feature needs Qt 6.11.2: set `QMAKE` to its `qmake.exe` and put its `bin` first in `PATH` for the command. `build.rs` lists every QML file in one table (`QML`, the root first) and every bridge in `BRIDGES`, and hands the root's resource URL to the crate as `MAIN_QML`. Without the feature (`--no-default-features`) the texts, settings, hub and module still build and test, as the Linux checks need.

## The `[ui]` settings

`defaults.toml` holds the values and `settings.rs` the limits of each number. The check refuses an unknown theme, a language that is neither `auto` nor in `LANGS`, a `min_contrast` outside `kx_theme::contrast::MIN_CONTRAST` (never below WCAG AA), and a timing or row count outside its range.

## The hub

The app makes one `Hub` over its UI ports before any window exists, so it can note the user's focus, and gets a `Receiver<Request>` with it.

- **Bridges:** `install(hub)` hands it to the QML objects once, before QML loads, since QML makes them without arguments.
- **The `ui` module:** `UiModule` needs the keyboard face; its start hands the hub the checked settings, the face and its settings handle, and its stop lets the face go.
- **Notices:** `listen(&bus)` hears every notice. Equal notices show once, texts are made in the UI language when the banner takes them, and only a failed module's notice offers Try again.
- **Requests:** Try again sends `Request::Retry(module)` and Quit sends `Request::Quit`; the app drains them on its own thread, calls `Kernel::retry` or stops the modules, then ends the UI with `command(Command::Quit)`.

## Threads

Every bridge call runs on the Qt GUI thread, including the guard and the window shape. The system look watch, the single-instance commands and the bus may call the hub from other threads: the hub then calls the waker the `Ui` object set, which queues the change to the Qt thread and never blocks. Commands that come before the `Ui` object exists wait in the hub. The hub never holds its lock during a face, port or waker call.

## QML objects

- `Ui`: settings and look JSON, texts, the space bar's text, notices (take, Try again, OK), the theme button, the light or dark switch, the backdrop and Quit, with signals for each change and for the app's Show and Quit.
- `Keyboard`: the board and state JSON, every click and size step, the language list, the window's place, the pointer, the bubble's spot, the window shape and the no-focus guard.

## Public API

`Hub`, `HubError`, `UiPorts`, `Request`, `Wake`, `Waker`, `Redraw`, `Resize`, `Shown`, `UiSettings`, `Backdrop`, `SPEC`, `UiModule`, `ID`, `MANIFEST`, `Area`, the i18n API (`I18n`, `Lang`, `LANGS`, `AUTO`, `find`, `pick`), and with `qt` also `install` and `MAIN_QML`.

## Security

See `docs/security/kx-ui.md`. The crate root denies unsafe code; only `src/bridges/` allows it, for the cxx-qt glue, with a `// SAFETY:` comment on each block (tidy F3).

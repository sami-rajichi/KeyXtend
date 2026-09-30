# kx-ui

The UI: the texts in English, French and Arabic, the `[ui]` settings, the `ui` module and the hub it shares with the app, and, with the `qt` feature (on by default), the QML module `KeyXtend.Ui` and its cxx-qt bridges (ADR-0013). It depends only on `kx-module-api`, `kx-platform` and `kx-theme`, plus Qt and Fluent.

## Build

The `qt` feature needs Qt 6.11.2: set `QMAKE` to its `qmake.exe` and put its `bin` first in `PATH` for the command. `build.rs` lists every QML file in one table (`QML`, the root first) and every bridge in `BRIDGES`, and hands the root's resource URL to the crate as `MAIN_QML`. Without the feature (`--no-default-features`) the texts, settings, hub and module still build and test, as the Linux checks need.

It also compiles in, as Qt resources, the icons the theme tokens name and every font under `assets/fonts/` (ADR-0016), found through `kx_theme::assets`; a missing icon file stops the build. `ASSETS` hands QML their URLs as JSON, and the app never reads them from disk.

## QML views

`qml/` holds the keyboard window and its parts, moved from the P1 test build with its look (Task 17). `main.qml` makes one `Ui` and one `Keyboard`, the keyboard window (`KbWindow`, its body `Plate`) and the minimise bubble, each over a click-through shadow window (`Glow`). The windows read `settingsJson`, `lookJson`, `boardJson` and `stateJson`, redraw from each action's `{board, state}` answer, and read everything again on `settingsChanged`.

- Every window is a no-focus tool window (ADR-0013); once shown, it is cut to its plate and corner, then guarded until the guard is done.
- Names, labels and tips are translation keys, looked up in the UI language; the Arabic UI mirrors the top bar, never the keys.
- Keys that come in a later version are dimmed, never press, and tell readers and the tooltip "Coming in a later version".
- Text uses `Label` (grayscale, no ClearType fringes), and hover uses `Hover` (the theme's tint, and a `Highlight` outline in high contrast). `Icon` tints with `MultiEffect`, public Qt Quick only.

## Qt Quick tests

`qmltest/` checks the views over stand-ins for `Ui` and `Keyboard` (`FakeUi`, `FakeKeyboard`), because `qmltestrunner` cannot load a module linked into a Rust binary. They answer from `qmltest/fixtures/` (written by the texts test from the hub and the face's golden files) and from kx-theme's look fixtures, so QML never reads a stale value. Run them with Qt's `bin` in `PATH` and `QML_DISABLE_DISK_CACHE=1`: `qmltestrunner -input crates/kx-ui/qmltest`.

## The `[ui]` settings

`defaults.toml` holds the values and `settings.rs` the limits of each number. The check refuses an unknown theme, a language that is neither `auto` nor in `LANGS`, a `min_contrast` outside `kx_theme::contrast::MIN_CONTRAST` (never below WCAG AA), and a timing or row count outside its range.

## The hub

The app makes one `Hub` over its UI ports before any window exists, so it can note the user's focus, and gets a `Receiver<Request>` with it.

- **Bridges:** `install(hub)` hands it to the QML objects once, before QML loads, since QML makes them without arguments (ADR-0018). An object made before it warns and does nothing.
- **The `ui` module:** `UiModule` needs the keyboard face; its start hands the hub the checked settings, the face and its settings handle, and its stop lets the face go.
- **Notices:** `listen(&bus)` hears every notice. Equal notices show once, texts are made in the UI language when the banner takes them, and only a failed module's notice offers Try again.
- **Requests:** Try again sends `Request::Retry(module)`. Quit, from the user (`Ui.quit`) or a second copy (`command(Command::Quit)`), sends `Request::Quit` once and then ends the queue, so the app's drain loop stops after it. The app calls `Kernel::retry`, or stops the modules and then `end_ui`. When the drain thread is gone, Quit ends the UI itself. Every request is sent under the hub's lock, so none lands after Quit.
- **A second copy's Show** wakes the UI through `command(Command::Show)`.

## Threads

Every bridge call runs on the Qt GUI thread, including the guard and the window shape. The system look watch, the single-instance commands and the bus may call the hub from other threads: the hub then calls the waker the `Ui` object set, which queues the change to the Qt thread and never blocks. Show and End wakes that come before the `Ui` object exists wait in the hub; a second `Ui` object takes the wakes and warns. The hub never holds its lock during a face, port or waker call.

## QML objects

- `Ui`: settings and look JSON (the settings also carry the size buttons' letters and names), texts, the space bar's text, notices (take, Try again, OK), the theme button, the light or dark switch, the backdrop, the compiled-in assets and Quit, with signals for each change and for the app's Show and Quit.
- `Keyboard`: the board and state JSON, every click and size step, the language list, the window's place, the pointer, the bubble's spot, the window shape and the no-focus guard.

## Public API

`Hub`, `HubError`, `UiPorts`, `Request`, `Wake`, `Waker`, `Redraw`, `Resize`, `Shown`, `UiSettings`, `Backdrop`, `SPEC`, `UiModule`, `ID`, `MANIFEST`, `Area`, the i18n API (`I18n`, `Lang`, `LANGS`, `AUTO`, `find`, `pick`), and with `qt` also `install`, `MAIN_QML` and `ASSETS`.

## Security

See `docs/security/kx-ui.md`. The crate root denies unsafe code; only `src/bridges/` allows it, for the cxx-qt glue, with a `// SAFETY:` comment on each block (tidy F3).

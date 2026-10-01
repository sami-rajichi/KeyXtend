# kx-ui

The UI: the texts in English, French and Arabic, the `[ui]` settings, the `ui` module and the hub it shares with the app, and, with the `qt` feature (on by default), the QML module `KeyXtend.Ui` and its cxx-qt bridges (ADR-0013). It depends only on `kx-module-api`, `kx-platform` and `kx-theme`, plus Qt and Fluent.

## Build

The `qt` feature needs Qt 6.11.2: set `QMAKE` to its `qmake.exe` and put its `bin` first in `PATH` for the command. `build.rs` lists every QML file in one table (`QML`, the root first) and every bridge in `BRIDGES`, and hands the root's resource URL to the crate as `MAIN_QML`. Without the feature (`--no-default-features`) the texts, settings, hub and module still build and test, as the Linux checks need.

`cpp/` holds the C++ the bridges call where cxx-qt-lib has no binding; `build.rs` adds it to the include path and rebuilds when it changes.

It also compiles in, as Qt resources, the icons the theme tokens name and every font under `assets/fonts/` (ADR-0016), found through `kx_theme::assets`; a missing icon file stops the build. `ASSETS` hands QML their URLs as JSON, and the app never reads them from disk.

## Qt's start

The app calls `harden_qt` before Qt's app object exists. It clears with `qunsetenv` every name with a Qt prefix (`harden::QT_PREFIXES`, any case) in the C runtime's narrow copy of the environment, which is what Qt reads; a name the code page narrows to a Qt one, such as a fullwidth `ＱＴ_`, goes too. It also turns off Qt Quick's shader cache on disk, and `qt_sees` tells whether Qt still reads a variable. In the uiAccess build the app also passes its folder, so Qt loads plugins only from there, and calls `pin_imports` on the engine before loading: QML modules then come only from the compiled-in ones and the `qml` folder beside the app (`QML_DIR`).

## QML views

`qml/` holds the keyboard window and its parts, moved from the P1 test build with its look (Task 17). `main.qml` makes one `Ui` and one `Keyboard` and hands them to `App`, which holds the keyboard window (`KbWindow`, its body `Plate`), the minimise bubble, the notice banner (`Notice`) and the tray menu (`TrayMenu`), each over a click-through shadow window (`Glow`), and draws the tray icon's picture (`TrayPicture`). The windows read `settingsJson`, `langJson`, `lookJson`, `boardJson` and `stateJson`, redraw from each action's `{board, state}` answer, read everything again on `settingsChanged`, and only the texts on `textsChanged`.

- **Banner:** notices queue; one shows above the keyboard (or at the bottom of the main screen while it is away) with Try again when the app can restart what failed, and OK, and says how many more wait. A new keyboard language tells the queued notices again (`shownNotices`).
- **Tray (ADR-0019):** × hides the keyboard; a click on the tray icon opens the themed menu (Show keyboard, Quit) above it, in the system's language (`trayText`, ADR-0020). It closes on a pick, a second click on the icon, a press on any of our windows (`PressWatch` hears it without taking it), and another program's window coming to the front once `[ui] tray_grace_ms` has passed since it opened. The picture is the theme's badge, drawn again when the look, the settings or the screen scale change, and the icon goes before the UI ends.
- **Language key:** Carousel, Cycle (`LangCycle`: globe, name, one dot per layout) or List (name and chevron). The List opens `LangList` (with its shadow in `LangPanel`): one page of `[ui] page_rows` rows (`LangRow`) above the keyboard with its left edge at the language key, driven by the board's arrows, Enter and Esc, with a pager when there is more than one page.
- **Top bar:** the theme button cycles the themes (`bar.theme`).

- Every window is a no-focus tool window (ADR-0013); once shown, it is cut to its plate and corner, then guarded until the guard is done.
- The keyboard and the bubble each own their tip (`Tip`), so Windows keeps it above them when the guard lifts them.
- Names, labels and tips are translation keys, looked up in the keyboard's language: the layout's in front, unless `[ui] language` pins one (ADR-0020). Arabic flips text only; nothing mirrors, and a tip in the last language closes when it changes.
- Keys that come in a later version are dimmed, never press, and tell readers and the tooltip "Coming in a later version".
- Text uses `Label` (grayscale, no ClearType fringes), and hover uses `Hover` (the theme's tint, and a `Highlight` outline in high contrast). `Icon` tints with `MultiEffect`, public Qt Quick only.

## Qt Quick tests

`qmltest/` checks the views over stand-ins for `Ui` and `Keyboard` (`FakeUi`, `FakeKeyboard`), because `qmltestrunner` cannot load a module linked into a Rust binary; `tst_app` builds `App` over them as `main.qml` does. They answer from `qmltest/fixtures/` (written by the texts test from the hub and the face's golden files) and from kx-theme's look fixtures, so QML never reads a stale value. The same test lays out the bundled fonts and icons in `target/tmp/qmltest-assets` as the app holds them, so run `cargo test -p kx-ui --test texts` first. Then run them with Qt's `bin` in `PATH` and `QML_DISABLE_DISK_CACHE=1`: `qmltestrunner -input crates/kx-ui/qmltest`. Any warning fails a test.

## The `[ui]` settings

`defaults.toml` holds the values and `settings.rs` the limits of each number. `language = "auto"` follows the keyboard's language, and `en`, `fr` or `ar` pins it; the tray always follows Windows. The check refuses an unknown theme, a language that is neither `auto` nor in `LANGS`, a `min_contrast` outside `kx_theme::contrast::MIN_CONTRAST` (never below WCAG AA), and a timing or row count outside its range. `tray_grace_ms` (500) may be 0, which turns the tray menu's grace off. `send_tries` (10) and `send_wait_ms` (100) set how a second copy hands Show or Quit to the running one; it reads the user's `[ui]` section without changing the file, and falls back to the defaults on any fault.

## The hub

The app makes one `Hub` over its UI ports before any window exists, so it can note the user's focus, and gets a `Receiver<Request>` with it.

- **Bridges:** `install(hub)` hands it to the QML objects once, before QML loads, since QML makes them without arguments (ADR-0018). An object made before it warns and does nothing.
- **The `ui` module:** `UiModule` needs the keyboard face; its start hands the hub the checked settings, the face and its settings handle, and its stop lets the face go.
- **Texts (ADR-0020):** two sets. The keyboard's follow the layout the face shows (`KeyboardFace::lang`): after an action whose state changed, a new language switches them and sends `Wake::Texts`. A layout language without texts gets English, and so does the time before a layout names one. The system's texts (`tray_i18n`) speak `SystemLook::ui_language`.
- **Notices:** `listen(&bus)` hears every notice. Equal notices show once, texts are made in the keyboard's language when the banner takes them and again by `shown_notices`, and only a failed module's notice offers Try again.
- **Requests:** Try again sends `Request::Retry(module)`. Quit, from the user (`Ui.quit`) or a second copy (`command(Command::Quit)`), sends `Request::Quit` once and then ends the queue, so the app's drain loop stops after it. The app calls `Kernel::retry`, or stops the modules and then `end_ui`. When the drain thread is gone, Quit ends the UI itself. Every request is sent under the hub's lock, so none lands after Quit.
- **A second copy's Show** wakes the UI through `command(Command::Show)`.
- **Tray:** `tray_show` shows or changes the icon with the keyboard window's title, in the system's language, as its tooltip; its clicks come back as `Wake::Tray`. `tray_hide` removes it; the root calls it before the UI ends.
- **Spots:** `pop_at` puts a pop-up the theme's gap above the keyboard (else below, else at the top), and `menu_at` puts the tray menu above its click; both stay inside the work area, at the scale of their screen.

## Threads

Every bridge call runs on the Qt GUI thread, including the guard and the window shape. The system look watch, the single-instance commands, the tray icon and the bus may call the hub from other threads: the hub then calls the waker the `Ui` object set, which queues the change to the Qt thread and never blocks. Show and End wakes that come before the `Ui` object exists wait in the hub; a second `Ui` object takes the wakes and warns. The hub never holds its lock during a face, port or waker call.

## QML objects

- `Ui`: settings, language (with the size buttons' letters and names, and the tray's language) and look JSON, the keyboard's texts and the tray's (`trayText`), the space bar's text, the banner's count and a pager's number, notices (take, tell again, Try again, OK), the theme button, the light or dark switch, the backdrop, the tray icon (`trayPx`, `trayShow`, `trayHide`), where pop-ups go (`popAt`, `menuAt`), the compiled-in assets and Quit, with signals for each change (`textsChanged` for a new keyboard language), for the app's Show and Quit, and for tray clicks (`trayClicked`, `trayAway`).
- `Keyboard`: the board and state JSON, every click and size step, the language list, the window's place, the pointer, the bubble's spot, the window shape and the no-focus guard.

## Public API

`Hub`, `HubError`, `UiPorts`, `Request`, `Wake`, `Waker`, `Redraw`, `Resize`, `Shown`, `UiSettings`, `Backdrop`, `SPEC`, `UiModule`, `ID`, `MANIFEST`, `Area`, the i18n API (`I18n`, `Lang`, `LANGS`, `AUTO`, `find`, `pick`, `os_lang`), `harden` (`QT_PREFIXES`, `NAME_END`, `is_qt`, `qt_names`), and with `qt` also `install`, `MAIN_QML`, `ASSETS`, `harden_qt`, `qt_sees`, `pin_imports` and `QML_DIR`.

## Security

See `docs/security/kx-ui.md`. The crate root denies unsafe code; only `src/bridges/` allows it, for the cxx-qt glue and the calls into `cpp/`, with a `// SAFETY:` comment on each block (tidy F3).

# Qt Quick tests

This is the one place that says how to run them. `cargo test -p kx-ui --test texts` writes the fixtures and the icon
plugin first.

On a PC someone uses, run each test offscreen, never on the real screen, at display scale 1 and again at 1.5: tests
that hover move the real pointer (`pointer.js`). CI runs them on its runner's own display.

Set these in the same shell as `qmltestrunner`, from the repository root:

| Variable | Value | Why |
|---|---|---|
| `QT_QPA_PLATFORM` | `offscreen:configfile=crates/kx-ui/qmltest/offscreen.json` | One 1920 × 1080 screen. Qt splits this value on colons, so give a path with no drive letter; a relative one works. |
| `QT_QPA_FONTDIR` | `C:\Windows\Fonts` | The offscreen platform finds the system fonts here. |
| `QML_DISABLE_DISK_CACHE` | `1` | Every run reads the QML as it is now. |
| `QT_SCALE_FACTOR` | `1`, then `1.5` | Both display scales the tests must pass at. |

Then run one test with Qt's `bin` folder on `PATH`, for example
`qmltestrunner -input crates/kx-ui/qmltest/tst_snip_editor.qml -o target/qmltest.txt,txt`. Qt writes its log only to
that file, never to a redirected output, and the exit code counts the failed tests.

`tst_lang`, `tst_notice` and `tst_fit` fail offscreen, as they need a real display; CI checks them.

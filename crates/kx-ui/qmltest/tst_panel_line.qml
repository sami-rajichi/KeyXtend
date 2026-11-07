// A reader's press on a panel's line asks its window for the aim only where that window can take it, and a press or a
// click always says so: in a plain window, or with no window yet, it runs with no warning (ADR-0040).
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    Component { id: lineC; PanelLine {} }
    Component { id: spyC; SignalSpy {} }
    // A hidden panel window, which takes the aim.
    Component {
        id: panelC
        Window {
            signal aiming
        }
    }

    TestCase {
        name: "panel_line"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // An empty line under `parent`, with a spy on its click.
        function made(parent) {
            const line = createTemporaryObject(lineC, parent, {
                said: { text: "", rtl: false, arabic: false },
                hint: "hint",
                rtl: false,
                lk: NL.lk,
                res: Board.resOf(NL.lk, En),
                s: 1,
                width: root.width
            });
            return { line: line, spy: createTemporaryObject(spyC, root, { target: line, signalName: "clicked" }) };
        }

        function test_a_line_in_a_plain_window_takes_a_click_with_no_warning() {
            const { line, spy } = made(root);
            verify(line.Window.window, "it has a window");
            mouseClick(line);
            compare(spy.count, 1);
            line.aim();
            compare(spy.count, 2, "a reader's press too");
        }

        function test_a_line_with_no_window_yet_still_says_it_was_clicked() {
            const { line, spy } = made(null);
            compare(line.Window.window, null);
            line.aim();
            compare(spy.count, 1);
        }

        function test_a_panels_window_is_asked_for_the_aim_once() {
            const panel = createTemporaryObject(panelC, root);
            const { line, spy } = made(panel.contentItem);
            const asked = createTemporaryObject(spyC, root, { target: panel, signalName: "aiming" });
            line.aim();
            compare(asked.count, 1);
            compare(spy.count, 1);
        }
    }
}

// When a pressed key acts against the frame that shows it pressed (`press_frame`), over stand-in bridges: at once by
// default, once that frame is out or its wait ends, or once it is drawn at once; a let-go always follows its press.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/ui-en.js" as En
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "pressframe"
        when: windowShown

        // A key that clicks, and the character key at US "q", which repeats.
        readonly property string esc: "esc"
        readonly property string q: "c-0x0010"

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A shown keyboard window over fresh stand-ins, its presses acting as `mode` says; the real pointer passes.
        function make(mode) {
            const settings = Object.assign({}, En.settings, { pressFrame: mode });
            const ui = createTemporaryObject(uiC, root, { settings: settings });
            const kb = createTemporaryObject(kbC, root);
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            Pointer.through([win]);
            win.visible = true;
            waitForRendering(win.contentItem);
            return win;
        }
        // The key UI Automation finds by `id`.
        function keyed(win, id) {
            return Board.all(win.contentItem, o => o.Accessible.id === id)[0];
        }
        // The keyboard calls that draw, type, hold or let go, in order.
        function sent(win) {
            return win.keyboard.calls.filter(c => /^(drawNow|tap|press|letGo|leave)\b/.test(c));
        }

        function test_by_default_a_key_acts_at_its_press() {
            const win = make(En.settings.pressFrame);
            const key = keyed(win, esc);
            mousePress(key);
            compare(sent(win), ["tap esc false"], "inside the press");
            mouseRelease(key);
            win.visible = false;
        }

        function test_first_acts_once_the_frame_showing_the_press_is_out() {
            const win = make("first");
            const key = keyed(win, esc);
            mousePress(key);
            compare(sent(win), [], "it waits for the frame");
            verify(win.presses.running, "and for its wait");
            win.frameSwapped();
            tryCompare(win.presses, "waiting", null);
            compare(sent(win), ["tap esc false"]);
            verify(!win.presses.running, "the wait ends");
            mouseRelease(key);
            win.visible = false;
        }

        function test_first_acts_when_its_wait_ends_before_a_frame() {
            const win = make("first");
            const key = keyed(win, esc);
            mousePress(key);
            win.presses.triggered();
            compare(sent(win), ["tap esc false"]);
            mouseRelease(key);
            win.visible = false;
        }

        function test_a_let_go_before_the_frame_follows_its_press() {
            const win = make("first");
            const key = keyed(win, q);
            mousePress(key);
            compare(sent(win), []);
            mouseRelease(key);
            compare(sent(win), ["press " + q, "letGo " + q]);
            verify(!win.hold.running, "no beat after the let-go");
            win.visible = false;
        }

        function test_a_press_waiting_as_the_window_hides_never_acts() {
            const win = make("first");
            mousePress(keyed(win, esc));
            win.visible = false;
            win.presses.triggered();
            compare(sent(win), []);
        }

        function test_now_draws_the_pressed_key_then_it_acts() {
            const win = make("now");
            const key = keyed(win, esc);
            let down = false;
            win.keyboard.drawing.connect(() => down = key.down);
            mousePress(key);
            compare(sent(win), [], "after the press is handled");
            tryCompare(win.presses, "waiting", null);
            compare(sent(win), ["drawNow", "tap esc false"]);
            verify(down, "the drawn frame shows the key pressed");
            mouseRelease(key);
            win.visible = false;
        }
    }
}

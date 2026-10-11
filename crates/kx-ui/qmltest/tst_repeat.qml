// A held key (spec §4.6) over stand-in bridges: a repeating key sends its press and let-go instead of a click, sliding
// off or hiding sends leave, and the one hold timer runs only while a key is held; the test drives its ticks.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "repeat"
        when: windowShown

        // The character keys at US "q" and "w", side by side.
        readonly property string q: "c-0x0010"
        readonly property string w: "c-0x0011"

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A shown keyboard window over fresh stand-ins, with `board` when given; the real pointer passes through it.
        function make(board) {
            const ui = createTemporaryObject(uiC, root);
            const kb = createTemporaryObject(kbC, root, board ? { board: board } : {});
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
        // The keyboard calls that type, hold or let go, in order.
        function sent(win) {
            return win.keyboard.calls.filter(c => /^(tap|press|letGo|leave|tick|panelKey)\b/.test(c));
        }
        // The window's hold timer, which alone has a pace.
        function holdOf(win) {
            return [...win.data].find(o => o.paced !== undefined);
        }

        function test_a_click_on_a_repeating_key_sends_its_press_then_its_let_go_once() {
            const win = make();
            const key = keyed(win, q);
            mousePress(key);
            compare(sent(win), ["press " + q]);
            mouseRelease(key);
            compare(sent(win), ["press " + q, "letGo " + q], "one let-go, so it types once");
            win.visible = false;
        }

        function test_sliding_off_the_key_sends_leave_and_a_let_go_outside_sends_nothing() {
            const win = make();
            const key = keyed(win, q);
            const next = keyed(win, w);
            mousePress(key);
            const off = next.mapToItem(key, next.width / 2, next.height / 2);
            mouseMove(key, off.x, off.y, -1, Qt.LeftButton);
            compare(sent(win), ["press " + q, "leave"]);
            mouseRelease(key, off.x, off.y);
            compare(sent(win), ["press " + q, "leave"], "nothing more, and no click on the other key");
            win.visible = false;
        }

        function test_the_hold_timer_runs_only_while_a_key_is_held_at_the_delay_then_the_pace() {
            const win = make();
            const key = keyed(win, q);
            const hold = holdOf(win);
            verify(!hold.running, "it rests while nothing is held");
            mousePress(key);
            verify(hold.running, "held");
            compare(hold.interval, Face.board.repeat_delay_ms, "first the delay");
            hold.triggered();
            compare(hold.interval, Face.board.repeat_every_ms, "then the pace");
            hold.triggered();
            compare(sent(win), ["press " + q, "tick", "tick"]);
            mouseRelease(key);
            verify(!hold.running, "it rests again");
            compare(sent(win), ["press " + q, "tick", "tick", "letGo " + q]);
            mousePress(key);
            compare(hold.interval, Face.board.repeat_delay_ms, "a new hold waits the delay again");
            mouseRelease(key);
            win.visible = false;
        }

        function test_hiding_the_keyboard_while_a_key_is_held_stops_the_repeat() {
            const win = make();
            mousePress(keyed(win, q));
            win.visible = false;
            verify(!holdOf(win).running);
            compare(sent(win), ["press " + q, "leave"]);
        }

        function test_the_language_key_named_keys_and_dimmed_keys_keep_their_click() {
            const win = make();
            for (const id of ["lang", "esc", "mic"])
                mouseClick(keyed(win, id));
            compare(sent(win), ["tap lang false", "tap esc false"], "the dimmed key sends nothing");
            win.visible = false;
        }

        function test_with_repeat_off_every_key_clicks() {
            const keys = Face.board.keys.map(k => Object.assign({}, k, { repeat: false }));
            const win = make(Object.assign({}, Face.board, { keys: keys }));
            mouseClick(keyed(win, q));
            compare(sent(win), ["tap " + q + " false"]);
            verify(!holdOf(win).running);
            win.visible = false;
        }
    }
}

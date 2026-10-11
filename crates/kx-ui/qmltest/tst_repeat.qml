// A held key (spec §4.6) over stand-in bridges: a repeating key sends its press and let-go instead of a click, Qt's
// cancel or hiding sends leave, and the one hold timer runs only while a key is held; the test drives its ticks.
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

        // The character key at US "q".
        readonly property string q: "c-0x0010"

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

        function test_a_click_on_a_repeating_key_sends_its_press_then_its_let_go_once() {
            const win = make();
            const key = keyed(win, q);
            mousePress(key);
            compare(sent(win), ["press " + q]);
            mouseRelease(key);
            compare(sent(win), ["press " + q, "letGo " + q], "one let-go, so it types once");
            win.visible = false;
        }

        function test_a_beat_that_redrew_reads_the_state_again() {
            const win = make();
            mousePress(keyed(win, q));
            win.keyboard.reply = { board: false, state: true, repeated: false };
            const shown = win.st;
            win.hold.triggered();
            verify(win.st !== shown, "the state is read again");
            mouseRelease(keyed(win, q));
            win.visible = false;
        }

        function test_a_beat_that_typed_reads_no_state() {
            const win = make();
            const key = keyed(win, q);
            mousePress(key);
            win.keyboard.reply = { board: false, state: false, repeated: true };
            const shown = win.st;
            win.hold.triggered();
            verify(win.st === shown, "a plain repeat redraws nothing");
            mouseRelease(key);
            win.visible = false;
        }

        function test_a_key_held_as_the_list_opens_lets_go_and_never_drives_the_list() {
            const win = make();
            const down = keyed(win, "down");
            mousePress(down);
            win.keyboard.state = Face.states.list;
            win.restate();
            const panel = [...win.data].find(o => o.item && o.item.win === win).item;
            const list = [...panel.data].find(o => o.view !== undefined);
            tryVerify(() => list.visible && panel.visible, 2000, "the list shows");
            waitForRendering(list.contentItem);
            mouseRelease(down);
            compare(sent(win), ["press down", "letGo down"]);
            win.visible = false;
        }

        function test_while_the_list_is_open_a_key_that_drives_it_acts_on_it_at_the_press() {
            const win = make();
            win.keyboard.state = Face.states.list;
            win.restate();
            const panel = [...win.data].find(o => o.item && o.item.win === win).item;
            tryVerify(() => panel.visible, 2000, "the list shows");
            const down = keyed(win, "down");
            mousePress(down);
            compare(sent(win), ["panelKey down"], "before the let-go");
            verify(!win.hold.running, "it never holds");
            mouseRelease(down);
            compare(sent(win), ["panelKey down"], "the let-go adds nothing");
            win.visible = false;
        }

        function test_the_hold_timer_runs_only_while_a_key_is_held_at_the_delay_then_the_pace() {
            const win = make();
            const key = keyed(win, q);
            const hold = win.hold;
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
            verify(!win.hold.running);
            compare(sent(win), ["press " + q, "leave"]);
        }

        function test_a_stopped_keyboard_ends_the_hold() {
            const win = make();
            mousePress(keyed(win, q));
            win.board = null;
            verify(!win.hold.running);
            compare(sent(win), ["press " + q, "leave"]);
            win.visible = false;
        }

        function test_a_press_qt_cancels_sends_leave() {
            const win = make();
            const key = keyed(win, q);
            mousePress(key);
            Board.all(key, o => o.containsMouse !== undefined)[0].canceled();
            compare(sent(win), ["press " + q, "leave"]);
            verify(!win.hold.running);
            win.visible = false;
        }

        function test_a_readers_press_on_a_repeating_key_still_clicks() {
            const win = make();
            keyed(win, q).Accessible.pressAction();
            compare(sent(win), ["tap " + q + " false"]);
            verify(!win.hold.running);
            win.visible = false;
        }

        function test_a_readers_toggle_on_a_latch_still_clicks() {
            const win = make();
            keyed(win, "shift").Accessible.toggleAction();
            compare(sent(win), ["tap shift false"]);
            win.visible = false;
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
            verify(!win.hold.running);
            win.visible = false;
        }
    }
}

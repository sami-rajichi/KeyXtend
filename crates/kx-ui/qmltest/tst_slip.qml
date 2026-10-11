// Keys act on the press they got (ADR-0024): at the press, so a slip of the hand onto a neighbour, into a gap or off
// the keyboard neither cancels the key nor acts on another; a held key repeats until let-go, wherever the pointer is.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "slip"
        when: windowShown

        // The character keys at US "q" and "w", side by side, and Tab, which never repeats, left of "q".
        readonly property string q: "c-0x0010"
        readonly property string w: "c-0x0011"
        readonly property string tab: "tab"
        // The owner's press, about 0.4 s, and its slips: 1 px each, from 1 px inside the key's right edge into the gap.
        readonly property int pressMs: 400
        readonly property var slips: [1, 1, 1, 1]

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A shown keyboard window over fresh stand-ins; the real pointer passes through it.
        function make() {
            const ui = createTemporaryObject(uiC, root);
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
        // The keyboard calls that type, hold, let go or drive the list, in order.
        function sent(win) {
            return win.keyboard.calls.filter(c => /^(tap|press|letGo|leave|tick|panelKey)\b/.test(c));
        }
        // The middle of key `other`, in `key`'s units.
        function onto(win, key, other) {
            const n = keyed(win, other);
            return n.mapToItem(key, n.width / 2, n.height / 2);
        }
        // A point left of the keyboard window, in `key`'s units.
        function outside(win, key) {
            return key.mapFromItem(win.contentItem, -win.contentItem.width / 2, win.contentItem.height / 2);
        }
        // Presses `key` at its middle, slips the still-down pointer to `to` and lets go there.
        function slip(key, to) {
            mousePress(key, key.width / 2, key.height / 2);
            mouseMove(key, to.x, to.y, -1, Qt.LeftButton);
            mouseRelease(key, to.x, to.y);
        }

        function test_a_key_acts_at_the_press_before_the_let_go() {
            const win = make();
            mousePress(keyed(win, q));
            compare(sent(win), ["press " + q], "a repeating key types at the press");
            mouseRelease(keyed(win, q));
            mousePress(keyed(win, tab));
            compare(sent(win), ["press " + q, "letGo " + q, "tap tab false"], "and any other key too");
            mouseRelease(keyed(win, tab));
            compare(sent(win).length, 3, "its let-go adds nothing");
            win.visible = false;
        }

        function test_a_press_in_the_gap_between_keys_acts_nothing() {
            const win = make();
            const a = keyed(win, q);
            const b = keyed(win, w);
            const gap = (a.x + a.width + b.x) / 2 - a.x;
            verify(gap > a.width && a.x + gap < b.x, "between the caps");
            mouseClick(a, gap, a.height / 2);
            compare(sent(win), []);
            win.visible = false;
        }

        function test_a_slip_onto_a_neighbour_acts_once_and_the_neighbour_never() {
            const win = make();
            const a = keyed(win, q);
            slip(a, onto(win, a, w));
            const t = keyed(win, tab);
            slip(t, onto(win, t, q));
            compare(sent(win), ["press " + q, "letGo " + q, "tap tab false"]);
            win.visible = false;
        }

        function test_a_slip_off_the_keyboard_acts_once_and_its_release_is_ignored() {
            const win = make();
            const a = keyed(win, q);
            slip(a, outside(win, a));
            const t = keyed(win, tab);
            slip(t, outside(win, t));
            compare(sent(win), ["press " + q, "letGo " + q, "tap tab false"]);
            win.visible = false;
        }

        function test_a_hold_that_slips_onto_a_neighbour_repeats_until_let_go() {
            const win = make();
            const a = keyed(win, q);
            mousePress(a);
            const on = onto(win, a, w);
            mouseMove(a, on.x, on.y, -1, Qt.LeftButton);
            verify(win.hold.running, "the hold goes on");
            win.hold.triggered();
            win.hold.triggered();
            mouseRelease(a, on.x, on.y);
            compare(sent(win), ["press " + q, "tick", "tick", "letGo " + q], "nothing for the neighbour");
            verify(!win.hold.running);
            win.visible = false;
        }

        function test_a_double_click_acts_twice_one_per_press() {
            const win = make();
            mouseDoubleClickSequence(keyed(win, q));
            mouseDoubleClickSequence(keyed(win, tab));
            const twice = ["press " + q, "letGo " + q, "press " + q, "letGo " + q, "tap tab false", "tap tab false"];
            compare(sent(win), twice);
            win.visible = false;
        }

        function test_the_language_key_takes_its_part_from_where_it_was_pressed() {
            const win = make();
            const lang = keyed(win, "lang");
            const back = lang.width * lang.lk.shape.lang.back_share;
            const y = lang.height / 2;
            mousePress(lang, back / 2, y);
            mouseMove(lang, (back + lang.width) / 2, y, -1, Qt.LeftButton);
            mouseRelease(lang, (back + lang.width) / 2, y);
            mousePress(lang, (back + lang.width) / 2, y);
            mouseMove(lang, back / 2, y, -1, Qt.LeftButton);
            mouseRelease(lang, back / 2, y);
            compare(sent(win), ["tap lang true", "tap lang false"]);
            win.visible = false;
        }

        // The owner's own press: held about 0.4 s, below the repeat delay, while the hand slips from 1 px inside the
        // edge into the gap. No real time passes: the hold's first beat is checked to come after such a press.
        function test_a_press_held_like_the_owners_with_slips_into_the_gap_types_once() {
            const win = make();
            const a = keyed(win, q);
            const at = { x: a.width - 1, y: a.height / 2 };
            mousePress(a, at.x, at.y);
            verify(win.hold.running && win.hold.interval > pressMs, "no repeat within the owner's press");
            for (const px of slips) {
                at.x += px;
                mouseMove(a, at.x, at.y, -1, Qt.LeftButton);
            }
            verify(at.x >= a.width, "the pointer ended off the key");
            mouseRelease(a, at.x, at.y);
            compare(sent(win), ["press " + q, "letGo " + q]);
            win.visible = false;
        }
    }
}

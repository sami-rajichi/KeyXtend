// Placing the keyboard again over stand-in bridges: a new scale or screen during a move or resize waits for its end,
// a new screen size or a screen change of the window's own places it again, and a window made again starts where the
// keyboard keeps it.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "kbwin.js" as Win
import "board.js" as Board

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    // A stand-in screen with its corner at (`virtualX`, 0), 4000 by 3000 Qt units, at scale 1.
    Component {
        id: screenC
        QtObject {
            property real devicePixelRatio: 1
            property int virtualX: 0
            property int virtualY: 0
            property int width: 4000
            property int height: 3000
        }
    }
    // A stand-in for the window's screen, whose size changes when the test says.
    Component {
        id: displayC
        QtObject {
            property int width: 1920
            property int height: 1080
        }
    }

    TestCase {
        id: test
        name: "place"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A shown keyboard window that the keyboard put at physical `at` (kbwin.js).
        function make(at) {
            return Win.make(test, { ui: uiC, kb: kbC, win: winC }, root, at);
        }
        // Starts a move or a resize from bar control `id`.
        function start(r, id) {
            mouseClick(Board.all(r.win.contentItem, o => o.Accessible.id === id)[0]);
            verify(r.mover.busy, id);
        }
        // Waits for a placement skipped while the window moves or resizes; none comes until it ends.
        function owed(r, placed) {
            tryVerify(() => r.placing.owed, 2000, "owed");
            compare(Win.heard(r.kb, "placed"), placed, "not placed while it runs");
        }

        function test_a_new_scale_during_a_resize_places_the_keyboard_when_it_ends() {
            const r = make({ x: 100, y: 200 });
            start(r, "bar.corner");
            const placed = Win.heard(r.kb, "placed");
            const ratio = r.rescale.ratio * 2;
            r.rescale.ratio = ratio;
            owed(r, placed);
            compare(r.kb.shapeDpr, ratio, "cut again at once");
            r.win.finish();
            compare(Win.heard(r.kb, "placed"), placed + 1, "placed as it ends");
            compare([r.kb.placedAt.x, r.kb.placedAt.y, r.kb.placedAt.w], [100, 200, r.win.wide * ratio]);
            verify(!r.placing.owed);
            r.win.visible = false;
        }

        // A screen that goes during a resize places the keyboard on the screens left once the resize ends.
        function test_a_screen_that_goes_during_a_resize_places_the_keyboard_on_the_screens_left_when_it_ends() {
            const r = make({ x: 4100, y: 90 });
            const a = createTemporaryObject(screenC, root);
            const b = createTemporaryObject(screenC, root, { virtualX: 4000 });
            r.win.spots.screens = [a, b];
            tryVerify(() => r.kb.placedAt && r.kb.placedAt.x === 4100, 2000, "on the second screen");
            start(r, "bar.corner");
            r.kb.state = Object.assign({}, r.kb.state, { at: { x: 100, y: 90 } });
            const placed = Win.heard(r.kb, "placed");
            r.win.spots.screens = [a];
            owed(r, placed);
            r.win.finish();
            compare(Win.heard(r.kb, "placed"), placed + 1, "placed as it ends");
            tryCompare(r.win, "at", Qt.point(100, 90), 2000, "where the keyboard puts it");
            r.win.visible = false;
        }

        // A move places the keyboard where it ends, so a placement owed meanwhile comes once, from there.
        function test_a_new_scale_during_a_move_places_the_keyboard_once_where_it_ends() {
            const r = make({ x: 120, y: 90 });
            start(r, "bar.grip");
            const placed = Win.heard(r.kb, "placed");
            r.rescale.ratio = r.rescale.ratio * 2;
            owed(r, placed);
            r.win.finish();
            compare(Win.heard(r.kb, "placed"), placed + 1, "once");
            verify(!r.placing.owed);
            r.win.visible = false;
        }

        // A new resolution at the same scale places the keyboard again, so it stays on the smaller screen.
        function test_a_new_screen_size_places_the_keyboard_again() {
            const r = make({ x: 100, y: 200 });
            const display = createTemporaryObject(displayC, root);
            r.placing.display = display;
            for (const side of ["width", "height"]) {
                const placed = Win.heard(r.kb, "placed");
                display[side] = display[side] / 2;
                tryVerify(() => Win.heard(r.kb, "placed") === placed + 1, 2000, side + ": placed again");
                compare([r.kb.placedAt.x, r.kb.placedAt.y], [100, 200], "from where it is");
            }
            r.win.visible = false;
        }

        // A window made while the keyboard keeps a place, as after a device loss, starts there and reports it unmoved.
        function test_a_window_made_again_starts_where_the_keyboard_keeps_it() {
            const state = Object.assign({}, Face.states.en, { at: { x: 100, y: 200 } });
            const kb = createTemporaryObject(kbC, root, { state: state });
            const win = createTemporaryObject(winC, root, { ui: createTemporaryObject(uiC, root), keyboard: kb });
            win.place();
            compare([kb.placedAt.x, kb.placedAt.y], [100, 200], "reported unmoved");
            const l = win.spots.logical(100, 200);
            compare([win.at.x, win.at.y], [l.x, l.y], "where the keyboard keeps it");
        }

        function test_the_windows_own_screen_change_places_it_again() {
            const r = make({ x: 100, y: 200 });
            const placed = Win.heard(r.kb, "placed");
            r.win.screenChanged();
            tryVerify(() => Win.heard(r.kb, "placed") === placed + 1, 2000, "placed again");
            r.win.visible = false;
        }
    }
}

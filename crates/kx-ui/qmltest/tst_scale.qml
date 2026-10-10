// A display scale or screen change over stand-in bridges: a window keeps the size and place its bindings give in Qt
// units, the keyboard is cut and placed again in the new pixels (G11), and the pop-ups go back where they belong.
import QtQuick
import QtTest
import "../qml"
import "kbwin.js" as Win
import "fixtures/face.js" as Face

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: spyC; SignalSpy {} }
    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {}
        }
    }
    // A stand-in screen with its corner at (`virtualX`, 0), 4000 by 3000 Qt units, at the scale the test sets.
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
    // A hidden window kept at 240 by 120 at (60, 50), and a value that changes each time the kept values are set.
    Component {
        id: keptC
        Window {
            readonly property alias rescale: rescale
            property var kept: [rescale.keep(1)]
            flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus
            x: rescale.keep(60)
            y: rescale.keep(50)
            width: rescale.keep(240)
            height: rescale.keep(120)
            Rescale {
                id: rescale
            }
        }
    }

    TestCase {
        id: test
        name: "scale"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A shown keyboard window that the keyboard put at physical `at` (kbwin.js).
        function make(at) {
            return Win.make(test, { ui: uiC, kb: kbC, win: winC }, root, at);
        }
        // The `Rescale` of window `w`.
        function rescaleOf(w) {
            return Win.all(w.contentItem, o => o.keep !== undefined)[0];
        }
        // The keyboard over stand-in `screens`, at its state's place in their Qt units.
        function onScreens(r, screens) {
            const placed = Win.heard(r.kb, "placed");
            r.win.spots.screens = screens;
            tryVerify(() => Win.heard(r.kb, "placed") === placed + 1, 2000, "placed again on the new screens");
            r.win.restate();
        }
        // The app over fresh stand-ins, once its keyboard shows.
        function app() {
            const a = createTemporaryObject(appC, root);
            tryVerify(() => a.face.visible, 2000, "the keyboard shows");
            waitForRendering(a.face.contentItem);
            return a;
        }
        // Doubles window `w`'s scale; the pop-up spots answer `spot` in physical pixels from then on.
        function newScale(a, w, spot) {
            a.ui.popSpot = Qt.point(spot.x, spot.y);
            const r = rescaleOf(w);
            r.ratio = r.ratio * 2;
            return a.face.spots.logical(spot.x, spot.y);
        }

        // Qt leaves the native window as it was on a new scale, and may resize it during a move: the bindings win.
        function test_a_window_changed_from_outside_takes_its_bound_size_and_place_again() {
            const w = createTemporaryObject(keptC, root);
            w.setGeometry(70, 80, 180, 90);
            compare(w.width, 180, "changed from outside");
            tryCompare(w, "width", 240, 2000, "the binding wins");
            compare([w.x, w.y, w.width, w.height], [60, 50, 240, 120]);
        }

        function test_a_new_scale_sets_every_kept_value_again() {
            const w = createTemporaryObject(keptC, root);
            const spy = createTemporaryObject(spyC, root, { target: w, signalName: "keptChanged" });
            w.rescale.ratio = w.rescale.ratio * 2;
            compare(spy.count, 1, "the bindings ran again");
        }

        function test_on_a_new_scale_the_keyboard_is_cut_and_placed_again_where_it_is() {
            const at = { x: 100, y: 200 };
            const r = make(at);
            const placed = Win.heard(r.kb, "placed");
            const ratio = r.rescale.ratio * 2;
            r.rescale.ratio = ratio;
            tryVerify(() => Win.heard(r.kb, "placed") === placed + 1, 2000, "placed again");
            const size = { w: r.win.wide * ratio, h: r.win.high * ratio, overhang: r.win.overhang / r.win.s };
            compare(r.kb.placedAt, Object.assign({}, at, size), "where it is, its size in the new pixels");
            compare(r.kb.shapeDpr, ratio, "cut at the new scale");
            r.win.visible = false;
        }

        // A moved keyboard keeps its place in pixels, in the new Qt units, though the keyboard answers nothing.
        function test_on_a_new_scale_a_moved_keyboard_keeps_its_place_in_pixels() {
            const r = make({ x: 400, y: 600 });
            const sc = createTemporaryObject(screenC, root);
            onScreens(r, [sc]);
            compare(r.win.at, Qt.point(400, 600), "on the stand-in screen");
            sc.devicePixelRatio = 2;
            r.rescale.ratio = 2;
            tryCompare(r.win, "at", Qt.point(200, 300), 2000, "its pixels in the new Qt units");
            r.win.visible = false;
        }

        // A new scale after a move starts from where the user left it, though the keyboard answered no change.
        function test_on_a_new_scale_a_keyboard_just_moved_is_placed_again_from_where_it_was_left() {
            const r = make({ x: 100, y: 200 });
            r.kb.keeps = true;
            mouseClick(Win.all(r.win.contentItem, o => o.Accessible.id === "bar.grip")[0]);
            r.kb.pointer = { x: 300, y: 260 };
            tryVerify(() => r.win.at.x > 100, 2000, "the poll moves it");
            mouseClick(Win.all(r.mover.covers.objectAt(0).contentItem, o => o.Accessible.id === "catch")[0]);
            const left = r.kb.placedAt;
            const placed = Win.heard(r.kb, "placed");
            r.rescale.ratio = r.rescale.ratio * 2;
            tryVerify(() => Win.heard(r.kb, "placed") === placed + 1, 2000, "placed again");
            compare([r.kb.placedAt.x, r.kb.placedAt.y], [left.x, left.y], "from where the move left it");
            r.win.visible = false;
        }

        // A screen that goes places the keyboard again from where it was, and it goes where the keyboard puts it.
        function test_a_screen_that_goes_places_the_keyboard_again_on_the_screens_left() {
            const r = make({ x: 4100, y: 90 });
            const a = createTemporaryObject(screenC, root);
            const b = createTemporaryObject(screenC, root, { virtualX: 4000 });
            onScreens(r, [a, b]);
            compare(r.win.at, Qt.point(4100, 90), "on the second screen");
            r.kb.state = Object.assign({}, r.kb.state, { at: { x: 100, y: 90 } });
            const placed = Win.heard(r.kb, "placed");
            r.win.spots.screens = [a];
            tryVerify(() => Win.heard(r.kb, "placed") === placed + 1, 2000, "placed again");
            compare([r.kb.placedAt.x, r.kb.placedAt.y], [4100, 90], "from where it was in pixels");
            tryCompare(r.win, "at", Qt.point(100, 90), 2000, "where the keyboard puts it");
            r.win.visible = false;
        }

        function test_a_new_scale_puts_the_shown_bubble_back_in_its_corner() {
            const a = app();
            a.keyboard.bubble = { x: 40, y: 50 };
            verify(a.minimise(), "minimised");
            a.keyboard.bubble = { x: 60, y: 70 };
            const r = rescaleOf(a.bubble);
            r.ratio = r.ratio * 2;
            const want = a.face.spots.logical(60, 70);
            tryCompare(a.bubble, "x", Math.trunc(want.x), 2000, "its corner in the new Qt units");
            compare(a.bubble.y, Math.trunc(want.y));
        }

        // The bubble goes back to its corner from the keyboard's place in the new scale, whichever hears it first.
        function test_a_new_scale_puts_the_bubble_back_from_the_keyboards_new_place() {
            const a = app();
            const kb = a.keyboard;
            kb.state = Object.assign({}, kb.state, { at: { x: 300, y: 200 } });
            a.face.restate();
            kb.bubble = { x: 40, y: 50 };
            verify(a.minimise(), "minimised");
            kb.state = Object.assign({}, kb.state, { at: { x: 500, y: 400 } });
            for (const w of [a.bubble, a.face]) {
                const r = rescaleOf(w);
                r.ratio = r.ratio * 2;
            }
            const f = a.face;
            const to = f.spots.logical(500, 400);
            tryVerify(() => f.at.x === to.x && f.at.y === to.y, 2000, "the keyboard placed in the new scale");
            const c = f.spots.physical(f.at.x + f.drawn.width / 2, f.at.y + f.drawn.height / 2);
            tryVerify(() => kb.bubbleFrom.x === c.x && kb.bubbleFrom.y === c.y, 2000, "from its new place");
        }

        function test_a_new_scale_puts_the_banner_back_over_the_keyboard() {
            const a = app();
            a.ui.notices = [{ id: 1, text: "Notice 1", retry: false }];
            a.ui.noticesWaiting();
            tryVerify(() => a.notice.visible, 2000, "the banner shows");
            const want = newScale(a, a.notice, { x: 90, y: 20 });
            tryCompare(a.notice, "x", Math.trunc(want.x), 2000, "placed again");
            compare(a.notice.y, Math.trunc(want.y));
        }

        function test_a_new_scale_puts_the_language_list_back_over_the_keyboard() {
            const a = app();
            a.keyboard.board = Object.assign({}, Face.board, { lang_key: "list" });
            a.keyboard.state = Face.states.list;
            a.face.redraw(JSON.stringify({ board: true, state: true }));
            const panel = () => [...a.face.data].find(o => o.item && o.item.win === a.face);
            tryVerify(() => panel() && panel().item.visible, 2000, "the list shows");
            const list = [...panel().item.data].find(o => o.view !== undefined);
            tryVerify(() => list.visible, 2000);
            const want = newScale(a, list, { x: 70, y: 10 });
            tryCompare(list, "x", Math.trunc(want.x), 2000, "placed again");
            compare(list.y, Math.trunc(want.y));
        }

        function test_a_new_scale_closes_the_tray_menu() {
            const a = app();
            a.ui.trayClicked(300, 500);
            verify(a.menu.visible, "the menu opens");
            const r = rescaleOf(a.menu);
            r.ratio = r.ratio * 2;
            verify(!a.menu.visible, "closed");
        }

        // A move places the keyboard when it ends, so a new scale meanwhile only cuts it again.
        function test_a_new_scale_during_a_move_only_cuts_the_keyboard() {
            const r = make({ x: 120, y: 90 });
            mouseClick(Win.all(r.win.contentItem, o => o.Accessible.id === "bar.grip")[0]);
            verify(r.mover.busy, "moving");
            const placed = Win.heard(r.kb, "placed");
            r.rescale.ratio = r.rescale.ratio * 2;
            tryVerify(() => r.kb.shapeDpr === r.rescale.ratio, 2000, "cut again");
            compare(Win.heard(r.kb, "placed"), placed, "not placed mid-move");
            r.win.finish();
            r.win.visible = false;
        }
    }
}

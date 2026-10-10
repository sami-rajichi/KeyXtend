// Moving and resizing over stand-in bridges: one click starts, the window follows the polled pointer from where it was
// grabbed, and a click on a catcher ends it; and Spots' units on a scaled second screen.
import QtQuick
import QtTest
import "../qml"
import "fixtures/ui-en.js" as En
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: spotsC; Spots {} }
    Component { id: plainC; Window {} }

    // A main screen at scale 1 and a second one to its right at 1.5, as Qt gives them: corners in physical pixels,
    // sizes in Qt units.
    readonly property var screens: [
        { virtualX: 0, virtualY: 0, width: 1920, height: 1080, devicePixelRatio: 1 },
        { virtualX: 1920, virtualY: 0, width: 1280, height: 720, devicePixelRatio: 1.5 }
    ]

    TestCase {
        name: "move"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Every item under `item` for which `ok` holds.
        function all(item, ok, found) {
            found = found || [];
            if (ok(item))
                found.push(item);
            for (let i = 0; i < item.children.length; i++)
                all(item.children[i], ok, found);
            return found;
        }
        // The one control UI Automation finds by `id`.
        function keyed(win, id) {
            const hits = all(win.contentItem, o => o.Accessible.id === id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        // A shown keyboard window over fresh stand-ins, its pointer at `p`; the real pointer passes through it and the
        // catchers, which cover whole screens while it runs.
        function make(p) {
            const kb = createTemporaryObject(kbC, root, { pointer: p });
            const win = createTemporaryObject(winC, root, { ui: createTemporaryObject(uiC, root), keyboard: kb });
            const mover = all(win.contentItem, o => o.covers !== undefined)[0];
            const covers = [];
            for (let i = 0; i < mover.covers.count; i++)
                covers.push(mover.covers.objectAt(i));
            Pointer.through([win].concat(covers));
            win.visible = true;
            waitForRendering(win.contentItem);
            return { win: win, kb: kb, mover: mover, covers: covers };
        }
        // How often the keyboard heard `call`.
        function heard(kb, call) {
            return kb.calls.filter(c => c === call).length;
        }
        // The window's place for physical pointer `p` when it was grabbed at `grab`, in Qt units.
        function under(win, p, grab) {
            const l = win.spots.logical(p.x, p.y);
            return Qt.point(l.x - grab.x, l.y - grab.y);
        }

        function test_the_window_follows_the_pointer_from_where_it_was_grabbed_until_the_next_click() {
            const start = { x: 120, y: 90 };
            const r = make(start);
            r.win.at = Qt.point(40, 30);
            mouseClick(keyed(r.win, "bar.grip"));
            verify(r.mover.busy, "the first click starts");
            const l = r.win.spots.logical(start.x, start.y);
            const grab = Qt.point(l.x - 40, l.y - 30);
            compare(r.mover.grab, grab, "held where it was grabbed");
            verify(r.covers.every(c => c.visible), "a catcher covers each screen");
            r.kb.pointer = { x: 260, y: 170 };
            tryCompare(r.win, "at", under(r.win, r.kb.pointer, grab), 2000, "the poll follows");
            const placed = heard(r.kb, "placed");
            mouseClick(keyed(r.win, "catch"));
            verify(!r.mover.busy, "the second click ends it");
            verify(r.covers.every(c => !c.visible), "the catchers go");
            compare(heard(r.kb, "placed"), placed + 1, "the keyboard hears the new place");
            const left = r.win.at;
            r.kb.pointer = { x: 400, y: 300 };
            r.mover.follow();
            compare(r.win.at, left, "a late poll no longer moves it");
            r.win.visible = false;
        }

        // The keyboard places the whole window: its box, and how far the theme's resize corner hangs past the plate.
        function test_a_placement_sends_the_whole_window_and_the_corners_overhang() {
            const r = make({ x: 120, y: 90 });
            mouseClick(keyed(r.win, "bar.grip"));
            mouseClick(keyed(r.win, "catch"));
            const p = r.kb.placedAt;
            compare(p.overhang, r.win.lk.shape.extra.corner_px[0] / 2, "half the corner, at size 1");
            const dpr = r.win.spots.physical(r.win.at.x, r.win.at.y).dpr;
            compare([p.w, p.h], [r.win.width * dpr, r.win.height * dpr], "the window, corner and all");
            verify(r.win.width > r.win.board.plate.w, "wider than the plate");
            r.win.visible = false;
        }

        // A new corner, or a look read after the first placement, places the keyboard again where the keyboard put it,
        // so it keeps the whole corner on screen and saves no move.
        function test_a_new_corner_places_the_keyboard_again_where_it_is() {
            const r = make({ x: 120, y: 90 });
            r.kb.keeps = true;
            r.win.place();
            const placed = heard(r.kb, "placed");
            const lk = JSON.parse(JSON.stringify(r.win.lk));
            lk.shape.extra.corner_px[0] *= 2;
            r.win.lk = lk;
            tryVerify(() => heard(r.kb, "placed") === placed + 1, 2000, "placed again");
            compare(r.kb.placedAt.overhang, lk.shape.extra.corner_px[0] / 2, "with the new corner, at size 1");
            const at = r.win.st.at;
            compare([r.kb.placedAt.x, r.kb.placedAt.y], [at.x, at.y], "where the keyboard put it");
            r.win.visible = false;
        }

        function test_a_resize_follows_the_pointer_and_a_click_on_a_screens_catcher_ends_it() {
            const r = make({ x: 300, y: 200 });
            mouseClick(keyed(r.win, "bar.corner"));
            verify(r.mover.busy, "the first click starts");
            compare(heard(r.kb, "resizeBegin"), 1);
            tryVerify(() => heard(r.kb, "resizeAt") > 0, 2000, "the poll resizes");
            const placed = heard(r.kb, "placed");
            mouseClick(all(r.covers[0].contentItem, o => o.Accessible.id === "catch")[0]);
            verify(!r.mover.busy, "a click anywhere ends it");
            compare(heard(r.kb, "resizeEnd"), 1, "the keyboard keeps the size reached");
            compare(heard(r.kb, "placed"), placed, "a resize keeps its place");
            r.win.visible = false;
        }

        function test_nothing_starts_while_windows_cannot_say_where_the_pointer_is() {
            const r = make(null);
            mouseClick(keyed(r.win, "bar.grip"));
            mouseClick(keyed(r.win, "bar.corner"));
            verify(!r.mover.busy);
            compare(heard(r.kb, "resizeBegin"), 0);
            r.win.visible = false;
        }

        function test_spots_turn_physical_pixels_into_qt_units_on_the_screen_that_holds_them_and_back() {
            const s = createTemporaryObject(spotsC, root, { win: createTemporaryObject(plainC, root), screens: root.screens });
            compare(s.logical(2220, 150), { x: 2120, y: 100 }, "the second screen's scale from its corner");
            compare(s.physical(2120, 100), { x: 2220, y: 150, dpr: 1.5 });
            compare(s.logical(500, 400), { x: 500, y: 400 }, "the main screen at scale 1");
            for (const p of [{ x: 1920, y: 0 }, { x: 3839, y: 1079 }, { x: 2500.5, y: 333 }]) {
                const l = s.logical(p.x, p.y);
                const back = s.physical(l.x, l.y);
                fuzzyCompare(back.x, p.x, 1e-9, "a round trip lands where it started");
                fuzzyCompare(back.y, p.y, 1e-9);
            }
            compare(s.logical(-50, 10), { x: -50, y: 10 }, "off every screen, unchanged");
        }
    }
}

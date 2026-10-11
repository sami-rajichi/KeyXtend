// The keyboard window's controls act at let-go on the press they got (ADR-0024): a slip onto a neighbour or off the
// window before the release still acts once, only once, and nothing it slips onto acts; the bubble too.
import QtQuick
import QtTest
import "../qml"
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    // The app as the root makes it, over the stand-ins, with the pointer and the bubble's spot known.
    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {
                pointer: ({ x: 300, y: 200 })
                bubble: ({ x: 40, y: 50 })
            }
        }
    }

    TestCase {
        name: "letgo_window"
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
        // The one control UI Automation finds by `id` in window `win`.
        function keyed(win, id) {
            const hits = all(win.contentItem, o => o.Accessible.id === id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        // The app over fresh stand-ins, once its keyboard shows; the real pointer passes through its windows and the
        // catchers, which cover whole screens while a move runs.
        function make() {
            const app = createTemporaryObject(appC, root);
            Pointer.through([app.face, app.bubble]);
            tryVerify(() => app.face.visible, 2000, "the keyboard shows");
            Pointer.through(covers(app.face));
            waitForRendering(app.face.contentItem);
            return app;
        }
        // The catchers of keyboard window `win`, one per screen.
        function covers(win) {
            const found = [];
            for (let i = 0; i < win.mover.covers.count; i++)
                found.push(win.mover.covers.objectAt(i));
            return found;
        }
        // The middle of `other`, in `item`'s units.
        function onto(item, other) {
            return other.mapToItem(item, other.width / 2, other.height / 2);
        }
        // A point left of `item`'s window, in `item`'s units.
        function outside(item) {
            const w = item.Window;
            return item.mapFromItem(w.contentItem, -w.width / 2, w.height / 2);
        }
        // Presses `item` at its middle, slips the still-down pointer to `to` and lets go there.
        function slip(item, to) {
            mousePress(item, item.width / 2, item.height / 2);
            mouseMove(item, to.x, to.y, -1, Qt.LeftButton);
            mouseRelease(item, to.x, to.y);
        }
        // The calls `fake` heard since it had heard `from` of them.
        function since(fake, from) {
            return fake.calls.slice(from);
        }

        function test_each_top_bar_button_acts_once_at_let_go_after_a_slip_onto_its_neighbour() {
            const app = make();
            const win = app.face;
            const cases = [
                ["bar.mode", "bar.theme", app.ui, "flipMode"],
                ["bar.theme", "bar.mode", app.ui, "stepTheme"],
                ["bar.smaller", "size.s", app.keyboard, "smaller"],
                ["size.s", "size.m", app.keyboard, "preset 0"],
                ["size.l", "bar.bigger", app.keyboard, "preset 2"],
                ["bar.bigger", "size.l", app.keyboard, "bigger"]
            ];
            for (const [id, next, fake, call] of cases) {
                const [ui, kb] = [app.ui.calls.length, app.keyboard.calls.length];
                const b = keyed(win, id);
                mousePress(b, b.width / 2, b.height / 2);
                compare(since(fake, fake === app.ui ? ui : kb).filter(c => c === call), [], id + ": not at the press");
                const to = onto(b, keyed(win, next));
                mouseMove(b, to.x, to.y, -1, Qt.LeftButton);
                mouseRelease(b, to.x, to.y);
                const heard = since(app.ui, ui).concat(since(app.keyboard, kb));
                compare(heard.filter(c => /^(flipMode|stepTheme|smaller|bigger|preset)\b/.test(c)), [call], id);
            }
            win.visible = false;
        }

        function test_close_with_a_slip_puts_the_keyboard_away_once_and_nothing_behind_gets_a_click() {
            const app = make();
            const calls = app.keyboard.calls.length;
            const close = keyed(app.face, "bar.close");
            slip(close, onto(close, keyed(app.face, "bar.minimise")));
            verify(app.away && !app.face.visible, "put away");
            verify(!app.bubble.visible, "never minimised");
            compare(since(app.keyboard, calls).filter(c => /^(tap|press|letGo)\b/.test(c)), [], "no key got it");
        }

        function test_minimise_with_a_slip_minimises_once_and_nothing_behind_gets_a_click() {
            const app = make();
            const calls = app.keyboard.calls.length;
            const min = keyed(app.face, "bar.minimise");
            slip(min, onto(min, keyed(app.face, "bar.close")));
            verify(app.minimised && app.bubble.visible, "minimised");
            compare(since(app.keyboard, calls).filter(c => /^(tap|press|letGo)\b/.test(c)), [], "no key got it");
            waitForRendering(app.bubble.contentItem);
            const disc = app.bubble.contentItem;
            slip(disc, outside(disc));
            verify(app.face.visible && !app.bubble.visible, "the bubble's slip still brings the keyboard back");
            app.face.visible = false;
        }

        function test_the_grip_starts_a_move_once_and_a_second_click_that_slips_off_ends_it_once() {
            const app = make();
            const win = app.face;
            const grip = keyed(win, "bar.grip");
            slip(grip, outside(grip));
            verify(win.mover.busy, "the first click starts");
            const placed = since(app.keyboard, 0).filter(c => c === "placed").length;
            const pad = keyed(win, "catch");
            slip(pad, outside(pad));
            verify(!win.mover.busy, "the second click ends it");
            compare(since(app.keyboard, 0).filter(c => c === "placed").length, placed + 1, "placed once");
            win.visible = false;
        }

        function test_the_corner_starts_a_resize_once_and_a_catchers_click_that_slips_ends_it_once() {
            const app = make();
            const win = app.face;
            const corner = keyed(win, "bar.corner");
            slip(corner, outside(corner));
            verify(win.mover.busy, "the first click starts");
            const cover = covers(win)[0];
            const c = all(cover.contentItem, o => o.Accessible.id === "catch")[0];
            waitForRendering(cover.contentItem);
            slip(c, outside(c));
            verify(!win.mover.busy, "the catcher's click ends it");
            const heard = since(app.keyboard, 0);
            compare([heard.filter(x => x === "resizeBegin").length, heard.filter(x => x === "resizeEnd").length], [1, 1]);
            win.visible = false;
        }
    }
}

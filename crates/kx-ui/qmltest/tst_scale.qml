// A display scale change over stand-in bridges: a window keeps the size and place its bindings give in Qt units, and
// the keyboard cuts itself at the new scale and asks to be placed again for its size in the new pixels (G11).
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: spyC; SignalSpy {} }
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
        name: "scale"
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
        // How often the keyboard heard `call`.
        function heard(kb, call) {
            return kb.calls.filter(c => c === call).length;
        }
        // A shown keyboard window that the keyboard put at physical `at`, its pointer there too; the real pointer
        // passes through it and the catchers.
        function make(at) {
            const kb = createTemporaryObject(kbC, root, { pointer: at, state: Object.assign({}, Face.states.en, { at: at }) });
            const win = createTemporaryObject(winC, root, { ui: createTemporaryObject(uiC, root), keyboard: kb });
            const mover = all(win.contentItem, o => o.covers !== undefined)[0];
            const covers = [];
            for (let i = 0; i < mover.covers.count; i++)
                covers.push(mover.covers.objectAt(i));
            Pointer.through([win].concat(covers));
            win.place();
            win.visible = true;
            waitForRendering(win.contentItem);
            const rescale = all(win.contentItem, o => o.keep !== undefined)[0];
            return { win: win, kb: kb, mover: mover, rescale: rescale };
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
            const placed = heard(r.kb, "placed");
            const ratio = r.rescale.ratio * 2;
            r.rescale.ratio = ratio;
            tryVerify(() => heard(r.kb, "placed") === placed + 1, 2000, "placed again");
            const size = { w: r.win.wide * ratio, h: r.win.high * ratio, overhang: r.win.overhang / r.win.s };
            compare(r.kb.placedAt, Object.assign({}, at, size), "where it is, its size in the new pixels");
            compare(r.kb.shapeDpr, ratio, "cut at the new scale");
            r.win.visible = false;
        }

        // A move places the keyboard when it ends, so a new scale meanwhile only cuts it again.
        function test_a_new_scale_during_a_move_only_cuts_the_keyboard() {
            const r = make({ x: 120, y: 90 });
            mouseClick(all(r.win.contentItem, o => o.Accessible.id === "bar.grip")[0]);
            verify(r.mover.busy, "moving");
            const placed = heard(r.kb, "placed");
            r.rescale.ratio = r.rescale.ratio * 2;
            tryVerify(() => r.kb.shapeDpr === r.rescale.ratio, 2000, "cut again");
            compare(heard(r.kb, "placed"), placed, "not placed mid-move");
            r.win.finish();
            r.win.visible = false;
        }
    }
}

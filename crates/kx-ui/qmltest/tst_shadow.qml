// The keyboard's shadow over stand-in bridges: it sits around the keyboard, and draws nothing while a live resize runs,
// since each window draws on its own and the shadow would trail the keyboard.
import QtQuick
import QtTest
import "../qml"
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {
                pointer: ({ x: 300, y: 300 })
            }
        }
    }

    TestCase {
        name: "shadow"
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
        // The app once its keyboard shows; the real pointer passes through the keyboard and the catchers.
        function make() {
            const a = createTemporaryObject(appC, root);
            tryVerify(() => a.face.visible, 2000, "the keyboard shows");
            const mover = all(a.face.contentItem, o => o.covers !== undefined)[0];
            const covers = [];
            for (let i = 0; i < mover.covers.count; i++)
                covers.push(mover.covers.objectAt(i));
            Pointer.through([a.face].concat(covers));
            waitForRendering(a.face.contentItem);
            return { app: a, mover: mover };
        }
        function control(a, id) {
            return all(a.face.contentItem, o => o.Accessible.id === id)[0];
        }
        // The shadowed box meets the keyboard window's edges, in whole Qt units as windows take them, with the look's
        // margin around it inside the shadow window.
        function around(a) {
            const g = a.kbGlow;
            const f = a.face;
            const m = f.lk.margin;
            tryCompare(f, "x", f.origin.x, 2000, "the keyboard at its place");
            compare([g.x + g.shade.x, g.y + g.shade.y], [f.x, f.y], "the edges meet");
            compare([g.shade.width, g.shade.height], [f.drawn.width, f.drawn.height], "at the plate's size");
            verify(g.shade.x >= m && g.shade.y >= m, "the margin fits on the left and top");
            verify(g.width - g.shade.x - g.shade.width >= m, "on the right");
            verify(g.height - g.shade.y - g.shade.height >= m, "and at the bottom");
        }

        function test_the_shadow_draws_nothing_while_the_keyboard_resizes() {
            const r = make();
            const a = r.app;
            verify(a.kbGlow.shade.visible, "drawn while still");
            mouseClick(control(a, "bar.corner"));
            verify(r.mover.busy, "resizing");
            verify(!a.kbGlow.shade.visible, "nothing while it resizes");
            a.face.finish();
            verify(a.kbGlow.shade.visible, "drawn again at the size reached");
            mouseClick(control(a, "bar.grip"));
            verify(r.mover.busy, "moving");
            verify(a.kbGlow.shade.visible, "a move only moves both windows, so it stays");
            a.face.finish();
        }

        function test_the_shadow_meets_the_keyboard_in_whole_units_and_after_a_new_scale() {
            const a = make().app;
            a.face.at = Qt.point(100.6, 200.4);
            around(a);
            for (const w of [a.face, a.kbGlow]) {
                const r = all(w.contentItem, o => o.keep !== undefined)[0];
                r.ratio = r.ratio * 2;
            }
            tryVerify(() => a.kbGlow.shade.visible, 2000);
            around(a);
        }
    }
}

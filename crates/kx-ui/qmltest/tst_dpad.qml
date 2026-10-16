// The scroll pad (spec §5.2): each arrow in the middle of its quarter of the ring, Stop's speed dots, a pressed arrow in
// the keys' press colours, taps and the pointer on the pad told to the keyboard, and a dimmed pad that takes no tap.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "pointer.js" as Pointer
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-dark.js" as Dd

Item {
    id: root
    width: 420
    height: 400

    Component { id: padC; DPad {} }
    Component { id: spyC; SignalSpy {} }
    // A window of its own, as the keyboard's, so hiding it hides the pad.
    Component {
        id: winC
        Window {
            width: 200
            height: 200
            flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus
            visible: true
        }
    }

    TestCase {
        name: "dpad"
        when: windowShown

        // Each arrow's way as a unit step on the screen, by its `icons.toml [dpad]` id.
        readonly property var ways: ({ up: [0, -1], right: [1, 0], left: [-1, 0], down: [0, 1] })

        // The pad sits in the test's own window, which the real pointer passes through.
        function initTestCase() {
            Pointer.through([Window.window]);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function spy(target, signalName) {
            return createTemporaryObject(spyC, root, { target: target, signalName: signalName });
        }
        // A working pad at size `s` in look `lk` under `parent`, with `state` merged into its box as Plate does.
        function pad(lk, s, state, parent) {
            const box = Object.assign({ x: 10, y: 10, w: 120 * s, h: 120 * s, name: Face.board.dpad.name }, { dim: false, tip: null, dir: null, fast: false }, state || {});
            const p = createTemporaryObject(padC, parent || root, { box: box, lk: lk, res: Board.resOf(lk, En), s: s });
            waitForRendering(p);
            return p;
        }
        // Arrow or Stop `id`, its icon and its pointer area.
        function arrow(p, id) {
            return Board.all(p, o => o.Accessible.id === "dpad." + id)[0];
        }
        function glyph(a) {
            return Board.all(a, o => o.icon !== undefined && o.tint !== undefined)[0];
        }
        function hit(a) {
            return Board.all(a, o => o.containsMouse !== undefined)[0];
        }
        // The middle of `item` in `other`'s units.
        function middle(item, other) {
            return item.mapToItem(other, item.width / 2, item.height / 2);
        }

        function test_each_arrow_sits_in_the_middle_of_its_quarter_of_the_ring_data() {
            const rows = [];
            for (const [name, lk] of [["native", N.lk], ["et66", E.lk], ["dolch dark", Dd.lk]])
                for (const s of [Face.sizeMin, 1, Face.sizeMax])
                    rows.push({ tag: name + " at " + s, lk: lk, s: s });
            return rows;
        }
        function test_each_arrow_sits_in_the_middle_of_its_quarter_of_the_ring(row) {
            const p = pad(row.lk, row.s), sh = row.lk.shape.dpad;
            const r = p.width / 2, hub = r * (1 - 2 * sh.hub);
            const want = hub + sh.reach * (r - hub);
            verify(want > hub && want < r, "inside the ring");
            for (const id in ways) {
                const c = middle(glyph(arrow(p, id)), p);
                fuzzyCompare(c.x, r + ways[id][0] * want, 0.01, id + " across");
                fuzzyCompare(c.y, r + ways[id][1] * want, 0.01, id + " down");
            }
        }

        // Stop shows the speed under its square: one lit dot for slow and both for fast, unlit ones faded.
        function test_stop_shows_the_speed_with_its_dots() {
            for (const s of [1, Face.sizeMax]) {
                const sh = N.lk.shape.dpad;
                const p = pad(N.lk, s, { dir: "up" });
                const stop = arrow(p, "stop");
                tryCompare(stop, "scale", 1, 2000, "Stop pops in");
                const dots = Board.all(stop, o => o.lit !== undefined && o.radius !== undefined);
                compare(dots.length, 2, "two speeds");
                for (const [fast, lit] of [[false, [true, false]], [true, [true, true]]]) {
                    p.box = Object.assign({}, p.box, { fast: fast });
                    compare(dots.map(d => d.lit), lit, fast ? "fast" : "slow");
                    compare(dots.map(d => d.opacity), lit.map(on => on ? 1 : sh.dot_off));
                    compare(stop.Accessible.description, En.texts[fast ? "badge.fast" : "badge.slow"], "readers hear it");
                }
                const icon = glyph(stop), under = icon.mapToItem(stop, 0, icon.height).y;
                for (const d of dots) {
                    fuzzyCompare(d.width, sh.dot_px * s, 0.001, "its size");
                    verify(Qt.colorEqual(d.color, N.lk.dpad.stop_ink), "in Stop's ink");
                    const c = middle(d, stop);
                    fuzzyCompare(c.y - d.height / 2 - under, sh.dot_gap_px * s, 0.01, "under the square");
                    verify(Math.hypot(c.x - stop.width / 2, c.y - stop.height / 2) + d.width / 2 < stop.width / 2, "inside Stop");
                }
                fuzzyCompare(dots[1].x - dots[0].x, (sh.dot_px + sh.dot_gap_px) * s, 0.001, "side by side");
            }
        }

        // In high contrast an unlit dot is faded as Cycle's are, since a hollow ring that small reads as filled; kx-theme
        // keeps it 3:1 from a lit one.
        function test_in_high_contrast_an_unlit_dot_is_faded_as_cycles_are() {
            for (const s of [Face.sizeMin, 1]) {
                const p = pad(NC.lk, s, { dir: "up" }), stop = arrow(p, "stop"), d = NC.lk.dpad;
                tryCompare(stop, "scale", 1, 2000);
                const [on, off] = Board.all(stop, o => o.lit !== undefined && o.radius !== undefined);
                compare([on.lit, off.lit], [true, false], "slow");
                for (const dot of [on, off])
                    verify(Qt.colorEqual(dot.color, d.stop_ink), "filled in Stop's ink, not a ring");
                compare([on.opacity, off.opacity], [1, NC.lk.shape.dpad.dot_off], "the unlit one faded");
                const ink = Qt.color(d.stop_ink), seen = Qt.tint(d.stop, Qt.rgba(ink.r, ink.g, ink.b, off.opacity));
                verify(!Qt.colorEqual(seen, d.stop_ink), "an unlit dot shows apart from a lit one at " + s);
            }
        }

        // Stop takes only taps on its disc, so a tap just outside the hub, on a corner of Stop's square, takes the arrow.
        function test_a_tap_beside_stop_and_outside_the_hub_takes_the_arrow() {
            for (const s of [Face.sizeMin, 1, Face.sizeMax]) {
                const p = pad(N.lk, s, { dir: "up" }), taps = spy(p, "tapped"), sh = N.lk.shape.dpad;
                tryCompare(arrow(p, "stop"), "scale", 1, 2000);
                const r = p.width / 2, hub = r * (1 - 2 * sh.hub), half = r * (1 - 2 * sh.stop);
                const [dx, dy] = [half * 0.95, half * 0.75], d = Math.hypot(dx, dy);
                verify(d > hub && d > half && (dx - dy) * Math.SQRT1_2 > p.g, "outside the hub and Stop's disc, off the gap");
                mouseClick(p, r + dx, r + dy);
                compare(taps.signalArguments.map(a => a[0]), ["right"], "at " + s);
            }
        }

        // Pressed, an arrow takes the keys' press shade and, in high contrast, their press text, at once; hovered, the
        // keys' tint, so a press shows on a hovered arrow too.
        function test_a_pressed_arrow_takes_the_keys_press_colours() {
            for (const lk of [N.lk, NC.lk]) {
                const p = pad(lk, 1), a = arrow(p, "up"), area = hit(a), at = middle(glyph(a), area);
                mouseMove(area, at.x, at.y);
                tryVerify(() => area.containsMouse, 2000);
                verify(Qt.colorEqual(a.fill, Qt.tint(a.base, lk.palette.hover)), "hovered");
                mousePress(area, at.x, at.y);
                verify(Qt.colorEqual(a.fill, Qt.tint(a.base, lk.palette.press)), "pressed");
                verify(Qt.colorEqual(glyph(a).tint, lk.palette.press_ink || a.ink), "its icon");
                if (!lk.contrast)
                    verify(!Qt.colorEqual(lk.palette.press, lk.palette.hover), "a press shows over the hover");
                mouseRelease(area, at.x, at.y);
                verify(Qt.colorEqual(glyph(a).tint, a.ink), "back at let-go");
                mouseMove(root, root.width - 1, root.height - 1);
            }
        }

        // A tap goes to the keyboard; the state it answers lights the arrow and brings Stop, whose tap stops.
        function test_taps_go_to_the_keyboard_and_its_state_lights_the_way() {
            const p = pad(N.lk, 1), taps = spy(p, "tapped"), up = arrow(p, "up");
            mouseClick(hit(up), middle(glyph(up), hit(up)).x, middle(glyph(up), hit(up)).y);
            compare(taps.signalArguments.map(a => a[0]), ["up"], "the arrow, once");
            verify(Qt.colorEqual(up.base, N.lk.dpad.face), "still until the state says");
            p.box = Object.assign({}, p.box, { dir: "up" });
            tryVerify(() => Qt.colorEqual(up.base, N.lk.dpad.on), 2000, "lit by the state");
            compare(up.Accessible.description, En.texts["badge.slow"], "readers hear its speed");
            const stop = arrow(p, "stop");
            tryCompare(stop, "scale", 1, 2000);
            mouseClick(hit(stop));
            arrow(p, "right").Accessible.pressAction();
            compare(taps.signalArguments.map(a => a[0]), ["up", "", "right"], "Stop is empty, readers press too");
        }

        // A dimmed pad, as while the scroll module is off, takes no tap and says why.
        function test_a_dimmed_pad_takes_no_tap() {
            const p = pad(N.lk, 1, Face.states.en.dpad), taps = spy(p, "tapped"), up = arrow(p, "up");
            mouseClick(hit(up), middle(glyph(up), hit(up)).x, middle(glyph(up), hit(up)).y);
            up.Accessible.pressAction();
            compare(taps.count, 0);
            compare(p.off, En.texts["tip.scroll-off"]);
            verify(Qt.colorEqual(up.base, N.lk.palette.dim), "dimmed");
        }

        // The keyboard hears the pointer come onto the disc once, across arrows, gaps and the hub, and leave it once,
        // also when the window hides with the pointer still there.
        function test_the_pointer_on_the_pad_is_told_once_each_way() {
            const win = createTemporaryObject(winC, root);
            Pointer.through([win]);
            const p = pad(N.lk, 1, null, win.contentItem), told = spy(p, "hovered");
            const said = () => told.signalArguments.map(a => a[0]);
            const move = (x, y) => mouseMove(p, x, y);
            const r = p.width / 2, gap = r * Math.SQRT1_2;
            move(middle(glyph(arrow(p, "up")), p).x, middle(glyph(arrow(p, "up")), p).y);
            tryCompare(told, "count", 1, 2000);
            for (const [x, y] of [[r + gap * 0.9, r - gap * 0.9], [r, r], middle(glyph(arrow(p, "right")), p)].map(q => q.x !== undefined ? [q.x, q.y] : q))
                move(x, y);
            wait(0);
            compare(said(), [true], "on the pad, once");
            move(1, 1);
            tryCompare(told, "count", 2, 2000, "a corner of its cell is off the disc");
            move(r, r / 2);
            tryCompare(told, "count", 3, 2000);
            win.visible = false;
            tryCompare(told, "count", 4, 2000, "the window hid");
            compare(said(), [true, false, true, false]);
            // The pad goes before its window, so nothing in it reads a parent the window took away.
            p.destroy();
            wait(0);
        }

        // Shown again after the pointer left while it was away, the pad tells nothing; a pad gone with the pointer on it
        // tells that it left.
        function test_a_pad_shown_again_or_gone_tells_where_the_pointer_is() {
            const win = createTemporaryObject(winC, root);
            Pointer.through([win]);
            const p = pad(N.lk, 1, null, win.contentItem), told = [];
            p.hovered.connect(on => told.push(on));
            const on = middle(glyph(arrow(p, "up")), p);
            mouseMove(p, on.x, on.y);
            tryVerify(() => told.length === 1, 2000);
            win.visible = false;
            tryVerify(() => told.length === 2, 2000, "the window hid");
            mouseMove(root, root.width - 1, root.height - 1);
            win.visible = true;
            tryVerify(() => p.shown, 2000);
            waitForRendering(p);
            wait(0);
            compare(told, [true, false], "the pointer is elsewhere");
            mouseMove(p, on.x + 1, on.y);
            tryVerify(() => told.length === 3, 2000, "back on the pad");
            p.destroy();
            wait(0);
            compare(told, [true, false, true, false], "gone with the pointer on it");
        }
    }
}

// The motion wiring over the real look fixtures: full timings animate, reduced ones jump and never loop; a press shows
// in full at once in every look.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/assets.js" as Assets
import "pointer.js" as Pointer
import "../../kx-theme/fixtures/look-native-light.js" as F
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as D
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", mark: "Segoe UI", ui: "Segoe UI" },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => k,
        bar: id => id,
        dpad: id => id,
        uid: (group, id) => group + "." + id,
        space: parts => parts.join(" · ")
    })
    readonly property var paintRec: ({ fill: "rec", ink: "rec_ink", skirt: "rec" })
    // English, then Arabic reached by a step forward, then Arabic reached another way.
    readonly property var lang1: Face.states.en.lang
    readonly property var lang2: Object.assign({}, Face.states.ar.lang, { turn: 1 })
    readonly property var lang3: Face.states.ar.lang
    // Every look a press must show in, by name: each theme light and dark, high contrast and reduced motion.
    readonly property var looks: [["Native", F.lk], ["Native dark", ND.lk], ["ET66", E.lk], ["ET66 dark", ED.lk], ["Dolch", D.lk], ["Dolch dark", DD.lk], ["high contrast", H.lk], ["reduced motion", R.lk]]

    // A key of `kind` in a cell at the top left.
    function kv(kind) {
        return { id: "k", x: 10, y: 10, w: 60, h: 56, kind: kind, icon: null, label: null, name: null, checkable: false, dim: false, side: false, tip: null };
    }
    // The English "1" key, which has a second legend, recording or not.
    function ks(rec) {
        const one = Face.states.en.keys["c-0x0002"];
        return Object.assign({}, one, { on: rec, paint: rec ? root.paintRec : one.paint });
    }
    // The first item under `item` for which `ok` holds.
    function find(item, ok) {
        if (ok(item))
            return item;
        for (let i = 0; i < item.children.length; i++) {
            const f = find(item.children[i], ok);
            if (f)
                return f;
        }
        return null;
    }

    Component { id: keyC; Key {} }
    Component { id: textC; KeyText {} }
    Component { id: padC; DPad {} }

    TestCase {
        name: "motion"
        when: windowShown

        // The keys sit in the test's own window, which the real pointer passes through, so a hand on the mouse keeps
        // the hover the tests set.
        function initTestCase() {
            Pointer.through([Window.window]);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function makeKey(lk, rec, kind) {
            return createTemporaryObject(keyC, root, { kv: root.kv(kind || "char"), ks: root.ks(rec), lk: lk, lang: root.lang1, design: Face.board.lang_key, res: root.res, s: 1 });
        }
        function makeText(kind, lk) {
            const c = createTemporaryObject(textC, root, { kv: root.kv(kind), ks: kind === "char" ? root.ks(false) : null, lk: lk, lang: root.lang1, design: Face.board.lang_key, res: root.res, s: 1, ink: "black", icon: "", label: "" });
            c.width = 60;
            c.height = 56;
            return c;
        }
        // The carousel's turn under the language key's face `l`: the animation that slides its names sideways.
        function turnOf(l) {
            const turn = a => a.duration !== undefined && a.property === "x";
            const row = find(l, o => o.data !== undefined && o.data.some(turn));
            return row ? row.data.find(turn) : null;
        }
        // The colour most of picture `img` shows, in 0-255 steps: a key's fill, past its legends and edges.
        function fillOf(img) {
            const [seen, best] = [{}, { n: 0, c: "" }];
            for (let y = 0; y < img.height; y++)
                for (let x = 0; x < img.width; x++) {
                    const p = img.pixel(x, y);
                    const c = [p.r, p.g, p.b].map(v => Math.round(v * 255)).join(",");
                    seen[c] = (seen[c] || 0) + 1;
                    if (seen[c] > best.n)
                        [best.n, best.c] = [seen[c], c];
                }
            return best.c.split(",").map(Number);
        }
        // The largest step in any channel between fills `a` and `b`.
        function step(a, b) {
            return Math.max(...a.map((v, i) => Math.abs(v - b[i])));
        }
        // Spec §10 and G8: the frame drawn next after a press on a hovered key shows the press in full, in its own shade,
        // in every look. Each key hides and the pointer leaves it after, so the next look's key draws and hovers alone.
        function test_a_press_shows_in_full_on_the_next_frame_in_every_look() {
            for (const [name, lk] of root.looks) {
                const k = makeKey(lk, false);
                const rest = grabImage(k);
                mouseMove(k);
                const hovered = grabImage(k);
                verify(!hovered.equals(rest), name + ": the hover is drawn");
                mousePress(k);
                compare(k.sink, 1, name + ": the cap sinks at once");
                const pressed = grabImage(k);
                verify(!pressed.equals(hovered), name + ": the next frame shows the press");
                const [r, h, d] = [fillOf(rest), fillOf(hovered), fillOf(pressed)];
                verify(step(h, d) > 1 && step(r, d) > 1, name + ": its own shade, " + [r, h, d].join(" / "));
                mouseRelease(k);
                mouseMove(root, root.width - 1, root.height - 1);
                k.visible = false;
            }
        }
        // The let-go eases back with full motion and jumps when reduced; a press while it eases shows in full at once.
        function test_the_let_go_eases_and_a_press_cuts_it_short() {
            const k = makeKey(F.lk, false);
            mousePress(k);
            mouseRelease(k);
            verify(k.sink > 0, "the let-go eases back: " + k.sink);
            tryVerify(() => k.sink < 1, 1000, "the rise has begun");
            mousePress(k);
            compare(k.sink, 1, "a press while it eases");
            mouseRelease(k);
            tryCompare(k, "sink", 0, 1000);
            const r = makeKey(R.lk, false);
            mousePress(r);
            mouseRelease(r);
            compare(r.sink, 0, "reduced motion lets go at once");
        }

        function test_dolch_top_inset_follows_the_sink() {
            const k = makeKey(D.lk, false);
            mousePress(k);
            tryCompare(k, "sink", 1, 1000);
            const face = find(k, o => o.inset !== undefined && o.sculpted);
            verify(face, "a sculpted cap");
            compare(face.inset[0], D.lk.look.cap.pressed_inset_px[0]);
            mouseRelease(k);
            tryCompare(k, "sink", 0, 1000);
            compare(face.inset[0], D.lk.look.cap.top_inset_px[0]);
        }

        function test_a_recording_key_glows_and_pulses_only_with_full_motion() {
            const k = makeKey(F.lk, true);
            verify(k.glowing, "the glow runs");
            tryVerify(() => k.lift > 0.5, 1500, "the cap brightens");
            const led = find(k, o => o.pulsing !== undefined);
            verify(led && led.pulsing, "the light pulses");
            tryVerify(() => led.opacity < 0.5, 1500, "the light dims mid-pulse");
            k.ks = root.ks(false);
            tryCompare(k, "lift", 0, 200);
            const d = makeKey(D.lk, true);
            tryVerify(() => d.lift > 0.5 && !Qt.colorEqual(d.capSkirt, d.skirt), 1500, "the Dolch skirt brightens too");
            const r = makeKey(R.lk, true);
            const still = find(r, o => o.pulsing !== undefined);
            verify(!r.glowing && !still.pulsing, "no loop at 0 ms");
            compare(r.lift, 0, "no glow");
            compare(still.opacity, 1, "no pulse");
        }

        function test_colours_fade_with_full_motion() {
            const k = makeKey(F.lk, false);
            const from = String(k.fillColour);
            k.ks = root.ks(true);
            verify(!Qt.colorEqual(k.fillColour, F.lk.palette.rec), "fading");
            tryVerify(() => Qt.colorEqual(k.fillColour, F.lk.palette.rec), 1000);
            verify(from !== String(k.fillColour), "it changed");
        }

        function test_a_language_change_swaps_legends_and_turns_the_carousel() {
            const c = makeText("char", F.lk);
            c.lang = root.lang2;
            verify(c.clip, "clipped while the legends rise");
            const legends = find(c, o => o.opacity < 1);
            verify(legends, "the legends fade in");
            const corner = find(c, o => o.text === root.ks(false).second);
            verify(corner && corner.parent === c.item, "the corner legend changes at once");
            tryVerify(() => !c.clip && legends.opacity === 1, 1000);
            const l = makeText("lang", F.lk);
            const slide = turnOf(l);
            l.lang = root.lang2;
            verify(slide && slide.running, "the carousel turns");
            tryVerify(() => !slide.running, 1000);
            verify(l.clip, "the language key always clips");
            const still = makeText("lang", F.lk);
            const slide2 = turnOf(still);
            still.lang = root.lang3;
            verify(!slide2.running, "no turn when the change came another way");
            const r = makeText("char", R.lk);
            r.lang = root.lang2;
            tryVerify(() => !r.clip, 50, "reduced motion ends at once");
        }

        function test_stop_pops_in_with_a_spring() {
            const box = { x: 200, y: 10, w: 120, h: 120, dim: false, name: "key.dpad", tip: null, dir: null, fast: false };
            const p = createTemporaryObject(padC, root, { box: box, lk: F.lk, res: root.res, s: 1 });
            const stop = find(p, o => o.scale === 0);
            verify(stop, "Stop starts at scale 0");
            p.box = Object.assign({}, box, { dir: "right" });
            verify(stop.scale < 1, "it grows");
            let over = false;
            tryVerify(() => { over = over || stop.scale > 1; return stop.scale === 1; }, 1000);
            verify(over, "the spring overshoots");
            p.box = box;
            tryVerify(() => !stop.visible, 1000);
        }
    }
}

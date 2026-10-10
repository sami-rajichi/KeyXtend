// The motion wiring over the real look fixtures: full timings animate, reduced ones jump and never loop.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as F
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R
import "../../kx-theme/fixtures/look-dolch-light.js" as D

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
        function test_press_eases_in_full_and_jumps_when_reduced() {
            const k = makeKey(F.lk, false);
            mousePress(k);
            verify(k.sink < 1, "the press eases in: " + k.sink);
            tryCompare(k, "sink", 1, 1000);
            mouseRelease(k);
            tryCompare(k, "sink", 0, 1000);
            const r = makeKey(R.lk, false);
            mousePress(r);
            tryCompare(r, "sink", 1, 50);
            mouseRelease(r);
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
            const p = createTemporaryObject(padC, root, { box: { x: 200, y: 10, w: 120, h: 120, dim: false, name: "key.dpad", tip: null }, lk: F.lk, res: root.res, s: 1 });
            const stop = find(p, o => o.scale === 0);
            verify(stop, "Stop starts at scale 0");
            p.dir = 1;
            verify(stop.scale < 1, "it grows");
            let over = false;
            tryVerify(() => { over = over || stop.scale > 1; return stop.scale === 1; }, 1000);
            verify(over, "the spring overshoots");
            p.dir = -1;
            tryVerify(() => !stop.visible, 1000);
        }
    }
}

// Offline check of the motion wiring: full timings animate, reduced ones jump and never loop.
import QtQuick
import QtTest
import "../qml"
import "look-full.js" as F
import "look-reduced.js" as R
import "look-dolch.js" as D

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({ fam: { latin: "Segoe UI", arabic: "Segoe UI" }, icons: Qt.resolvedUrl("../../../target/spike-stage/qt-kb/assets/icons"), ext: "svg" })
    readonly property var paintKey: ({ fill: "key", ink: "legend", skirt: "key" })
    readonly property var paintRec: ({ fill: "rec", ink: "rec_ink", skirt: "rec" })
    readonly property var lang1: ({ prev: "FR", cur: "EN", next: "AR", ar: [false, false, true], rtl: false, space: "English", turn: 0 })
    readonly property var lang2: ({ prev: "EN", cur: "AR", next: "FR", ar: [false, true, false], rtl: true, space: "Arabic", turn: 1 })
    readonly property var lang3: ({ prev: "EN", cur: "AR", next: "FR", ar: [false, true, false], rtl: true, space: "Arabic", turn: 0 })

    function kv(kind) {
        return { id: "k", x: 10, y: 10, w: 60, h: 56, kind: kind, paint: paintKey, name: "a", icon: null, label: null, side: kind === "act" };
    }
    function ks(rec) {
        return { main: "a", second: "A", third: "", main_ar: false, second_ar: false, name: "a", on: rec, rec: rec, paint: rec ? paintRec : paintKey };
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
    Component { id: capC; Caption {} }
    Component { id: pillC; Pill {} }
    Component { id: glowC; Glow {} }

    TestCase {
        name: "motion"
        when: windowShown

        function makeKey(lk, rec, kind) {
            return createTemporaryObject(keyC, root, { kv: root.kv(kind || "char"), ks: root.ks(rec), lk: lk, lang: root.lang1, res: root.res, s: 1 });
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
            tryVerify(() => k.lift > 0.5, 1500, "the cap brightens");
            const led = find(k, o => o.d !== undefined);
            verify(led, "the light");
            tryVerify(() => led.opacity < 0.5, 1500, "the light dims mid-pulse");
            k.ks = root.ks(false);
            tryCompare(k, "lift", 0, 200);
            const d = makeKey(D.lk, true);
            tryVerify(() => d.lift > 0.5 && !Qt.colorEqual(d.capSkirt, d.skirt), 1500, "the Dolch skirt brightens too");
            const r = makeKey(R.lk, true);
            wait(400);
            compare(r.lift, 0, "no glow loop at 0 ms");
            compare(find(r, o => o.d !== undefined).opacity, 1, "no pulse loop at 0 ms");
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
            const c = createTemporaryObject(textC, root, { kv: root.kv("char"), ks: root.ks(false), lk: F.lk, lang: root.lang1, res: root.res, s: 1, ink: "black" });
            c.width = 60;
            c.height = 56;
            c.lang = root.lang2;
            verify(c.clip, "clipped while the legends rise");
            const legends = find(c, o => o.opacity < 1);
            verify(legends, "the legends fade in");
            const corner = find(c, o => o.text === "A");
            verify(corner && corner.parent === c, "the corner legend changes at once");
            tryVerify(() => !c.clip && legends.opacity === 1, 1000);
            const l = createTemporaryObject(textC, root, { kv: root.kv("lang"), ks: null, lk: F.lk, lang: root.lang1, res: root.res, s: 1, ink: "black" });
            const slide = l.data.find(o => o.duration !== undefined && o.property === "x");
            l.lang = root.lang2;
            verify(slide && slide.running, "the carousel turns");
            tryVerify(() => !slide.running, 1000);
            verify(l.clip, "the language key always clips");
            const still = createTemporaryObject(textC, root, { kv: root.kv("lang"), ks: null, lk: F.lk, lang: root.lang1, res: root.res, s: 1, ink: "black" });
            const slide2 = still.data.find(o => o.duration !== undefined && o.property === "x");
            still.lang = root.lang3;
            verify(!slide2.running, "no turn when the change came another way");
            const r = createTemporaryObject(textC, root, { kv: root.kv("char"), ks: root.ks(false), lk: R.lk, lang: root.lang1, res: root.res, s: 1, ink: "black" });
            r.lang = root.lang2;
            tryVerify(() => !r.clip, 50, "reduced motion ends at once");
        }

        function test_stop_pops_in_with_a_spring() {
            const btn = { icon: "square", name: "n" };
            const p = createTemporaryObject(padC, root, { box: { x: 200, y: 10, w: 120, h: 120 }, lk: F.lk, res: root.res, s: 1, buttons: [btn, btn, btn, btn, btn] });
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

        function test_caption_pops_in_pulses_and_shimmers() {
            const c = createTemporaryObject(capC, root, { lk: F.lk, res: root.res, barWidth: 300, barHeight: 56, busyIcon: "audio-lines", wordsIcon: "captions" });
            c.say({ text: "Listening…", rec: true, ar: false, hide_ms: null }, { x: 20, y: 20 });
            const popped = find(c.contentItem, o => o.opacity < 1);
            verify(popped, "it pops in");
            tryVerify(() => find(c.contentItem, o => o.width === F.lk.shape.caption.dot_px && o.opacity < 0.5) !== null, 1500, "the dot pulses");
            c.say({ text: "Transcribing…", rec: false, busy: true, ar: false, hide_ms: null }, { x: 20, y: 20 });
            const sh = find(c.contentItem, o => o.span !== undefined && o.phase !== undefined);
            verify(sh && sh.visible && sh.width > 0, "the shimmer shows");
            tryVerify(() => sh.phase > 0, 1000, "and slides");
            c.visible = false;
            const r = createTemporaryObject(capC, root, { lk: R.lk, res: root.res, barWidth: 300, barHeight: 56, busyIcon: "audio-lines", wordsIcon: "captions" });
            r.say({ text: "Transcribing…", rec: false, busy: true, ar: false, hide_ms: null }, { x: 20, y: 20 });
            const rs = find(r.contentItem, o => o.span !== undefined && o.phase !== undefined);
            wait(300);
            compare(rs.phase, 0, "a still shimmer at 0 ms");
            r.visible = false;
        }

        function test_pill_and_its_shadow_pop_in() {
            const p = createTemporaryObject(pillC, root, { lk: F.lk, res: root.res, label: "Copy", icon: "copy", least: 76, height: 28 });
            const g = createTemporaryObject(glowC, root, { lk: F.lk, box: Qt.rect(20, 120, 72, 28), radius: 14, follow: p.popped });
            p.place({ x: 20, y: 120 });
            g.visible = true;
            p.visible = true;
            const shade = g.contentItem.children[0];
            verify(p.popped.opacity < 1 && p.popped.y > 0, "fading in and rising");
            compare(shade.opacity, p.popped.opacity, "the shadow fades with it");
            compare(shade.y, F.lk.margin + p.popped.y, "and rises with it");
            tryCompare(p.popped, "opacity", 1, 1000);
            compare(shade.scale, 1);
            p.visible = false;
            g.visible = false;
        }
    }
}

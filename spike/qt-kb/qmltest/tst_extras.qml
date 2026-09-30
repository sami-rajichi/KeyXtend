// The caption's and the pill's motion checks, kept as source for P10 and P11 when tst_motion moved to kx-ui (P3).
// The shared parts (Glow, Icon, the tweens) and the look fixtures now live in the product.
import QtQuick
import QtTest
import "../qml"
import "../../../crates/kx-ui/qml"
import "../../../crates/kx-theme/fixtures/look-native-light.js" as F
import "../../../crates/kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({ fam: { latin: "Segoe UI", arabic: "Segoe UI", ui: "Segoe UI" }, icons: Qt.resolvedUrl("../../../assets/icons/lucide"), ext: "svg" })

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

    Component { id: capC; Caption {} }
    Component { id: pillC; Pill {} }
    Component { id: glowC; Glow {} }

    TestCase {
        name: "extras"
        when: windowShown

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
            verify(!rs.sliding, "no loop at 0 ms");
            compare(rs.phase, 0, "a still shimmer");
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

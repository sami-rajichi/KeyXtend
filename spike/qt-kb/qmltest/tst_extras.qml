// The caption's motion checks, kept as source for P11 when tst_motion moved to kx-ui (P3); the pill's went to kx-ui in P10.
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
    }
}

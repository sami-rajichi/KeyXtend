// Glows and lights exist only while they show: a lock key's glow and its light's glow come when it lights and go once
// their fade ends, taking their material along, and only a key that can light, or a theme that shows its off lights,
// holds a light.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({
        fam: { latin: N.lk.look.latin_fonts[0], arabic: N.lk.look.arabic_fonts[0], mark: N.lk.look.mark_fonts[0], ui: N.lk.look.latin_fonts[0] },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => k,
        space: parts => parts.join(" · "),
        noteTip: (name, note) => name + "\n" + note
    })
    // The recording mic's paint.
    readonly property var paintRec: ({ fill: "rec", ink: "rec_ink", skirt: "rec" })
    // Caps Lock off, and on.
    readonly property var capsOff: Face.states.en.keys.caps
    readonly property var capsOn: Face.states["en-caps"].keys.caps

    Component { id: keyC; Key {} }
    // Holds an item until Qt frees it.
    Component { id: heldC; QtObject { property Item item } }

    TestCase {
        name: "glows"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Board key `id` over look `lk`, in the English state unless `ks` is given.
        function makeKey(id, lk, ks) {
            const kv = Face.board.keys.find(k => k.id === id);
            return createTemporaryObject(keyC, root, { kv: kv, ks: ks || Face.states.en.keys[id], lk: lk, lang: Face.states.en.lang, design: Face.board.lang_key, res: root.res, s: 1 });
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
        // The QML type of `o`, such as `Led`.
        function type(o) {
            return String(o).replace(/(_QML.*)?\(.*$/, "");
        }
        // The glows on key `k`: the lock glow and its light's, not the cap's shadows.
        function glows(k) {
            return all(k, o => type(o) === "Soft" && type(o.parent) !== "Shade");
        }
        // The light on key `k`, or null.
        function lightOf(k) {
            return all(k, o => type(o) === "Led")[0] || null;
        }

        function test_an_unlit_key_has_no_glow() {
            for (const lk of [N.lk, E.lk, D.lk])
                for (const kv of Face.board.keys)
                    compare(glows(makeKey(kv.id, lk)).length, 0, kv.id);
        }

        // Where off lights do not show (Native, Dolch), a key holds a light only while lit; ET66 and high contrast
        // show them on every key, which keep them as before (edge case 11).
        function test_only_a_key_that_shows_a_light_holds_one() {
            for (const lk of [N.lk, D.lk]) {
                compare(lightOf(makeKey("c-0x0002", lk)), null, "a character key");
                compare(lightOf(makeKey("caps", lk)), null, "an unlit lock key");
                verify(lightOf(makeKey("caps", lk, root.capsOn)), "a lit lock key");
            }
            for (const lk of [E.lk, H.lk]) {
                const light = lightOf(makeKey("c-0x0002", lk));
                verify(light, "an off light on a character key");
                const dot = all(light, o => type(o) === "QQuickRectangle")[0];
                verify(Qt.colorEqual(dot.color, lk.palette.led_off), "in the off colour");
            }
        }

        // Edge case 9: a key that lights gets both glows, which stay until their fade ends.
        function test_a_lit_key_keeps_its_glows_until_their_fade_ends() {
            const k = makeKey("caps", D.lk);
            k.ks = root.capsOn;
            tryVerify(() => glows(k).length === 2 && glows(k).every(g => g.opacity === 1), 1000, "both glows, fully lit");
            k.ks = root.capsOff;
            compare(glows(k).length, 2, "both stay while they fade");
            tryVerify(() => glows(k).some(g => g.opacity > 0 && g.opacity < 1), 1000, "they fade");
            tryVerify(() => glows(k).length === 0, 1000, "and go once faded");
            compare(lightOf(k), null, "the light goes too, as Dolch shows no off light");
        }

        // Edge case 9: lit again mid-fade, the glows go on from where they were, on the same items.
        function test_lit_again_mid_fade_goes_on_without_a_jump() {
            const k = makeKey("caps", D.lk, root.capsOn);
            k.ks = root.capsOff;
            tryVerify(() => k.lockLit > 0 && k.lockLit < 1, 1000, "mid-fade");
            const [was, items] = [k.lockLit, glows(k)];
            k.ks = root.capsOn;
            compare(k.lockLit, was, "no jump");
            const now = glows(k);
            verify(now.length === 2 && now.every((g, i) => g === items[i]), "the same glows");
            tryCompare(k, "lockLit", 1, 1000);
        }

        // A soft shadow that goes, a glow or a theme's key shade, takes the material Qt made for it along, so lock and
        // theme changes keep no memory.
        function test_a_soft_shadow_that_goes_frees_its_material() {
            const k = makeKey("caps", D.lk, root.capsOn);
            tryVerify(() => glows(k).length === 2, 1000, "both glows");
            const soft = all(k, o => type(o) === "Soft");
            verify(soft.length > 2, "the glows and Dolch's soft key shade");
            const held = soft.map(s => createTemporaryObject(heldC, root, { item: s.material }));
            verify(held.every(h => h.item !== null), "each shadow has its material");
            k.ks = root.capsOff;
            k.lk = N.lk;
            tryVerify(() => held.every(h => h.item === null), 1000, "every material went with its shadow");
        }

        // Edge case 10: high contrast makes no glow, as before.
        function test_high_contrast_makes_no_glow() {
            const k = makeKey("caps", H.lk, root.capsOn);
            compare(glows(k).length, 0);
            verify(lightOf(k), "the light itself shows");
        }

        function test_a_recording_key_holds_a_pulsing_light() {
            const k = makeKey("mic", N.lk);
            compare(lightOf(k), null, "none before it records");
            k.ks = Object.assign({}, k.ks, { paint: root.paintRec });
            const light = lightOf(k);
            verify(light && light.pulsing, "the light pulses");
            k.ks = Face.states.en.keys.mic;
            tryVerify(() => lightOf(k) === null, 1000, "and goes when recording ends");
        }
    }
}

// Icons are one Image each, which the icon provider draws in its tint: no key holds an effect or a layer, as a sweep
// that finds each kind shows, a tint's alpha is the image's opacity, and a new theme, mode or size draws every icon again.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/assets.js" as Assets
import "board.js" as Board
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-dark.js" as ND
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
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        space: parts => parts.join(" · "),
        noteTip: (name, note) => name + "\n" + note
    })

    Component { id: keyC; Key {} }
    Component { id: iconC; Icon {} }

    TestCase {
        name: "icons"
        when: windowShown

        // Any warning fails the test, a refused icon's among them.
        function init() {
            failOnWarning(/./);
        }
        // Board key `id` in the English state over look `lk`, at size `s`.
        function makeKey(id, lk, s) {
            const kv = Face.board.keys.find(k => k.id === id);
            return createTemporaryObject(keyC, root, { kv: kv, ks: Face.states.en.keys[id], lk: lk, lang: Face.states.en.lang, design: Face.board.lang_key, res: root.res, s: s || 1 });
        }
        // The QML type of `o`, such as `Icon`; Qt names a C++ type by its class, such as `QQuickMultiEffect`.
        function type(o) {
            return String(o).replace(/(_QML.*)?\(.*$/, "");
        }
        // The key's icon.
        function iconOf(k) {
            return Board.all(k, o => type(o) === "Icon")[0];
        }
        // Whether `o` draws through an effect of its own: a MultiEffect, a ShaderEffect outside a key's shadow, or a
        // layer.
        function effect(o) {
            const t = type(o);
            const shader = /ShaderEffect/.test(t) && type(o.parent) !== "QQuickRectangularShadow";
            return /MultiEffect$/.test(t) || shader || (o.layer !== undefined && o.layer.enabled);
        }
        // Colour `c` as the provider names a tint: six hex digits, without its alpha.
        function hex(c) {
            return Qt.rgba(c.r, c.g, c.b, 1).toString().slice(1);
        }

        function test_no_key_holds_an_effect_or_a_layer() {
            for (const lk of [N.lk, E.lk, D.lk])
                for (const kv of Face.board.keys) {
                    const k = makeKey(kv.id, lk);
                    const odd = Board.all(k, effect);
                    compare(odd.length, 0, kv.id + " holds " + odd.map(type));
                }
        }

        // The sweep above finds each kind of effect it looks for.
        function test_the_effect_sweep_finds_each_kind() {
            const shader = "ShaderEffect { objectName: 'shader'; property Item source: ShaderEffectSource {} }";
            const kinds = "MultiEffect { objectName: 'multi' } " + shader + " Item { objectName: 'layer'; layer.enabled: true }";
            const held = createTemporaryQmlObject("import QtQuick; import QtQuick.Effects; Item { " + kinds + " }", root);
            const found = Board.all(held, effect).map(o => o.objectName);
            for (const name of ["multi", "shader", "layer"])
                verify(found.indexOf(name) >= 0, name + " is found");
        }

        function test_an_icon_is_one_image_in_its_tint() {
            const i = createTemporaryObject(iconC, root, { icon: "minus", base: root.res.icons, ext: root.res.ext, px: 20, tint: "#FF005FB8" });
            compare(i.children.length, 0, "no effect under it");
            compare(String(i.source), root.res.icons + "/005fb8/minus." + root.res.ext);
            tryCompare(i, "status", Image.Ready);
            compare([i.sourceSize.width, i.sourceSize.height], [20, 20]);
            compare(i.opacity, 1);
        }

        // Edge case 6: a tint's alpha, such as a dimmed key's, fades the icon as before; a share fades it further.
        function test_a_tint_alpha_is_the_icon_opacity() {
            const i = createTemporaryObject(iconC, root, { icon: "minus", base: root.res.icons, ext: root.res.ext, px: 20, tint: "#80005FB8" });
            compare(String(i.source), root.res.icons + "/005fb8/minus." + root.res.ext, "the same image");
            fuzzyCompare(i.opacity, 0x80 / 0xff, 1e-6);
            i.share = 0.5;
            fuzzyCompare(i.opacity, 0x80 / 0xff / 2, 1e-6);
        }

        // Edge case 5: a theme, mode or high-contrast switch draws every icon in its new ink.
        function test_a_look_switch_retints_every_icon() {
            const k = makeKey("bksp", N.lk);
            const icon = iconOf(k);
            tryCompare(icon, "status", Image.Ready);
            for (const lk of [ND.lk, H.lk, E.lk, D.lk, N.lk]) {
                k.lk = lk;
                const ink = hex(Qt.color(lk.palette[k.paint.ink]));
                tryVerify(() => String(icon.source).indexOf("/" + ink + "/") > 0, 1000, "the icon takes " + ink);
                tryCompare(icon, "status", Image.Ready);
            }
        }

        // Edge case 7: a new size asks the provider for the icon at that size, so it stays sharp.
        function test_a_size_change_redraws_icons_at_the_new_size() {
            const k = makeKey("bksp", N.lk, 1);
            const icon = iconOf(k);
            const px = icon.px;
            for (const s of [1.5, 0.75, 1]) {
                k.s = s;
                fuzzyCompare(icon.px, px * s, 1e-9, "the icon's size at " + s);
                const side = Math.round(icon.px);
                compare([icon.sourceSize.width, icon.sourceSize.height], [side, side], "the size it asks at " + s);
                tryCompare(icon, "status", Image.Ready);
            }
        }
    }
}

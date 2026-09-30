// Single parts over the real look fixtures: the cap inside its cell, icons tinted with public Qt Quick only, the hover
// outline in high contrast, dimmed keys and D-pad, plate icons and chips, and bidi marks that take no room.
import QtQuick
import QtQml.Models
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
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
        fam: { latin: N.lk.look.latin_fonts[0], arabic: N.lk.look.arabic_fonts[0], ui: N.lk.look.latin_fonts[0] },
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        bar: id => En.texts["bar." + id],
        dpad: id => En.texts["dpad." + id],
        space: parts => parts.join(" · "),
        dimTip: (name, later) => name + "\n" + later
    })
    // The bundled fonts loaded so far, from where the texts test lays them out.
    property int fontsIn: 0

    Instantiator {
        model: Assets.assets.fonts
        delegate: FontLoader {
            required property string modelData
            source: Qt.resolvedUrl(Assets.root + "/" + modelData)
            onStatusChanged: if (status === FontLoader.Ready) root.fontsIn++
        }
    }

    // Board key `id` and its English state.
    function kv(id) {
        return Face.board.keys.find(k => k.id === id);
    }
    function ks(id) {
        return Face.states.en.keys[id];
    }

    Component { id: keyC; Key {} }
    Component { id: iconC; Icon {} }
    Component { id: chipC; Chip {} }
    Component { id: labelC; Label {} }
    Component {
        id: plateC
        Rectangle {
            property alias label: text
            width: text.implicitWidth
            height: text.implicitHeight
            color: N.lk.palette.plate
            Label { id: text; text: "Esc Ctrl q 1 ! Caps Lock ض ع"; font.pixelSize: N.lk.look.legend_px; color: N.lk.palette.legend }
        }
    }
    Component { id: barC; TopBar {} }
    Component { id: cornerC; Corner {} }
    Component { id: padC; DPad {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "keys"
        when: windowShown

        // The most colour gray text on the near-gray plate may show at an edge; ClearType's fringes reach about 0.5.
        readonly property real grayEdge: 0.05

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function makeKey(id, lk) {
            return createTemporaryObject(keyC, root, { kv: root.kv(id), ks: root.ks(id), lk: lk || N.lk, lang: Face.states.en.lang, res: root.res, s: 1 });
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

        function test_the_cap_sits_half_a_gap_inside_its_cell() {
            const cell = root.kv("esc");
            const k = makeKey("esc");
            const half = N.lk.look.gap_px / 2;
            compare([k.x, k.y, k.width, k.height], [cell.x + half, cell.y + half, cell.w - 2 * half, cell.h - 2 * half]);
        }

        function test_an_icon_takes_its_tint_with_public_qt_quick_only() {
            const tint = "#FF005FB8";
            const i = createTemporaryObject(iconC, root, { icon: "minus", base: root.res.icons, ext: "svg", px: 48, tint: tint });
            waitForRendering(i);
            const shot = grabImage(i);
            // The grab is in device pixels; Lucide's minus is a line across the middle.
            const at = (x, y) => shot.pixel(Math.round(x * shot.width / i.width), Math.round(y * shot.height / i.height));
            verify(Qt.colorEqual(at(24, 24), tint), "the stroke is the tint: " + at(24, 24));
            verify(!Qt.colorEqual(at(24, 4), tint), "and nothing else is");
        }

        function test_a_named_key_shows_its_icon_or_else_its_label() {
            const bksp = makeKey("bksp");
            verify(find(bksp, o => o.icon === N.lk.icons.keys.bksp && o.tint !== undefined), "the Backspace icon");
            const esc = makeKey("esc");
            verify(find(esc, o => o.text === En.texts["label.esc"] && o.visible), "Esc has no icon, so its label");
        }

        function test_high_contrast_outlines_the_key_under_the_pointer() {
            const k = makeKey("c-0x0010", H.lk);
            mouseMove(k, k.width / 2, k.height / 2);
            const mark = find(k, o => o.border !== undefined && o.tint !== undefined);
            verify(mark && mark.visible, "the hover mark shows");
            compare(mark.border.width, H.lk.shape.hc_outline.px);
            verify(Qt.colorEqual(mark.border.color, H.lk.hover_line), "in Highlight");
            const plain = makeKey("c-0x0010");
            mouseMove(plain, plain.width / 2, plain.height / 2);
            compare(find(plain, o => o.border !== undefined && o.tint !== undefined).border.width, 0, "no outline in a theme");
        }

        function test_a_dimmed_key_neither_sinks_nor_marks_the_hover() {
            const k = makeKey("mic");
            verify(Qt.colorEqual(k.fillColour, N.lk.palette.dim), "dimmed");
            mousePress(k);
            verify(!k.down, "a press does not hold it down");
            compare(k.sink, 0, "so it does not sink");
            verify(!find(k, o => o.border !== undefined && o.tint !== undefined).visible, "no hover mark");
            mouseRelease(k);
        }

        function test_chip_icons_take_the_plate_text_colour() {
            const c = createTemporaryObject(chipC, root, { chip: { text: "Hello", icon: D.lk.icons.keys.clip }, lk: D.lk, res: root.res, s: 1, radius: 4 });
            const icon = find(c, o => o.tint !== undefined && o.icon !== undefined);
            verify(Qt.colorEqual(icon.tint, D.lk.palette.legend_2_plate), "legend_2_plate reaches AA on Dolch's chip");
        }

        function test_a_dimmed_key_tip_says_when_it_comes() {
            const k = makeKey("mic");
            const spy = createTemporaryObject(spyC, root, { target: k, signalName: "tip" });
            mouseMove(k, k.width / 2, k.height / 2);
            compare(spy.count, 1);
            compare(spy.signalArguments[0][0], root.res.dimTip(En.texts["key.mic"], En.texts["tip.later"]));
        }

        function test_plate_icons_take_the_plate_text_colour() {
            const bar = createTemporaryObject(barC, root, { box: Face.board.bar, lk: D.lk, res: root.res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.settings.sizes, rtl: false, dark: false });
            const grip = find(bar, o => o.icon === D.lk.icons.bar.grip && o.tint !== undefined);
            verify(Qt.colorEqual(grip.tint, D.lk.palette.legend_2_plate), "the top bar");
            const c = createTemporaryObject(cornerC, root, { lk: D.lk, res: root.res, s: 1 });
            const icon = find(c, o => o.icon === D.lk.icons.bar.corner && o.tint !== undefined);
            verify(Qt.colorEqual(icon.tint, D.lk.palette.legend_2_plate), "the resize corner");
        }

        function test_a_working_dpad_names_each_arrow_and_a_dimmed_one_is_one_button() {
            const box = Object.assign({}, Face.board.dpad, { dim: false, tip: null });
            const pad = createTemporaryObject(padC, root, { box: box, lk: N.lk, res: root.res, s: 1 });
            compare(pad.Accessible.role, Accessible.Grouping, "a group of arrows");
            for (const id of Object.keys(N.lk.icons.dpad).filter(id => id !== "stop"))
                verify(find(pad, o => o.Accessible.name === En.texts["dpad." + id] && !o.Accessible.ignored), id);
            const dim = createTemporaryObject(padC, root, { box: Face.board.dpad, lk: N.lk, res: root.res, s: 1 });
            compare(dim.Accessible.role, Accessible.Button);
            compare(dim.Accessible.description, En.texts["tip.later"]);
            verify(!find(dim, o => o !== dim && o.Accessible.role === Accessible.Button && !o.Accessible.ignored && o.visible), "no arrow on its own");
        }

        function test_text_has_no_colour_fringes() {
            const plate = createTemporaryObject(plateC, root);
            waitForRendering(plate);
            const shot = grabImage(plate);
            let most = 0;
            for (let y = 0; y < shot.height; y++)
                for (let x = 0; x < shot.width; x++) {
                    const c = shot.pixel(x, y);
                    most = Math.max(most, Math.max(c.r, c.g, c.b) - Math.min(c.r, c.g, c.b));
                }
            verify(most < grayEdge, "the most colour at an edge: " + most);
        }

        function test_bidi_isolation_marks_take_no_room() {
            tryCompare(root, "fontsIn", Assets.assets.fonts.length, 5000, "the bundle is laid out by cargo test -p kx-ui --test texts");
            const lists = [N, E, D].map(t => t.lk.look.latin_fonts.concat(t.lk.look.arabic_fonts));
            for (const family of [...new Set([].concat(...lists))]) {
                verify(Qt.fontFamilies().indexOf(family) >= 0, family + " is in");
                const plain = createTemporaryObject(labelC, root, { text: "English · US", "font.family": family });
                const marked = createTemporaryObject(labelC, root, { text: "⁨English⁩ · ⁨US⁩", "font.family": family });
                compare(marked.implicitWidth, plain.implicitWidth, family);
                const ar = createTemporaryObject(labelC, root, { text: "العربية · Arabic", "font.family": family });
                const arMarked = createTemporaryObject(labelC, root, { text: "⁨العربية⁩ · ⁨Arabic⁩", "font.family": family });
                compare(arMarked.implicitWidth, ar.implicitWidth, family + " in Arabic");
            }
        }
    }
}

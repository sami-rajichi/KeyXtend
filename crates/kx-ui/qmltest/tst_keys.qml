// Single parts over the real look fixtures: the cap inside its cell, icons tinted with public Qt Quick only, the hover
// outline in high contrast, dimmed keys, plate icons and chips, and bidi marks that take no room.
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
import "pointer.js" as Pointer
import "fonts.js" as Fonts

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
        uid: (group, id) => group + "." + id,
        dpad: id => En.texts["dpad." + id],
        space: parts => parts.join(" · "),
        noteTip: (name, note) => name + "\n" + note
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
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "keys"
        when: windowShown

        // The most colour gray text on the near-gray plate may show at an edge; ClearType's fringes reach about 0.5.
        readonly property real grayEdge: 0.05
        // Float rounding in a measured position, far below the whole-pixel steps anchors take.
        readonly property real slack: 0.01

        // The keys sit in the test's own window, which the real pointer passes through.
        function initTestCase() {
            Pointer.through([Window.window]);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Key `id` in look `lk` at `size` (1 when not given), its cell scaled as the module scales it.
        function makeKey(id, lk, size) {
            const z = size || 1;
            const c = root.kv(id);
            const kv = Object.assign({}, c, { x: c.x * z, y: c.y * z, w: c.w * z, h: c.h * z });
            return createTemporaryObject(keyC, root, { kv: kv, ks: root.ks(id), lk: lk || N.lk, lang: Face.states.en.lang, design: Face.board.lang_key, res: root.res, s: z });
        }
        // Every item under `item` for which `ok` holds, and the first one.
        function all(item, ok, found) {
            found = found || [];
            if (ok(item))
                found.push(item);
            for (let i = 0; i < item.children.length; i++)
                all(item.children[i], ok, found);
            return found;
        }
        function find(item, ok) {
            return all(item, ok)[0] || null;
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

        // At each preset an icon keeps the anchors' whole pixels, so it draws crisp; a label centres exactly on the face
        // the owner sees, so a fitted label stays in its room.
        function test_an_icon_sits_on_whole_pixels_and_a_label_centres_exactly() {
            for (const lk of [N.lk, E.lk, D.lk])
                for (const size of Face.board.presets) {
                    const keys = Face.board.keys.filter(k => k.icon && k.label).map(k => makeKey(k.id, lk, size));
                    const shown = keys.map(k => centred(k, lk.look.latin_fonts[0] + " " + size + " " + k.kv.id));
                    verify(shown.indexOf("icon") >= 0 && shown.indexOf("label") >= 0, "both kinds: " + shown);
                }
        }
        // Checks key `k`'s icon or label, as said above, and tells which it shows.
        function centred(k, at) {
            const t = find(k, o => o.labelRoom !== undefined);
            const part = t.icon ? find(t, o => o.icon === t.icon && o.tint !== undefined) : find(t, o => o.text === t.label && o.visible);
            // A row lays out once a frame; this one lays out now.
            part.parent.forceLayout();
            if (t.icon) {
                const x = part.mapToItem(t, 0, 0).x;
                verify(Math.abs(x - Math.round(x)) < slack, at + " icon at " + x);
                return "icon";
            }
            const faces = all(k, o => o.base !== undefined && o.g !== undefined && o.visible);
            const face = faces[faces.length - 1];
            const off = part.mapToItem(face, part.width / 2, 0).x - face.width / 2;
            verify(Math.abs(off) < slack, at + " label off centre by " + off);
            return "label";
        }

        function test_a_key_centres_again_when_a_theme_drops_or_brings_back_its_icon() {
            const k = makeKey("tab");
            const bare = Object.assign({}, N.lk, { icons: Object.assign({}, N.lk.icons, { keys: {} }) });
            compare(centred(k, "tab"), "icon");
            k.lk = bare;
            compare(centred(k, "tab without its icon"), "label");
            k.lk = N.lk;
            compare(centred(k, "tab with its icon back"), "icon");
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

        // A chip is a button, so high contrast draws its icon in ButtonText, as its word, never WindowText.
        function test_in_high_contrast_chip_icons_take_the_chip_text_colour() {
            const c = createTemporaryObject(chipC, root, { chip: { text: "Hello", icon: H.lk.icons.keys.clip }, lk: H.lk, res: root.res, s: 1, radius: 4 });
            const icon = find(c, o => o.tint !== undefined && o.icon !== undefined);
            verify(Qt.colorEqual(icon.tint, H.lk.palette.legend), "the chip's own text colour");
        }

        function test_a_dimmed_key_tip_says_when_it_comes() {
            const k = makeKey("mic");
            const spy = createTemporaryObject(spyC, root, { target: k, signalName: "tip" });
            mouseMove(k, k.width / 2, k.height / 2);
            compare(spy.count, 1);
            compare(spy.signalArguments[0][0], root.res.noteTip(En.texts["key.mic"], En.texts["tip.later"]));
        }

        // The lone language's tip goes as the pointer leaves, though a second layout took the note away meanwhile.
        function test_a_tip_goes_on_leave_even_when_its_note_went_meanwhile() {
            const lone = Object.assign({}, root.ks("lang"), { tip: "tip.one-lang" });
            const k = createTemporaryObject(keyC, root, { kv: root.kv("lang"), ks: lone, lk: N.lk, lang: Face.states.en.lang, design: Face.board.lang_key, res: root.res, s: 1 });
            const spy = createTemporaryObject(spyC, root, { target: k, signalName: "tip" });
            mouseMove(k, k.width / 2, k.height / 2);
            compare(spy.count, 1, "the lone language's tip");
            k.ks = root.ks("lang");
            mouseMove(k, -k.width, k.height / 2);
            compare(spy.count, 2, "the tip goes");
            compare(spy.signalArguments[1][0], "");
        }

        function test_plate_icons_take_the_plate_text_colour() {
            const bar = createTemporaryObject(barC, root, { box: Face.board.bar, lk: D.lk, res: root.res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.lang.sizes, dark: false });
            const grip = find(bar, o => o.icon === D.lk.icons.bar.grip && o.tint !== undefined);
            verify(Qt.colorEqual(grip.tint, D.lk.palette.legend_2_plate), "the top bar");
            const c = createTemporaryObject(cornerC, root, { lk: D.lk, res: root.res, s: 1 });
            const icon = find(c, o => o.icon === D.lk.icons.bar.corner && o.tint !== undefined);
            verify(Qt.colorEqual(icon.tint, D.lk.palette.legend_2_plate), "the resize corner");
        }

        // The lit size letter is text in the plate's text colour and the ring marks it as a shape (mock-up .tb.on).
        function test_the_lit_size_letter_reads_as_text_and_the_ring_marks_it() {
            const bar = createTemporaryObject(barC, root, { box: Face.board.bar, lk: D.lk, res: root.res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.lang.sizes, dark: false });
            const [lit, unlit] = [true, false].map(on => find(bar, o => o.lit === on && o.text !== undefined && o.text.length > 0));
            const letter = find(lit, o => o.font !== undefined && o.text === lit.text);
            verify(Qt.colorEqual(letter.color, D.lk.palette.pop_ink), "the letter is plate text");
            const mark = find(lit, o => o.mark === true);
            verify(mark.visible && Qt.colorEqual(mark.color, D.lk.palette.ring), "the ring marks it");
            compare([mark.width, mark.height], D.lk.shape.bar.mark_px.slice(0, 2));
            verify(!find(unlit, o => o.mark === true).visible, "an unlit size has no mark");
        }

        // In high contrast a hovered button is Highlight, so its letter and mark take the hover text colour.
        function test_in_high_contrast_a_hovered_lit_size_marks_in_the_hover_text_colour() {
            const bar = createTemporaryObject(barC, root, { box: Face.board.bar, lk: H.lk, res: root.res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.lang.sizes, dark: false });
            const lit = find(bar, o => o.lit === true && o.text !== undefined);
            const mark = find(lit, o => o.mark === true);
            verify(Qt.colorEqual(mark.color, H.lk.palette.ring), "Highlight on the window");
            mouseMove(lit, lit.width / 2, lit.height / 2);
            tryVerify(() => lit.hot, 1000, "hovered");
            verify(Qt.colorEqual(mark.color, H.lk.sel_ink) && Qt.colorEqual(lit.ink, H.lk.sel_ink), "HighlightText on Highlight");
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

        // One row per family a theme names, so a Windows family this system lacks skips alone (fonts.js).
        function test_bidi_isolation_marks_take_no_room_data() {
            const lists = [N, E, D].map(t => t.lk.look.latin_fonts.concat(t.lk.look.arabic_fonts));
            return [...new Set([].concat(...lists))].map(f => ({ tag: f, family: f }));
        }
        function test_bidi_isolation_marks_take_no_room(row) {
            tryCompare(root, "fontsIn", Assets.assets.fonts.length, 5000, "the bundle is laid out by cargo test -p kx-ui --test texts");
            const family = row.family;
            Fonts.need([family], skip);
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

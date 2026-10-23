// Texts without bidi marks read in their UI's order (ADR-0022): the space bar puts the language first, a reset notice
// names its settings in order, before the count, and a shortcut's tip keeps its keys in order after its name. Text
// items give no character places, so a twin of the drawn text, with the part to find as a link, shows through `linkAt`
// where Qt lays that part out.
pragma ComponentBehavior: Bound
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/order.js" as Order
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D

Item {
    id: root

    // The largest size the keyboard allows, as the pictures use.
    readonly property real s: Face.sizeMax
    // The UI languages by tag, for their texts and direction.
    readonly property var uis: ({ en: En, fr: Fr, ar: Ar })
    // The space bar's cell on the board.
    readonly property var cell: Face.board.keys.find(k => k.kind === "space")
    // The bundled fonts, from where the texts test lays them out.
    property var bundled: []

    width: Math.max(cell.w * s, N.lk.shape.panel.width_px * s)
    height: Face.board.plate.h * s

    // The space bar alone, in face state `st`.
    Component {
        id: spaceC
        Key {
            kv: Object.assign({}, root.cell, { x: 0, y: 0, w: root.cell.w * root.s, h: root.cell.h * root.s })
            design: Face.board.lang_key
            s: root.s
        }
    }
    // A text as the banner draws its message, in the UI font, on one line as on a wide keyboard, or wrapped. It wraps
    // at the panel's whole width, a little wider than the banner's words (`Notice.wordsW`), so its breaks may differ.
    Component {
        id: noteC
        Label {
            required property var lk
            required property var res
            required property bool wrap
            width: wrap ? lk.shape.panel.width_px * root.s : implicitWidth
            wrapMode: wrap ? Text.Wrap : Text.NoWrap
            font.family: res.fam.ui
            font.pixelSize: lk.shape.notice.font_px * root.s
        }
    }
    // A shortcut's tip as the tip window draws it: one line in the UI font, at the tip's size and weight.
    Component {
        id: tipC
        Label {
            required property var lk
            required property var res
            font.family: res.fam.ui
            font.pixelSize: lk.shape.tip.font_px
            font.weight: lk.shape.tip.weight
        }
    }
    // A twin of a drawn text with one part as a link, so `linkAt` finds where Qt lays that part out.
    Component {
        id: twinC
        Text {
            textFormat: Text.StyledText
            opacity: 0
        }
    }

    TestCase {
        name: "order"
        when: windowShown

        // The bundled fonts load once, for every row.
        function initTestCase() {
            root.bundled = Board.loadFonts(root);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
            tryVerify(() => Board.loaded(root.bundled), 5000, "the bundle is laid out by the texts test");
        }
        // `text` safe inside styled text, its no-break spaces kept as such.
        function escaped(text) {
            return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\u00a0/g, "&nbsp;");
        }
        // Where drawn text `label` lays out `part`, the first time it holds it: as `placeAt`.
        function placeOf(label, part) {
            const at = label.text.indexOf(part);
            verify(at >= 0, JSON.stringify(part) + " is drawn");
            return placeAt(label, at, part.length);
        }
        // Where drawn text `label` lays out its `len` characters from `at`: the first line holding them and the middle
        // of their pixels there.
        function placeAt(label, at, len) {
            const part = label.text.slice(at, at + len);
            const [head, tail] = [label.text.slice(0, at), label.text.slice(at + len)];
            const linked = escaped(head) + "<a href=\"part\">" + escaped(part) + "</a>" + escaped(tail);
            const twin = createTemporaryObject(twinC, root, { text: linked, font: label.font, width: label.width, wrapMode: label.wrapMode });
            compare(twin.lineCount, label.lineCount, "the twin breaks its lines as the drawn text does");
            const lineH = twin.contentHeight / twin.lineCount;
            for (let line = 0; line < twin.lineCount; line++) {
                const xs = [];
                for (let x = 0; x < twin.width; x++)
                    if (twin.linkAt(x, (line + 0.5) * lineH) === "part")
                        xs.push(x);
                if (xs.length > 0)
                    return { line: line, x: (xs[0] + xs[xs.length - 1]) / 2 };
            }
            fail(JSON.stringify(part) + " is laid out");
        }
        // Whether place `a` reads before place `b`: an earlier line, else further left, or right in a right-to-left UI.
        function before(a, b, rtl) {
            if (a.line !== b.line)
                return a.line < b.line;
            return rtl ? a.x > b.x : a.x < b.x;
        }
        // Data rows: in each theme and UI language, one per entry `each(theme, tag)` gives.
        function rows(each) {
            const out = [];
            for (const t of [N, E, D])
                for (const tag of Object.keys(root.uis))
                    each(t, tag).forEach(r => out.push(Object.assign({}, r, { lk: t.lk, ui: root.uis[tag], tag: t.lk.look.id + " " + tag + " " + r.tag })));
            return out;
        }

        function test_the_space_bar_reads_the_language_first_data() {
            return rows((t, tag) => Order.texts[tag].spaces.map(e => ({ tag: e.state, entry: e })));
        }
        function test_the_space_bar_reads_the_language_first(row) {
            const e = row.entry;
            const st = Face.states[e.state];
            const key = createTemporaryObject(spaceC, root, { ks: st.keys[root.cell.id], lk: row.lk, lang: st.lang, res: Board.resOf(row.lk, row.ui) });
            const label = Board.all(key, o => o.text === e.text && o.font !== undefined)[0];
            verify(label && label.visible, "the space bar draws the hub's text: " + e.text);
            const rtl = row.ui.lang.rtl;
            const [lang, layout] = [placeOf(label, e.parts[0]), placeOf(label, e.parts[1])];
            // A UI pinned over a layout whose language reads the other way keeps Qt's order: a P13 note (ADR-0022).
            if (st.lang.cur.rtl !== rtl)
                expectFail(row.tag, "a pinned UI over a layout that reads the other way puts the layout first");
            verify(before(lang, layout, rtl), "the language reads first");
        }

        function test_a_reset_notice_names_its_settings_in_order_before_the_count_data() {
            const reset = tag => Order.texts[tag].reset;
            return rows((t, tag) => [{ tag: "one line", entry: reset(tag), wrap: false }, { tag: "wrapped", entry: reset(tag), wrap: true }]);
        }
        function test_a_reset_notice_names_its_settings_in_order_before_the_count(row) {
            const e = row.entry;
            const label = createTemporaryObject(noteC, root, { lk: row.lk, res: Board.resOf(row.lk, row.ui), wrap: row.wrap, text: e.text });
            const [first, second] = e.names.map(n => placeOf(label, n));
            const count = placeOf(label, e.more);
            verify(before(first, second, row.ui.lang.rtl), "the settings read in their order");
            verify(before(second, count, row.ui.lang.rtl), "the settings read before the count");
        }

        function test_a_shortcuts_tip_reads_its_keys_left_to_right_after_its_name_data() {
            return rows((t, tag) => Order.texts[tag].tips.map(e => ({ tag: e.keys, entry: e })));
        }
        // Each key and mark reads left to right, as keyboards print them, and the name reads first in its UI: Arabic
        // starts on the keys, so the line runs left to right with the name on their right.
        function test_a_shortcuts_tip_reads_its_keys_left_to_right_after_its_name(row) {
            const e = row.entry;
            const label = createTemporaryObject(tipC, root, { lk: row.lk, res: Board.resOf(row.lk, row.ui), text: e.text });
            const from = e.text.indexOf(e.keys);
            verify(from >= 0, "the tip holds its keys");
            const marks = [...e.keys].map((c, i) => ({ c: c, at: from + i })).filter(m => m.c.trim().length > 0);
            const keys = marks.map(m => placeAt(label, m.at, 1));
            for (let i = 1; i < keys.length; i++)
                verify(before(keys[i - 1], keys[i], false), marks[i - 1].c + " reads before " + marks[i].c);
            const rtl = row.ui.lang.rtl;
            const edge = rtl ? keys[keys.length - 1] : keys[0];
            verify(before(placeOf(label, e.name), edge, rtl), "the name reads first");
        }
    }
}

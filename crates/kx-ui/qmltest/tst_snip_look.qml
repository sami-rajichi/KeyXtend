// Snip's editor, question and panel in every theme: each text on its ground reaches 4.5:1, the ink swatches' edge and
// ring 3:1, a step's number on each ink 3:1 as large bold text, and the editor's rows fit on one line at its least width
// in English, French and Arabic (edge 19).
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "snipkit.js" as Kit
import "voicekit.js" as V
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 1200
    height: 200

    SnipNames { id: names }
    Component { id: toolsC; SnipTools {} }

    TestCase {
        name: "sniplook"
        when: windowShown

        readonly property var looks: [NL, ND, NC, NR, EL, ED, DL, DD].map(f => f.lk)

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The editor as its rows read it, for look `lk`, language `tag` and view `name`, drawing nothing.
        function editor(lk, tag, name) {
            const sh = lk.shape.snip;
            const sent = { thick: sh.pen_px.map((pen, i) => ({ pen: pen, marker: sh.marker_px[i], disc: sh.disc_px[i], head: sh.head_px[i] })), text: sh.text_px.map(h => ({ height: h, letter: 1 })) };
            const ed = Snip.views[name].edit;
            return {
                lk: lk, res: Kit.resOf(lk, tag), n: names, ed: ed, pic: ed.picture,
                snips: { sent: sent, act: () => {} }, hint: () => {}, keep: () => {}
            };
        }
        // How far row `keys`, with its own gap after it, reaches past the left edge of Copy and Save `keeps`, as drawn.
        function past(keys, keeps) {
            return keys.x + keys.width + keys.spacing - keeps.x;
        }

        function test_each_text_reads_on_its_ground_in_every_theme() {
            for (const lk of looks) {
                const p = lk.palette;
                const id = lk.look.id + " " + lk.mode;
                verify(V.ratio(p.pop_ink, p.pop_bg) >= V.text, id + " text");
                verify(V.ratio(p.pop_muted, p.pop_bg) >= V.text, id + " muted text");
                verify(V.ratio(lk.sel_ink, p.pop_sel) >= V.text, id + " a picked key");
                verify(V.ratio(lk.sel_ink, V.over(p.pop_hover, p.pop_bg)) >= V.text, id + " a key under the pointer");
                verify(V.ratio(p.enter_ink, p.enter) >= V.text, id + " Save");
                verify(V.ratio(p.pop_muted, p.pop_bg) >= V.shapes && V.ratio(p.ring, p.pop_bg) >= V.shapes, id + " an ink's edge and ring");
            }
        }

        function test_a_steps_number_reads_on_every_ink_as_text() {
            for (const lk of looks) {
                for (const ink of names.ids.inks) {
                    const [c, number] = [lk.common[names.ink + ink], lk.common[names.number + ink]];
                    verify(V.ratio(number, c) >= V.text, lk.look.id + " " + ink + ": " + V.ratio(number, c));
                }
            }
        }

        function test_the_rows_fit_on_one_line_at_the_least_width() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load, as each theme draws with them");
            const over = [];
            const check = (id, w, least) => {
                if (w > least)
                    over.push(id + ": " + Math.ceil(w));
            };
            for (const lk of looks) {
                const least = lk.shape.snip.editor_min_px[0];
                for (const tag of Kit.langs) {
                    const id = [lk.look.id, lk.mode, tag].join(" ");
                    for (const name of ["edit", "typing", "crop-framed"]) {
                        const t = createTemporaryObject(toolsC, root, { editor: editor(lk, tag, name), width: least });
                        check(id + " tools", past(t.keys, t.keeps), 0);
                        check(id + " " + name + " options", lk.shape.snip.editor_pad_px[0] * 2 + t.options.width, least);
                    }
                }
            }
            compare(over, []);
        }
    }
}

// The cloud pages fit the panel, which never scrolls (ADR-0008): under its bands each page takes one page of rows, as a
// list does, and lays all it shows inside it; no text is cut or drawn past the panel. Each page shows its longest
// content, in English, French and Arabic, and a typed address's key page fits in every theme too.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "fixtures/face.js" as Face
import "fixtures/voicepanel.js" as Panel
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NH
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 200
    height: 100

    VoiceFit {
        name: "voicecloudfit"
        host: root

        // The cloud pages: the Cloud page, also with an Off row ticked, and each page of the table's rows, also with
        // each row Off, the cards with each table service's long one, the key pages, also with each line's empty
        // Paste, Models with its Local | Cloud chips, and Compare's set text and run with cloud models and the warning.
        readonly property var pasted: Kit.named("en", "servicePasted-")
        // The most lines the send warning may take (T24 wave 2 review).
        readonly property int warnLines: 2
        readonly property var looks: ({ "native-light": NL, "native-dark": ND, "native-contrast": NH, "native-reduced": NR, "et66-light": EL, "et66-dark": ED, "dolch-light": DL, "dolch-dark": DD })
        readonly property var cloudPages: ["cloud", "cloudSealed", "cloudOff", "card", "cardOld"].concat(Kit.named("en", "cloudLong-"), Kit.named("en", "cloudLongOff-"),
            Kit.named("en", "cardLong"), pasted, [
            "service", "serviceRouter", "serviceAssembly", "serviceCustom", "serviceTyped", "serviceLong", "models", "modelsSlow",
            "modelsCloud", "modelsLong", "compareCloud", "comparingCloud", "compareLong"])

        // Pick `o`'s service mark.
        function markOf(o) {
            return Kit.only(Board.all(o, n => n.markId !== undefined));
        }
        // The word of pick `o`'s service mark.
        function word(o) {
            const mark = markOf(o);
            return Kit.only(Board.all(mark, n => n.text === mark.mark.text));
        }

        // Whether `o` lies in a cloud pick, whose name and mark the pick tests measure.
        function inPick(o) {
            for (let at = o.parent; at; at = at.parent)
                if (at.natural !== undefined && at.entry !== undefined)
                    return true;
            return false;
        }

        function test_each_page_keeps_a_page_of_rows_data() {
            const all = [];
            for (const t of ["en", "fr", "ar"])
                cloudPages.forEach(n => all.push({ tag: n + " " + t, name: n, t: t }));
            return all;
        }
        function test_each_page_keeps_a_page_of_rows(d) {
            keeps(make(d.name, d.t));
        }

        function test_no_text_is_cut_data() {
            return test_each_page_keeps_a_page_of_rows_data();
        }
        function test_no_text_is_cut(d) {
            uncut(make(d.name, d.t));
        }

        function test_a_typed_addresss_key_page_fits_in_every_theme_data() {
            const all = [];
            for (const [tag, f] of Object.entries(looks))
                for (const name of ["serviceLong"].concat(pasted))
                    ["en", "fr", "ar"].forEach(t => all.push({ tag: tag + " " + name + " " + t, name: name, lk: f.lk, t: t }));
            return all;
        }
        // The owner's rule (2026-11-21): its address, model and key lines, with their longest answers or a line's empty
        // Paste, read together.
        function test_a_typed_addresss_key_page_fits_in_every_theme(d) {
            const box = make(d.name, d.t, d.lk);
            keeps(box);
            uncut(box);
            compare(["address", "model", "key"].map(id => one(box, "voice.line-" + id).visible), [true, true, true]);
        }

        function test_compares_widest_picks_stay_in_the_body_data() {
            const all = [];
            for (const [tag, f] of Object.entries(looks))
                for (const [name, typed] of [["compareWide", 1], ["compareTyped", 2]])
                    ["en", "fr", "ar"].forEach(t => all.push({ tag: tag + " " + name + " " + t, name, typed, lk: f.lk, t }));
            return all;
        }
        // T24 re-review 1: the offered picks longest by name plus mark, by the pager, beside a typed id at its most or
        // beside Custom's and OpenRouter's, under the warning naming every service. The page keeps its height and every
        // other text shows whole, each chip stays in the body, and only a pick past its room is shortened.
        function test_compares_widest_picks_stay_in_the_body(d) {
            const box = make(d.name, d.t, d.lk);
            keeps(box);
            uncut(box, inPick);
            const p = page(box);
            verify(p.pager > 0, "the pager beside the picks");
            const picks = Board.all(p.body, o => o.visible && o.Accessible.role === Accessible.CheckBox);
            compare(picks.length, p.all.length, "a page of picks");
            const room = rect(box, picks[0].parent);
            const out = picks.filter(o => {
                const r = rect(box, o);
                return r.x < room.x - 0.5 || r.x + r.w > room.x + room.w + 0.5;
            });
            compare(out.map(o => o.entry.name), [], "every chip inside the body");
            const names = picks.map(o => Kit.only(Board.all(o, n => n.text === o.entry.name)));
            const longest = Math.max(...p.all.map(e => e.name.length));
            const typed = p.all.filter(e => e.name.length === longest).map(e => e.name);
            compare(typed.length, d.typed, "the page's typed ids");
            const over = picks.filter(o => o.natural > p.most + 0.5).map(o => o.entry.name);
            verify(typed.every(t => over.indexOf(t) >= 0), "each typed id passes its room");
            const cut = picks.filter((o, i) => names[i].truncated || word(o).truncated).map(o => o.entry.name);
            compare(cut, over, "only a pick past its room is shortened");
            compare(picks.filter(o => o.width > p.most + 0.5).map(o => o.entry.name), [], "each chip within its room");
            const lines = new Set(picks.map(o => Math.round(rect(box, o).y))).size;
            verify(lines <= Math.ceil(box.per / d.lk.shape.voice.pick_across), "a full page's rows, paired: " + lines);
            compare(names.filter(n => n.contentWidth > n.width + 0.5).map(n => n.text), [], "each name within its width");
            verify(picks.every(o => o.Accessible.name.indexOf(o.entry.name) >= 0), "readers hear each name whole");
        }

        function test_the_send_warning_takes_two_lines_at_most_data() {
            return test_compares_widest_picks_stay_in_the_body_data();
        }
        // T24 re-review 3, note 2: the warning naming every service wraps with no line cap, so its lines are counted.
        function test_the_send_warning_takes_two_lines_at_most(d) {
            const box = make(d.name, d.t, d.lk);
            const band = Kit.only(Board.all(box.contentItem, o => o.visible && o.warn === true && o.pieces !== undefined));
            verify(band, "the send warning shows");
            const flow = Kit.only(Board.all(band, o => o.visible && o.flow !== undefined));
            const parts = Board.all(flow, o => o.visible && o.lineCount !== undefined && !!o.text);
            verify(parts.length > 0, "the warning's texts show");
            const one = Math.min(...parts.map(o => o.height / o.lineCount));
            const lines = Math.round(flow.height / one);
            verify(lines >= 1 && lines <= warnLines, "the warning's lines: " + lines);
        }

        function test_a_cut_name_keeps_its_least_and_its_end_data() {
            return test_compares_widest_picks_stay_in_the_body_data();
        }
        // T24 re-review 2: a cut name loses its middle and keeps the least a name keeps; only then is its service's mark
        // cut too, and readers hear both whole.
        function test_a_cut_name_keeps_its_least_and_its_end(d) {
            const box = make(d.name, d.t, d.lk);
            const picks = Board.all(page(box).body, o => o.visible && o.Accessible.role === Accessible.CheckBox);
            const least = d.lk.shape.voice.pick_name_px * box.s;
            for (const o of picks) {
                const name = Kit.only(Board.all(o, n => n.text === o.entry.name));
                const service = word(o).text, tag = o.entry.name + " " + service;
                compare(name.elide, Text.ElideMiddle, "a cut name keeps its end: " + tag);
                verify(name.width >= Math.min(name.implicitWidth, least) - 0.5, "the name keeps its least: " + tag);
                verify(!word(o).truncated || name.width <= least + 0.5, "the mark is cut only past the name's least: " + tag);
                verify(o.Accessible.name.indexOf(service) >= 0, "readers hear the mark whole: " + tag);
            }
        }

        function test_a_whole_mark_keeps_every_letter_at_other_sizes_data() {
            const all = [];
            for (const size of [Face.sizeMin, Face.sizeMax])
                for (const name of ["compareWide", "compareTyped"])
                    ["en", "fr", "ar"].forEach(t => all.push({ tag: size + " " + name + " " + t, size, name, t }));
            return all;
        }
        // T24 re-review 3: at the keyboard's smallest and largest sizes a mark is capped only once its name is at its
        // least, so a whole mark keeps every letter and only a pick past its room is shortened.
        function test_a_whole_mark_keeps_every_letter_at_other_sizes(d) {
            const box = make(d.name, d.t, null, null, d.size);
            compare(box.s, d.size, "the keyboard's size");
            keeps(box);
            uncut(box, inPick);
            const p = page(box), least = box.lk.shape.voice.pick_name_px * box.s;
            const picks = Board.all(p.body, o => o.visible && o.Accessible.role === Accessible.CheckBox);
            const names = picks.map(o => Kit.only(Board.all(o, n => n.text === o.entry.name)));
            picks.forEach((o, i) => {
                const atLeast = names[i].width <= least + 0.5;
                verify(markOf(o).most === Infinity || atLeast, "a cap only once the name is at its least: " + o.entry.name);
                verify(!word(o).truncated || atLeast, "a whole mark keeps every letter: " + o.entry.name);
            });
            const over = picks.filter(o => o.natural > p.most + 0.5).map(o => o.entry.name);
            const cut = picks.filter((o, i) => names[i].truncated || word(o).truncated).map(o => o.entry.name);
            compare(cut, over, "only a pick past its room is shortened");
        }

        function test_a_short_page_keeps_every_name_whole_data() {
            const all = [];
            for (const [tag, f] of Object.entries(looks))
                ["en", "fr", "ar"].forEach(t => all.push({ tag: tag + " " + t, lk: f.lk, t }));
            return all;
        }
        // T24 re-review 2: a short last page whose picks fit at their own widths never pairs them, so nothing is cut.
        function test_a_short_page_keeps_every_name_whole(d) {
            const box = make("compareShort", d.t, d.lk);
            keeps(box);
            verify(!page(box).pair, "its picks do not pair");
            uncut(box);
        }

        function test_a_page_at_the_pairing_edge_keeps_every_name_whole_data() {
            return test_a_short_page_keeps_every_name_whole_data();
        }
        // T24 re-review 3, note 3: three picks too wide to share a row take a full page's rows paired, so they do not
        // pair and every name shows whole.
        function test_a_page_at_the_pairing_edge_keeps_every_name_whole(d) {
            const box = make("compareEdge", d.t, d.lk);
            const p = page(box);
            const flow = Kit.only(Board.all(p.body, o => o.visible && o.flow !== undefined));
            compare(p.rows(flow.children), Math.ceil(box.per / p.across), "a full page's rows, paired");
            verify(!p.pair, "its picks do not pair");
            keeps(box);
            uncut(box);
        }

        function test_compare_fits_at_the_fewest_rows_data() {
            return test_a_short_page_keeps_every_name_whole_data();
        }
        // T24 re-review 2: at the fewest rows a page may show, Compare's page is at least a list's height and grows to
        // hold its set text and picks, so no text is cut or drawn past the panel.
        function test_compare_fits_at_the_fewest_rows(d) {
            const box = make("compareFew", d.t, d.lk, Panel.fewest);
            compare(box.per, Panel.fewest, "the fewest rows");
            verify(page(box).height >= listH(box) - 0.5, "at least a list's height");
            holds(box);
            uncut(box);
        }
    }
}

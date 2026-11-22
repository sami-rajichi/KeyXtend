// The cloud pages fit the panel, which never scrolls (ADR-0008): under its bands each page takes one page of rows, as a
// list does, and lays all it shows inside it; no text is cut or drawn past the panel. Each page shows its longest
// content, in English, French and Arabic, and a typed address's key page fits in every theme too.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
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

    VoiceCase {
        name: "voicecloudfit"
        host: root

        // The cloud pages: the Cloud page, also with an Off row ticked, and each page of the table's rows, also with
        // each row Off, the cards with each table service's long one, the key pages, also with each line's empty
        // Paste, Models with its Local | Cloud chips, and Compare's set text and run with cloud models and the warning.
        readonly property var pasted: Kit.named("en", "servicePasted-")
        readonly property var cloudPages: ["cloud", "cloudSealed", "cloudOff", "card", "cardOld"].concat(Kit.named("en", "cloudLong-"), Kit.named("en", "cloudLongOff-"),
            Kit.named("en", "cardLong"), pasted, [
            "service", "serviceRouter", "serviceAssembly", "serviceCustom", "serviceTyped", "serviceLong", "models", "modelsSlow",
            "modelsCloud", "modelsLong", "compareCloud", "comparingCloud", "compareLong"])

        // The shown children of `item`.
        function kids(item) {
            const out = [];
            for (let i = 0; i < item.children.length; i++)
                if (item.children[i].visible)
                    out.push(item.children[i]);
            return out;
        }
        // The panel's page under the bands: its list, Compare's set text, a card or a key page.
        function page(box) {
            const hit = Kit.only(kids(box.foot.parent).filter(o => o.sheet !== undefined));
            verify(hit, "one page shows");
            return hit;
        }
        // A list's height: one page of rows and the body's padding.
        function listH(box) {
            return (box.pn.body_pad_px[0] + box.pn.body_pad_px[2]) * box.s + box.pageH;
        }
        // Whether `o` lies in a typed line, which keeps its text's end in sight as a text box does.
        function typed(o) {
            for (let at = o.parent; at; at = at.parent)
                if (at.drawn !== undefined && at.masked !== undefined)
                    return true;
            return false;
        }
        // `o`'s box in the panel's coordinates.
        function rect(box, o) {
            const at = o.mapToItem(box.contentItem, 0, 0);
            return { x: at.x, y: at.y, w: o.width, h: o.height };
        }

        // Checks that `box`'s page keeps a list's height and lays its body inside it.
        function keeps(box) {
            const p = page(box);
            verify(p.height <= listH(box) + 0.5, "the page keeps a list's height: " + p.height + " of " + listH(box));
            if (p.rowW !== undefined)
                return;
            const body = p.body !== undefined ? p.body : Kit.only(kids(p).filter(o => o.spacing !== undefined));
            verify(body, "one body");
            const inside = rect(box, body), room = rect(box, p);
            verify(inside.y >= room.y - 0.5, "it starts inside the page");
            verify(inside.y + inside.h <= room.y + room.h + 0.5, "it ends inside the page: " + inside.h + " of " + room.h);
        }
        // Checks that no text on `box` is cut, drawn past the panel or past its own width.
        function uncut(box) {
            const shown = o => o.visible && o.text !== undefined && o.font !== undefined && !!o.text && !typed(o);
            const said = Board.all(box.contentItem, shown);
            verify(said.length > 0, "texts show");
            const cut = said.filter(o => o.truncated);
            compare(cut.map(o => o.text), [], "no text elided");
            const out = said.filter(o => {
                const r = rect(box, o);
                return r.x < -0.5 || r.y < -0.5 || r.x + r.w > box.width + 0.5 || r.y + r.h > box.height + 0.5;
            });
            compare(out.map(o => o.text), [], "every text inside the panel");
            const wide = said.filter(o => o.contentWidth > o.width + 0.5);
            compare(wide.map(o => o.text), [], "every text within its own width");
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
            const looks = { "native-light": NL, "native-dark": ND, "native-contrast": NH, "native-reduced": NR, "et66-light": EL, "et66-dark": ED, "dolch-light": DL, "dolch-dark": DD };
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
    }
}

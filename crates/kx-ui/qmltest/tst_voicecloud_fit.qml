// The cloud pages fit the panel, which never scrolls (ADR-0008): under its bands each page takes one page of rows, as a
// list does, and lays all it shows inside it; no text is cut or drawn past the panel. Each page shows its longest
// content, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit

Item {
    id: root
    width: 200
    height: 100

    VoiceCase {
        name: "voicecloudfit"
        host: root

        // The cloud pages: the Cloud page, the cards with each table service's long one, the key pages, Models with its
        // Local | Cloud chips, and Compare's set text and run with cloud models and the warning.
        readonly property var cloudPages: ["cloud", "cloudOff", "cloudLong", "card", "cardOld"].concat(Kit.named("en", "cardLong"), [
            "service", "serviceRouter", "serviceCustom", "serviceLong", "serviceLongModel", "models", "modelsSlow",
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

        function test_each_page_keeps_a_page_of_rows_data() {
            const all = [];
            for (const t of ["en", "fr", "ar"])
                cloudPages.forEach(n => all.push({ tag: n + " " + t, name: n, t: t }));
            return all;
        }
        function test_each_page_keeps_a_page_of_rows(d) {
            const box = make(d.name, d.t);
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

        function test_no_text_is_cut_data() {
            return test_each_page_keeps_a_page_of_rows_data();
        }
        function test_no_text_is_cut(d) {
            const box = make(d.name, d.t);
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
    }
}

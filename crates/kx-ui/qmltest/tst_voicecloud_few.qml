// The cloud pages at the fewest rows a page may show (ADR-0008): each page is at least a list's height and grows to hold
// its body and its pager, so nothing spills over the bands, the folder line or the footer, and no text is cut, in
// English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "fixtures/voicepanel.js" as Panel

Item {
    id: root
    width: 200
    height: 100

    VoiceFit {
        name: "voicecloudfew"
        host: root

        // A card and each table service's long card parts, the longest key pages, Compare's widest pick, two lists
        // of more than one page and the empty History.
        readonly property var paged: ["card"].concat(Kit.named("en", "cardLong"), ["serviceLong"],
            Kit.named("en", "servicePasted-"), ["compareFew", "history", "cloudLong-1", "historyEmpty"])

        function test_each_paged_page_fits_at_the_fewest_rows_data() {
            const all = [];
            for (const t of ["en", "fr", "ar"])
                paged.forEach(n => all.push({ tag: n + " " + t, name: n, t }));
            return all;
        }
        // T24 re-review 3: a card, a key page, Compare and a list at the fewest rows hold their body and their pager.
        function test_each_paged_page_fits_at_the_fewest_rows(d) {
            const box = make(d.name, d.t, null, Panel.fewest);
            compare(box.per, Panel.fewest, "the fewest rows");
            const p = page(box);
            verify(p.height >= listH(box) - 0.5, "at least a list's height");
            const list = p.rowW !== undefined;
            const body = list ? Kit.only(Board.all(p, o => o.visible && o.Accessible.role === Accessible.List)) : p.body;
            within(box, body, "its body");
            const pagers = Board.all(p, o => o.visible && o.canBack !== undefined);
            if (list)
                compare(pagers.length, box.view.pages.count > 1 ? 1 : 0, "a list of more than one page shows its pager");
            pagers.forEach(o => within(box, o, "its pager"));
            const empty = Board.all(p, o => o.visible && !!p.sheet.empty && o.text === p.sheet.empty);
            empty.forEach(o => within(box, o.parent, "its empty line"));
            uncut(box);
        }

        function test_a_pages_height_holds_while_it_hides_data() {
            return test_each_paged_page_fits_at_the_fewest_rows_data();
        }
        // T24 wave 4 re-review, note 2: the pager's least height and the page's height read no shown flag, so neither
        // changes while the page hides.
        function test_a_pages_height_holds_while_it_hides(d) {
            const box = make(d.name, d.t, null, Panel.fewest);
            const p = page(box);
            const pg = Kit.only(Board.all(p, o => o.least !== undefined && o.canBack !== undefined));
            verify(pg, "one pager");
            const shown = [pg.least, p.height];
            p.visible = false;
            compare([pg.least, p.height], shown, "the same while hidden");
        }
    }
}

// The clipboard panel's preview page over the hub's views: long text paged on demand with no scrolling, each page
// fitting its box, the pages joining back to the whole text and the total shown once known, Arabic in its font from
// the right, an image with its icon to fall back on, files paged across redraws, and the footer sent to the hub.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/clip.js" as Clip
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    readonly property var res: Board.resOf(NL.lk, En)

    Component { id: uiC; FakeUi {} }
    Component { id: boxC; ClipBox {} }
    Component { id: namesC; ClipNames {} }

    TestCase {
        name: "clip_preview"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing preview `name` in English, open and laid out.
        function open(name, res) {
            const ui = createTemporaryObject(uiC, root);
            const box = createTemporaryObject(boxC, root, { ui: ui, lk: NL.lk, res: res || root.res, s: 1, view: Kit.view("en", name), rows: En.settings.pageRows, rtl: false });
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        function acts(box) {
            return box.ui.calls.filter(c => c.startsWith("clipAct"));
        }
        // The shown page of each kind: the text's, whose `page` is its label, and the files'.
        function pager(box) {
            return Board.all(box.contentItem, o => o.visible && o.ends !== undefined)[0];
        }
        // Every page the preview shows, by its page buttons, each checked by `check`.
        function walk(box, check) {
            const seen = [];
            const on = one(box.contentItem, "clip.text-on");
            verify(!one(box.contentItem, "clip.text-back").enabled, "back is off on the first page");
            for (let i = 0; i < 100; i++) {
                verify(waitForPolish(box));
                seen.push(check(pager(box)));
                if (!on.enabled)
                    break;
                mouseClick(on);
            }
            return seen;
        }
        // Readers hear the image preview's one shown graphic, `what`, by the hub's name for it.
        function named(box, what) {
            const seen = Board.all(box.contentItem, o => o.visible && !o.Accessible.ignored && o.Accessible.role === Accessible.Graphic);
            compare(seen.length, 1, what + " is one graphic");
            compare(seen[0].Accessible.name, box.view.preview.name, what);
        }
        // Text page `t` fits its box: nothing cut, nothing past its bottom.
        function fits(t) {
            verify(!t.page.truncated, "cut");
            verify(t.page.contentHeight <= t.height + 0.5, "past the box: " + t.page.contentHeight + " > " + t.height);
            return t.page.text;
        }

        function test_long_text_is_paged_and_the_pages_join_back_to_the_whole() {
            const box = open("text");
            const pages = walk(box, fits);
            verify(pages.length > 1, "a long text takes pages");
            compare(pages.join(""), box.view.preview.text, "nothing lost or doubled");
            verify(Kit.texts(box.contentItem, box.ui.pageText(pages.length, pages.length)).length === 1, "the last page's number");
            mouseClick(one(box.contentItem, "clip.text-back"));
            verify(waitForPolish(box));
            compare(pager(box).page.text, pages[pages.length - 2], "back a page");
            compare(acts(box), [], "the panel pages the text itself");
            const count = Kit.texts(box.contentItem, box.view.preview.chars);
            compare(count.length, 1, "the character count");
            verify(!count[0].Accessible.ignored, "readers hear the count");
            compare(count[0].Accessible.role, Accessible.StaticText);
            compare(count[0].Accessible.name, box.view.preview.chars);
            box.visible = false;
        }

        // The shown page and the next are laid out at once, the next on each turn, and the rest one page per frame.
        function test_a_long_text_lays_out_its_pages_on_demand() {
            const box = open("text");
            const t = pager(box);
            t.cut();
            compare(t.ends.length, 2, "the shown page and the next, no more");
            verify(!t.done, "the rest waits");
            t.at = 1;
            compare(t.ends.length, 3, "a turn lays out the page after it");
            tryVerify(() => t.done, 5000, "the rest comes a page per frame");
            compare(t.ends[t.count - 1], box.view.preview.text.length, "to the end");
            box.visible = false;
        }

        // The total shows only once every page is laid out, so a reader never hears a count that still grows.
        function test_the_page_total_waits_for_the_last_page() {
            const box = open("text");
            const t = pager(box);
            t.cut();
            verify(!t.done, "pages still to lay out");
            compare(Kit.texts(box.contentItem, box.ui.pageText(1, t.count)).length, 0, "no partial total");
            tryVerify(() => t.done, 5000, "the rest comes a page per frame");
            compare(Kit.texts(box.contentItem, box.ui.pageText(1, t.count)).length, 1, "the whole total");
            box.visible = false;
        }

        // A new view of the same files, as each thumbnail or copy brings, keeps the page; other files start on page 1.
        function test_the_files_page_stays_across_views_of_the_same_entry() {
            const box = open("files");
            mouseClick(one(box.contentItem, "clip.text-on"));
            verify(waitForPolish(box));
            compare(pager(box).at, 1);
            box.view = Kit.view("en", "files");
            verify(waitForPolish(box));
            compare(pager(box).at, 1, "a redraw keeps the page");
            const other = Kit.view("en", "files");
            other.preview.paths = other.preview.paths.slice(1);
            box.view = other;
            verify(waitForPolish(box));
            compare(pager(box).at, 0, "other files start on page 1");
            box.visible = false;
        }

        // The preview kinds QML tells apart are the ones the hub's views name.
        function test_the_preview_kinds_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            const views = Object.values(Clip.views.en).filter(v => v.preview);
            const kinds = [...new Set(views.map(v => v.preview.kind))].sort();
            compare(Object.values(names.previews).sort(), kinds);
        }

        function test_arabic_text_pages_in_its_font_from_the_right() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            const arabic = NL.lk.look.arabic_fonts[NL.lk.look.arabic_fonts.length - 1];
            const res = Object.assign({}, root.res, { fam: Object.assign({}, root.res.fam, { arabic: arabic }) });
            const box = open("arabic", res);
            const page = pager(box).page;
            compare(page.font.family, arabic);
            compare(page.effectiveHorizontalAlignment, Text.AlignRight);
            compare(walk(box, fits).join(""), box.view.preview.text);
            box.visible = false;
        }

        // An image keeps its own size at the keyboard's scale, centred; a larger one shrinks to fit the box.
        function test_an_image_keeps_its_size_and_the_kind_icon_waits_for_it() {
            const box = open("image");
            const picture = () => Board.all(box.contentItem, o => o.visible && o.status !== undefined && o.cache !== undefined && o.icon === undefined)[0];
            const kind = () => Board.all(box.contentItem, o => o.visible && o.icon === NL.lk.icons.clip.kind_image);
            tryVerify(() => picture() && picture().status === Image.Ready, 2000, "the picture loads");
            verify(!picture().cache, "never cached");
            const p = picture();
            verify(p.width <= p.sourceSize.width * box.s + 0.5 && p.height <= p.sourceSize.height * box.s + 0.5, "a small picture is never blown up");
            verify(Math.abs(p.x + p.width / 2 - p.parent.width / 2) <= 1, "centred");
            compare(kind().length, 0);
            named(box, "the picture");
            const v = Kit.view("en", "image");
            v.preview.url = Kit.large;
            box.view = v;
            tryVerify(() => picture() && picture().status === Image.Ready && picture().sourceSize.width > p.parent.width, 2000, "the large one loads");
            const big = picture();
            verify(Math.abs(big.paintedWidth - big.parent.width) < 1 || Math.abs(big.paintedHeight - big.parent.height) < 1, "a large picture fills the box");
            verify(big.paintedWidth <= big.parent.width + 0.5 && big.paintedHeight <= big.parent.height + 0.5, "and stays in it");
            const drawing = open("drawing");
            const icon = Board.all(drawing.contentItem, o => o.visible && o.icon === NL.lk.icons.clip.kind_image);
            compare(icon.length, 1, "the kind's icon while the provider has none");
            named(drawing, "the kind's icon");
            box.visible = drawing.visible = false;
        }

        // Many paths take pages and each shows once; a path in an Arabic folder takes the Arabic font, a Latin one not.
        function test_many_files_are_paged_and_each_shows_once_in_its_font() {
            const box = open("files");
            const paths = box.view.preview.paths;
            const shown = walk(box, f => f.shown.slice());
            verify(shown.length > 1, "the paths take pages");
            verify(shown.every(p => p.length <= shown[0].length && p.length > 0));
            compare([].concat(...shown).map(p => p.path), paths.map(p => p.path));
            const last = shown[shown.length - 1];
            verify(last.some(p => p.arabic) && last.some(p => !p.arabic), "the last page holds both kinds");
            for (const p of last) {
                const label = Kit.texts(box.contentItem, p.path);
                compare(label.length, 1, p.path);
                compare(label[0].font.family, p.arabic ? box.res.fam.arabic : box.res.fam.latin, p.path);
            }
            box.visible = false;
        }

        // The footer stays in place: Preview shows disabled, Insert is plain, and Back leads home.
        function test_back_pin_delete_and_insert_act_on_the_previewed_entry() {
            const box = open("text");
            const button = id => one(box.contentItem, id);
            verify(!button("clip.preview").enabled, "already previewing");
            compare(button("clip.insert").label, box.view.words.insert);
            compare(box.view.words.insert, "Insert");
            for (const id of ["clip.back", "clip.pin", "clip.delete", "clip.insert"]) {
                verify(button(id).enabled, id);
                mouseClick(button(id));
            }
            compare(acts(box), ["clipAct back 0", "clipAct pin 0", "clipAct delete 0", "clipAct insert 0"]);
            box.visible = false;
        }
    }
}

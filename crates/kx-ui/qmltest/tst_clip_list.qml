// The clipboard panel's list over the hub's views: five slots on every page, tick numbers, the highlight ring, the
// pager, an Arabic row that reads right to left in place (ADR-0020), thumbnails with the kind's icon to fall back on,
// and the empty line.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    readonly property var res: Board.resOf(NL.lk, En)

    Component { id: uiC; FakeUi {} }
    Component { id: boxC; ClipBox {} }

    TestCase {
        name: "clip_list"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in English, open and laid out; the real pointer passes through it.
        function open(name, res) {
            const ui = createTemporaryObject(uiC, root);
            const box = createTemporaryObject(boxC, root, { ui: ui, lk: NL.lk, res: res || root.res, s: 1, view: Kit.view("en", name), rows: En.settings.pageRows, rtl: false });
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        // Every row slot, padding too, and the filled rows a screen reader meets.
        function slots(box) {
            return Board.all(box.contentItem, o => o.place !== undefined && o.hi !== undefined && o.visible);
        }
        function rowsOf(box) {
            return slots(box).filter(o => !o.Accessible.ignored);
        }

        function test_a_short_page_keeps_five_slots_of_one_height() {
            compare(En.settings.pageRows, 5, "the [ui] page_rows default");
            const box = open("short");
            const all = slots(box);
            compare(all.length, 5, "padded, so the page never jumps");
            compare(rowsOf(box).length, 2);
            verify(all.every(r => r.height === all[0].height && r.height > 0), "one height");
            verify(all.slice(2).every(r => !r.Accessible.name), "blank slots say nothing");
            box.visible = false;
        }

        function test_ticked_rows_show_their_tick_numbers_and_readers_hear_them() {
            const box = open("list");
            const ticks = rowsOf(box).map(r => r.entry.tick);
            compare(ticks, [2, null, 1, 3, null]);
            for (const r of rowsOf(box)) {
                const shown = Board.all(r, o => o.visible && o.font !== undefined && /^\d+$/.test(o.text)).map(o => Number(o.text));
                compare(shown, r.entry.tick === null ? [] : [r.entry.tick], r.entry.title);
                compare(r.Accessible.checked, r.entry.tick !== null);
                compare(r.Accessible.name, r.entry.name);
            }
            box.visible = false;
        }

        function test_a_row_click_sends_a_tick_with_its_place_and_a_blank_slot_takes_none() {
            const box = open("list");
            mouseClick(rowsOf(box)[3]);
            rowsOf(box)[0].Accessible.toggleAction();
            const acts = b => b.ui.calls.filter(c => c.startsWith("clipAct"));
            compare(acts(box), ["clipAct tick 3", "clipAct tick 0"], "a click, then a reader's toggle");
            const short = open("short");
            mouseClick(slots(short)[4]);
            compare(acts(short), [], "a blank slot takes no click");
            box.visible = false;
            short.visible = false;
        }

        function test_the_highlight_ring_sits_on_the_highlighted_row() {
            const box = open("list");
            const list = Board.all(box.contentItem, o => o.ring !== undefined)[0];
            const row = rowsOf(box)[box.view.highlight];
            verify(list.ring.visible);
            const [ring, at] = [list.ring, row].map(o => o.mapToItem(box.contentItem, 0, 0));
            compare([ring.x, ring.y], [at.x, at.y], "on the highlighted row");
            compare(list.ring.border.color, row.mark, "in the row's mark colour");
            box.visible = false;
        }

        function test_the_page_buttons_send_pages_and_back_is_off_on_the_first() {
            const box = open("list");
            const back = one(box.contentItem, "clip.page-back");
            const on = one(box.contentItem, "clip.page-on");
            verify(!back.enabled && on.enabled, "page 1 of 3");
            mouseClick(back);
            mouseClick(on);
            compare(box.ui.calls.filter(c => c.startsWith("clipAct")), ["clipAct page 1"]);
            verify(Kit.texts(box.contentItem, box.ui.pageText(1, 3)).length === 1, "1 of 3");
            box.visible = false;
        }

        // Arabic flips text only (ADR-0020): the title reads right to left in the Arabic font, and the tick box, the
        // thumbnail and the trailing marks keep their places.
        function test_an_arabic_row_reads_right_to_left_in_its_font_and_keeps_its_places() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            const arabic = NL.lk.look.arabic_fonts[NL.lk.look.arabic_fonts.length - 1];
            const res = Object.assign({}, root.res, { fam: Object.assign({}, root.res.fam, { arabic: arabic }) });
            const box = open("list", res);
            const [latin, ar] = [rowsOf(box)[0], rowsOf(box)[2]];
            const title = r => Kit.texts(r, r.entry.title)[0];
            compare(title(ar).font.family, arabic);
            compare(title(ar).effectiveHorizontalAlignment, Text.AlignRight, "reads right to left");
            compare(title(latin).effectiveHorizontalAlignment, Text.AlignLeft);
            compare(title(latin).font.family, root.res.fam.latin);
            const tickX = r => Board.all(r, o => o.visible && o.text === String(r.entry.tick))[0].mapToItem(r, 0, 0).x;
            verify(tickX(ar) < title(ar).mapToItem(ar, 0, 0).x, "the tick box stays first");
            compare(tickX(ar), tickX(latin), "in the same place as on a Latin row");
            box.visible = false;
        }

        function test_rows_show_their_kind_icon_pin_and_expiry_chip() {
            const box = open("list");
            const icons = NL.lk.icons.clip;
            const shown = r => Board.all(r, o => o.visible && o.icon !== undefined && o.tint !== undefined).map(o => o.icon);
            const [text, , , media, link] = rowsOf(box);
            verify(shown(text).indexOf(icons.kind_text) >= 0);
            verify(shown(media).indexOf(icons.kind_media) >= 0);
            verify(shown(link).indexOf(icons.pin) >= 0, "a pinned row shows the pin");
            verify(shown(text).indexOf(icons.pin) < 0);
            compare(Kit.texts(media, box.view.words.expires).length, 1, "media that goes tonight wears its chip");
            compare(Kit.texts(link, box.view.words.expires).length, 0);
            box.visible = false;
        }

        // The provider gives one clear pixel when it has no picture, as for an old round; the row then shows its kind.
        function test_a_thumbnail_shows_and_falls_back_to_the_kind_icon() {
            const box = open("list");
            const image = rowsOf(box)[1];
            const picture = () => Board.all(image, o => o.visible && o.status !== undefined && o.cache !== undefined && o.icon === undefined)[0];
            const kind = () => Board.all(image, o => o.visible && o.icon === NL.lk.icons.clip.kind_image);
            tryVerify(() => picture() && picture().status === Image.Ready, 2000, "the thumbnail loads");
            verify(!picture().cache, "never cached, so an old round's texture goes");
            compare(kind().length, 0, "no icon over the picture");
            const v = Kit.view("en", "list");
            v.rows[1].thumb = Kit.clear;
            box.view = v;
            tryVerify(() => kind().length === 1, 2000, "a clear pixel shows the kind");
            v.rows[1].thumb = null;
            box.view = JSON.parse(JSON.stringify(v));
            tryVerify(() => kind().length === 1 && !picture(), 2000, "no thumbnail shows the kind");
            box.visible = false;
        }

        function test_an_empty_history_shows_one_line_with_its_icon_and_no_rows() {
            const box = open("empty");
            compare(slots(box).length, 0, "no empty rows");
            compare(Kit.texts(box.contentItem, box.view.empty).length, 1);
            const icon = Board.all(box.contentItem, o => o.visible && o.icon === NL.lk.icons.clip.title);
            verify(icon.length >= 2, "the title's icon and the empty line's");
            verify(!one(box.contentItem, "clip.page-on"), "no pager");
            box.visible = false;
        }
    }
}

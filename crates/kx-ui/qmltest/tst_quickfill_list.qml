// Quick-fill's list over the hub's views: five slots on every page, each row's buttons by its kind with Open last, a
// click on the row itself sends nothing, a button sends its id with the shown round, the filters and the search line,
// By label's counts, the empty notebook's invitation, and an Arabic row that reads right to left in place.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/fill.js" as Fill
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }
    Component { id: namesC; QuickFillNames {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "quickfill_list"
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
        function open(name) {
            const props = { ui: createTemporaryObject(uiC, root), fill: createTemporaryObject(fillC, root), lk: NL.lk, res: Board.resOf(NL.lk, En), s: 1, view: Kit.view("en", name), rows: En.settings.pageRows };
            const box = createTemporaryObject(boxC, root, props);
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        function button(box, id) {
            const b = one(box.contentItem, id);
            verify(b, id + " shows once");
            return b;
        }
        // The ids of a row's shown buttons, in order from the left.
        function idsOf(row) {
            const shown = Board.all(row, o => o.visible && o.uid !== undefined && o.label !== undefined && o.uid.length > 0);
            return shown.sort((a, b) => a.mapToItem(row, 0, 0).x - b.mapToItem(row, 0, 0).x).map(o => o.uid.split("-").pop());
        }

        function test_the_click_kinds_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            compare(names.acts, Fill.acts);
        }

        function test_a_short_page_keeps_five_slots_of_one_height() {
            compare(En.settings.pageRows, 5, "the [ui] page_rows default");
            const box = open("short");
            const all = Kit.slots(box.contentItem);
            compare(all.length, 5, "padded, so the page never jumps");
            compare(Kit.filled(box.contentItem).length, 2);
            verify(all.every(r => r.height === all[0].height && r.height > 0), "one height");
            verify(all.slice(2).every(r => r.Accessible.ignored && !r.Accessible.name), "blank slots say nothing");
            box.visible = false;
        }

        function test_a_rows_buttons_come_by_its_kind_with_open_last() {
            const box = open("list");
            const rows = Kit.filled(box.contentItem);
            compare(rows.map(idsOf), [["user", "password", "both"], ["open"], ["open"], ["insert"], ["open"]]);
            compare(button(box, "fill.row-0-both").label, "Both");
            compare(button(box, "fill.row-0-both").Accessible.description, "GitHub", "readers hear whose button it is");
            box.visible = false;
        }

        // A slip on a row never types, so only its buttons act.
        function test_a_click_on_the_row_itself_sends_nothing() {
            const box = open("list");
            for (const r of Kit.filled(box.contentItem))
                mouseClick(r, 4, r.height / 2);
            compare(box.fill.calls, []);
            box.visible = false;
        }

        // The vault drops a click on a row that moved since it was drawn, so each click names the round it saw.
        function test_a_button_sends_its_id_and_place_with_the_shown_round() {
            const box = open("list");
            const drawn = box.view.round;
            mouseClick(button(box, "fill.row-0-password"));
            mouseClick(button(box, "fill.row-3-insert"));
            box.view = Object.assign({}, box.view, { round: drawn + 1 });
            verify(waitForPolish(box), "laid out again");
            mouseClick(button(box, "fill.row-1-open"));
            mouseClick(button(box, "fill.page-on"));
            compare(box.fill.calls, ["row password 0", "row insert 3", "row open 1", "page 1"]);
            compare(box.fill.rounds, [drawn, drawn, drawn + 1, drawn + 1]);
            box.visible = false;
        }

        function test_the_highlight_ring_sits_on_the_highlighted_row() {
            const box = open("list");
            const list = Board.all(box.contentItem, o => o.ring !== undefined && o.visible)[0];
            const row = Kit.filled(box.contentItem)[box.view.highlight];
            verify(list.ring.visible);
            const [ring, at] = [list.ring, row].map(o => o.mapToItem(box.contentItem, 0, 0));
            compare([ring.x, ring.y], [at.x, at.y], "on the highlighted row");
            compare(row.Accessible.selected, true);
            box.visible = false;
        }

        function test_the_filter_chips_send_their_id_and_mark_the_shown_one() {
            const box = open("list");
            const chips = box.view.filters.map(f => button(box, "fill.filter-" + f.id));
            compare(chips.map(c => c.Accessible.checked), [true, false, false, false, false, false, false]);
            const check = Board.all(chips[0], o => o.visible && o.icon === NL.lk.icons.fill.shown);
            compare(check.length, 1, "the shown chip is marked by more than its colour");
            mouseClick(chips[6]);
            mouseClick(chips[2]);
            compare(box.fill.calls, ["filter labels 0", "filter logins 0"]);
            box.visible = false;
        }

        function test_the_search_line_shows_its_hint_then_the_typed_text_and_never_takes_focus() {
            const box = open("list");
            const search = button(box, "fill.search");
            box.aimed = true;
            compare(Kit.texts(search, box.view.search.hint).length, 1, "the hint while empty");
            compare(Board.all(search, o => o.visible && o.caret === true).length, 1, "beside the caret, as it alone says what the line does");
            const none = open("none");
            verify(Kit.texts(button(none, "fill.search"), "zzz")[0], "the typed text");
            verify(Kit.texts(none.contentItem, none.view.sheet.none)[0], "no match says so");
            compare(Kit.filled(none.contentItem).length, 0);
            verify(box.flags & Qt.WindowDoesNotAcceptFocus, "letters come from our keyboard, never focus");
            box.visible = none.visible = false;
        }

        // Only an aimed line looks as if it takes the letters: caret, underline and ring together (ADR-0040).
        function test_only_an_aimed_search_draws_its_caret_and_ring() {
            const box = open("list");
            const search = button(box, "fill.search");
            const carets = () => Board.all(search, o => o.visible && o.caret === true).length;
            compare(carets(), 0, "no caret");
            verify(Qt.colorEqual(search.border.color, NL.lk.palette.pop_line), "no ring");
            compare(Kit.texts(search, box.view.search.hint).length, 1, "its hint still says what it does");
            box.aimed = true;
            compare(carets(), 1, "the caret");
            verify(Qt.colorEqual(search.border.color, NL.lk.palette.ring), "the ring");
            box.visible = false;
        }

        // A press anywhere on the panel asks the aim, once and at the press, whatever lies under it (ADR-0040).
        function test_a_press_anywhere_asks_the_aim_at_the_press() {
            const box = open("list");
            Board.apart(box.contentItem);
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "aiming" });
            const row = Kit.filled(box.contentItem)[0];
            const parts = [[button(box, "fill.search"), "the search"], [row, "a row"], [button(box, "fill.row-0-both"), "a row's button"], [button(box, "fill.page-on"), "a page button"]];
            let asked = 0;
            for (const [part, what] of parts) {
                mousePress(part, 4, part.height / 2);
                compare(spy.count, ++asked, what + " at the press");
                mouseRelease(part, 4, part.height / 2);
                compare(spy.count, asked, what + " once");
            }
            tryVerify(() => box.body.scale === 1, 2000, "popped in, so the panel reaches its corner");
            mousePress(box.contentItem, 1, box.height - 1);
            compare(spy.count, ++asked, "blank space");
            mouseRelease(box.contentItem, 1, box.height - 1);
            compare(spy.count, asked, "blank space once");
            box.visible = false;
        }

        // The view holds the search's caret and QML only draws it; a click sends its place, and the editor takes no focus.
        function test_the_searchs_caret_stands_where_the_view_says_and_a_click_places_it() {
            const box = open("list");
            box.aimed = true;
            const search = button(box, "fill.search");
            const said = at => {
                const v = Kit.view("en", "list");
                v.search = { text: "WWWWWW", hint: v.search.hint, arabic: false, rtl: false, caret: at };
                box.view = v;
            };
            const place = () => Board.all(search, o => o.visible && o.caret === true)[0].mapToItem(search, 0, 0).x;
            said(2);
            const low = place();
            said(4);
            verify(place() > low, "it moves as the view says");
            const ed = Board.all(search, o => o.cursorPosition !== undefined)[0];
            const at = ed.mapToItem(search, ed.positionToRectangle(1).x, 0);
            mouseClick(search, at.x + 1, search.height / 2);
            mouseClick(search, search.width - 2, search.height / 2);
            compare(box.fill.calls, ["caret 1", "caret 6"], "on a letter, then past the end");
            compare(box.fill.rounds, [box.view.round, box.view.round], "with the round drawn");
            compare(ed.activeFocus, false, "focus stays with the app");
            box.visible = false;
        }

        // A reader's press has no pointer, so the line itself asks for the aim.
        function test_a_readers_press_on_the_search_asks_the_aim_once() {
            const box = open("list");
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "aiming" });
            button(box, "fill.search").Accessible.pressAction();
            compare(spy.count, 1);
            compare(box.fill.calls, [], "it sends the panel nothing");
            box.visible = false;
        }

        function test_the_empty_notebook_invites_a_first_entry() {
            const box = open("empty");
            verify(Kit.texts(box.contentItem, box.view.empty)[0], "the invitation");
            const invite = button(box, "fill.invite");
            compare(invite.label, box.view.buttons["new"]);
            mouseClick(invite);
            compare(box.fill.calls, ["button new 0"]);
            box.visible = false;
        }

        // The empty notebook also offers Import, beside New.
        function test_the_empty_notebook_offers_import_beside_new() {
            const box = open("empty");
            const [invite, keepass] = [button(box, "fill.invite"), button(box, "fill.import")];
            compare(keepass.label, box.view.buttons["import"]);
            compare(keepass.tone, "ghost", "New stays the first thing to do");
            compare(Board.all(keepass, o => o.visible && o.tint !== undefined && o.icon === NL.lk.icons.fill.keepass).length, 1);
            const [p, q] = [invite, keepass].map(b => b.mapToItem(box.body, 0, 0));
            verify(p.x + invite.width <= q.x, "beside New");
            compare(p.y, q.y, "on its line");
            mouseClick(keepass);
            compare(box.fill.calls, ["button import 0"]);
            compare(box.fill.rounds, [box.view.round]);
            compare(one(open("short").contentItem, "fill.import"), null, "a notebook with entries has it on the Backup page");
            box.visible = false;
        }

        function test_by_label_lists_each_label_with_its_count_and_opens_one() {
            const box = open("labels");
            const rows = Kit.filled(box.contentItem);
            compare(rows.length, 4);
            verify(Kit.texts(rows[0], "Email")[0] && Kit.texts(rows[0], "3 values")[0], "the label and its count");
            mouseClick(button(box, "fill.row-2-open"));
            compare(box.fill.calls, ["row open 2"]);
            box.visible = false;
        }

        // The Arabic name reads from the right in the Arabic font, while the row keeps the Latin rows' layout.
        function test_an_arabic_row_reads_right_to_left_in_place() {
            const box = open("list");
            const [latin, arabic] = [Kit.filled(box.contentItem)[1], Kit.filled(box.contentItem)[4]];
            const name = Kit.texts(arabic, arabic.slot.title.text)[0];
            compare(name.effectiveHorizontalAlignment, Text.AlignRight);
            compare(name.font.family, box.res.fam.arabic);
            const latinName = Kit.texts(latin, latin.slot.title.text)[0];
            compare(name.mapToItem(arabic, 0, 0).x, latinName.mapToItem(latin, 0, 0).x, "the text box stays put");
            const at = id => button(box, id).mapToItem(box.contentItem, 0, 0).x;
            compare(at("fill.row-4-open"), at("fill.row-1-open"), "so do the buttons");
            box.visible = false;
        }
    }
}

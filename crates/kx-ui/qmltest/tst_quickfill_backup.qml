// Quick-fill's Backup page over the hub's views: each place's row with its kind, name and state, Replace's question for
// the whole page, the buttons below the rows and the memory checks switch; Arabic flips the text only; and every backup
// page draws only inks the theme checks. Remove place's question has tst_quickfill_backup_row.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 200
    height: 100

    readonly property var langs: ({ en: En, fr: Fr, ar: Ar })
    // The buttons below the rows, each by its view's click.
    readonly property var below: ["back-up-now", "add-place", "bring-back", "change-password", "import"]
    // The backup pages' views.
    readonly property var pages: ["backup", "backup-off", "backup-paged", "backup-none", "remove", "remove-long", "replace", "imported", "places", "places-refused", "drives", "folders", "files", "keepass", "pass-new", "pass-check", "pass-wrong", "pass-open", "pass-own", "pass-foreign", "pass-keepass", "found", "foreign", "foreign-undated"]

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }

    TestCase {
        name: "quickfill_backup"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in language `tag` (English by default) and `look` (Native light by default), laid out.
        function open(name, tag, look) {
            const ui = langs[tag || "en"];
            const lk = (look || NL).lk;
            const props = { ui: createTemporaryObject(uiC, root), fill: createTemporaryObject(fillC, root), lk: lk, res: Board.resOf(lk, ui), s: 1, view: Kit.view(tag || "en", name), rows: En.settings.pageRows, rtl: ui.lang.rtl };
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
        // Where `item` sits on the panel, measured in the box, whose pop-in scale would leave a rounding trace.
        function place(box, item) {
            return item.mapToItem(box.body, 0, 0);
        }

        function test_a_place_row_shows_its_kind_its_name_and_its_state() {
            const box = open("backup");
            const rows = Kit.filled(box.contentItem);
            compare(rows.length, 3);
            const places = box.view.sheet.places;
            const ic = NL.lk.icons.fill;
            [ic.place_cloud, ic.place_removable, ic.place_drive].forEach((icon, i) => {
                verify(Kit.texts(rows[i], places[i].name.text)[0], places[i].name.text + " shows");
                verify(Kit.texts(rows[i], places[i].state)[0], places[i].state + " shows");
                compare(Board.all(rows[i], o => o.visible && o.tint !== undefined && o.icon === icon).length, 1, "its kind's icon");
                compare(rows[i].Accessible.name, places[i].name.text, "readers hear its name");
                compare(rows[i].Accessible.description, places[i].state, "and its state");
            });
            compare(places[0].state, "Updated today 10:42");
            compare(places[2].state, "Waiting");
            const remove = button(box, "fill.row-1-remove");
            compare(remove.label, "Remove");
            compare(remove.Accessible.description, places[1].name.text, "readers hear which place");
            compare(Board.all(remove, o => o.visible && o.tint !== undefined && o.icon === ic.remove).length, 1, "with its icon");
            mouseClick(remove);
            compare(box.fill.calls, ["row remove 1"]);
            compare(box.fill.rounds, [box.view.round]);
            verify(Kit.texts(box.contentItem, box.view.waiting.text)[0], "the foot says what waits");
            box.visible = false;
        }

        function test_each_button_below_the_rows_sends_its_click_with_the_round() {
            const box = open("backup");
            const last = Kit.slots(box.contentItem).pop();
            const end = place(box, last).y + last.height;
            const foot = root.below.concat(["back", "checks"]).map(id => button(box, "fill." + id));
            foot.forEach(b => verify(place(box, b).y >= end, b.uid + " sits below the rows"));
            root.below.concat(["back"]).forEach(id => mouseClick(button(box, "fill." + id)));
            compare(box.fill.calls, root.below.concat(["back"]).map(id => "button " + id + " 0"));
            compare(new Set(box.fill.rounds).size, 1);
            compare(box.fill.rounds[0], box.view.round);
            box.visible = false;
        }

        // A switch reads as on or off, by more than colour, and sends the state it turns to.
        function test_the_memory_checks_switch_reads_and_sends_its_new_state() {
            const ic = NL.lk.icons.fill;
            for (const [name, on, icon, sent] of [["backup", true, ic.checks_on, "checks 0"], ["backup-off", false, ic.checks_off, "checks 1"]]) {
                const box = open(name);
                const sw = button(box, "fill.checks");
                compare(sw.label, box.view.words.checks);
                compare(sw.Accessible.role, Accessible.CheckBox);
                compare(sw.Accessible.checked, on, name + " reads as it is");
                compare(sw.tone, on ? "shown" : "ghost");
                compare(sw.icon, icon);
                mouseClick(sw);
                compare(box.fill.calls, [sent]);
                box.visible = false;
            }
        }

        // Replace asks about the whole page in the rows' place; the hold's note stays in the foot.
        function test_replace_asks_for_the_whole_page() {
            const box = open("replace");
            const said = Kit.texts(box.contentItem, box.view.sheet.replace.text);
            compare(said.length, 1, "it asks once");
            compare(said[0].elide, Text.ElideNone, "only a row's question is capped");
            compare(Kit.filled(box.contentItem).length, 0, "the rows give way");
            verify(Kit.texts(box.contentItem, box.view.note)[0], "the hold's note shows");
            root.below.concat(["back", "checks"]).forEach(id => verify(!button(box, "fill." + id).enabled, id + " waits"));
            compare(button(box, "fill.yes").tone, "danger", "Yes writes over the backup");
            mouseClick(said[0]);
            compare(box.fill.calls, [], "a click on the question answers nothing");
            mouseClick(button(box, "fill.no"));
            mouseClick(button(box, "fill.yes"));
            compare(box.fill.calls, ["answer 0", "answer 1"]);
            box.visible = false;
        }

        // With no place the page invites one, and Back up now has nothing to write.
        function test_a_page_with_no_place_invites_one() {
            const box = open("backup-none");
            verify(Kit.texts(box.contentItem, box.view.sheet.none)[0], "the invitation");
            compare(Kit.filled(box.contentItem).length, 0);
            verify(!button(box, "fill.back-up-now").enabled, "nothing to write yet");
            mouseClick(button(box, "fill.add-place"));
            compare(box.fill.calls, ["button add-place 0"]);
            box.visible = false;
        }

        function test_a_full_page_keeps_its_buttons_below_five_rows_and_pages() {
            const box = open("backup-paged");
            compare(Kit.filled(box.contentItem).length, 5);
            verify(Kit.texts(box.contentItem, "1 of 2")[0], "where the list is");
            mouseClick(button(box, "fill.page-on"));
            compare(box.fill.calls, ["page 1"]);
            const last = Kit.slots(box.contentItem).pop();
            const end = place(box, last).y + last.height;
            root.below.forEach(id => verify(place(box, button(box, "fill." + id)).y >= end, id + " sits below"));
            box.visible = false;
        }

        // The state reads from the right in the Arabic font, a Latin name stays as written, and every part keeps its place.
        function test_arabic_reads_right_to_left_with_the_layout_unmoved() {
            const latin = open("backup");
            const box = open("backup", "ar");
            const rows = Kit.filled(box.contentItem);
            const state = Kit.texts(rows[0], box.view.sheet.places[0].state)[0];
            compare(state.effectiveHorizontalAlignment, Text.AlignRight);
            compare(state.font.family, box.res.fam.arabic);
            compare(Kit.texts(rows[0], "OneDrive")[0].effectiveHorizontalAlignment, Text.AlignLeft, "a Latin name");
            const at = (b, id) => place(b, button(b, id));
            fuzzyCompare(at(box, "fill.back").x, at(latin, "fill.back").x, 0.01, "Back keeps its place");
            fuzzyCompare(at(box, "fill.back").y, at(latin, "fill.back").y, 0.01, "Back keeps its place");
            const end = (b, id) => at(b, id).x + button(b, id).width;
            fuzzyCompare(end(box, "fill.row-0-remove"), end(latin, "fill.row-0-remove"), 0.01, "Remove keeps its end");
            const order = ["back", "back-up-now", "add-place"].map(id => at(box, "fill." + id).x);
            verify(order[0] < order[1] && order[1] < order[2], "the buttons keep their order");
            latin.visible = box.visible = false;
        }

        // Every text and icon on a backup page wears an ink whose pair kx-theme checks at 4.5:1 (drawn/fill.rs).
        function test_every_backup_page_draws_only_inks_the_theme_checks() {
            for (const look of [NL, NC, DD]) {
                const p = look.lk.palette;
                const inks = [p.pop_ink, p.pop_muted, look.lk.sel_ink, p.enter_ink, p.danger_ink, p.ring];
                const known = c => inks.some(i => Qt.colorEqual(c, i));
                const box = open(root.pages[0], "en", look);
                for (const name of root.pages) {
                    box.view = Kit.view("en", name);
                    verify(waitForPolish(box), name + " laid out");
                    const where = look.lk.look.id + " " + name;
                    for (const t of Board.all(box.contentItem, o => o.visible && o.font !== undefined && o.text))
                        verify(known(t.color), where + ": " + t.text + " is checked");
                    for (const i of Board.all(box.contentItem, o => o.visible && o.tint !== undefined && o.icon))
                        verify(known(i.tint), where + ": " + i.icon + " is checked");
                }
                box.visible = false;
            }
        }
    }
}

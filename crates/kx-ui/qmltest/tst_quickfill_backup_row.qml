// Quick-fill's Remove place question over the hub's views: it asks once on its row, naming the place, while the other
// rows and the page's buttons wait; a long folder's name ends in an ellipsis within the token's lines; and each language
// starts from its own side.
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
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-dolch-light.js" as DL

Item {
    id: root
    width: 200
    height: 100

    readonly property var langs: ({ en: En, fr: Fr, ar: Ar })
    // The buttons below the rows.
    readonly property var below: ["back-up-now", "add-place", "bring-back", "change-password", "import"]

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }

    TestCase {
        name: "quickfill_backup_row"
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

        // The question stands in the asked row's place, with that row's kind; the others stay as they were, and wait.
        function test_remove_place_asks_once_on_that_row_only() {
            const box = open("remove");
            const asked = box.view.sheet.confirm;
            compare(asked.row, 1);
            const said = Kit.texts(box.contentItem, asked.text);
            compare(said.length, 1, "it asks once");
            verify(asked.text.indexOf(box.view.sheet.places[1].name.text) >= 0, "it names its place, as two sticks share a tile");
            verify(!said[0].truncated, "and shows it");
            const slots = Kit.slots(box.contentItem);
            const top = place(box, slots[1]).y;
            for (const part of [said[0], button(box, "fill.yes"), button(box, "fill.no")]) {
                const y = place(box, part).y;
                verify(y >= top - 0.5 && y + part.height <= top + slots[1].height + 0.5, "on the row");
            }
            compare(Board.all(box.contentItem, o => o.visible && o.tint !== undefined && o.icon === NL.lk.icons.fill.place_removable && o.mapToItem(box.body, 0, 0).y >= top && o.mapToItem(box.body, 0, 0).y < top + slots[1].height).length, 1, "its kind's icon stays");
            compare(one(box.contentItem, "fill.row-1-remove"), null, "its Remove gives way");
            compare(Kit.filled(box.contentItem).length, 2, "the other rows stay");
            for (const r of [0, 2]) {
                verify(!button(box, "fill.row-" + r + "-remove").enabled, "row " + r + " waits");
                verify(Kit.texts(slots[r], box.view.sheet.places[r].state)[0], "row " + r + " keeps its state");
            }
            root.below.concat(["back", "checks"]).forEach(id => verify(!button(box, "fill." + id).enabled, id + " waits"));
            verify(button(box, "fill.close").enabled, "Close still acts");
            mouseClick(said[0]);
            compare(box.fill.calls, [], "a click on the question answers nothing");
            compare(button(box, "fill.yes").tone, "danger");
            mouseClick(button(box, "fill.no"));
            mouseClick(button(box, "fill.yes"));
            compare(box.fill.calls, ["answer 0", "answer 1"]);
            compare(box.fill.rounds, [box.view.round, box.view.round]);
            box.visible = false;
        }

        // A new view can come with fewer places, and the question goes with the row it stood on.
        function test_a_new_view_takes_the_question_away_with_its_places() {
            const box = open("remove");
            const said = box.view.sheet.confirm.text;
            box.view = Kit.view("en", "backup-none");
            verify(waitForPolish(box));
            compare(Kit.texts(box.contentItem, said).length, 0, "the question is gone");
            compare(one(box.contentItem, "fill.yes"), null);
            box.view = Kit.view("en", "backup");
            verify(waitForPolish(box));
            compare(Kit.filled(box.contentItem).length, 3, "every row shows again");
            verify(button(box, "fill.row-1-remove").enabled, "and acts");
            verify(button(box, "fill.back-up-now").enabled, "as the page's buttons do");
            box.visible = false;
        }

        // The page has no heading line, so the head names it; readers hear which place a question is about.
        function test_the_head_names_the_page_and_readers_hear_the_place_asked_about() {
            const box = open("remove");
            verify(Kit.texts(box.contentItem, box.view.buttons.backup).length >= 2, "the head's title and its Backup");
            compare(Kit.texts(box.contentItem, box.view.title).length, 0, "instead of Quick-fill");
            const asked = box.view.sheet.confirm;
            const alert = Board.all(box.contentItem, o => o.visible && o.Accessible.name === asked.text && !o.Accessible.ignored);
            compare(alert.length, 1, "readers hear the question once");
            compare(alert[0].Accessible.role, Accessible.AlertMessage);
            verify(alert[0].Accessible.name.indexOf(box.view.sheet.places[asked.row].name.text) >= 0, "the question names it");
            compare(alert[0].Accessible.description, "", "so nothing says it twice");
            box.visible = false;
        }

        // Each language's question names its place and fits its row in every look, from the language's own side.
        function test_the_row_question_fits_its_row_in_every_look_and_language() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                for (const tag of ["en", "fr", "ar"]) {
                    const box = open("remove", tag, look);
                    const where = look.lk.look.id + " " + tag;
                    const said = Kit.texts(box.contentItem, box.view.sheet.confirm.text)[0];
                    const row = Kit.slots(box.contentItem)[1];
                    verify(said.text.indexOf(box.view.sheet.places[1].name.text) >= 0, where + ": it names its place");
                    verify(!said.truncated, where + ": the question is whole");
                    verify(said.contentHeight <= row.height + 0.5, where + ": " + said.contentHeight + " in " + row.height);
                    verify(said.width > 0, where + ": it has room");
                    compare(said.effectiveHorizontalAlignment, tag === "ar" ? Text.AlignRight : Text.AlignLeft, where + ": its side");
                    box.visible = false;
                }
            }
        }

        // A long folder's name ends in an ellipsis within the token's lines, while readers still hear the whole question.
        function test_a_long_place_name_ends_in_an_ellipsis_on_its_row() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                for (const tag of ["en", "fr", "ar"]) {
                    const box = open("remove-long", tag, look);
                    const where = look.lk.look.id + " " + tag;
                    const asked = box.view.sheet.confirm;
                    const said = Kit.texts(box.contentItem, asked.text)[0];
                    const row = Kit.slots(box.contentItem)[asked.row];
                    verify(said.truncated, where + ": it ends in an ellipsis");
                    compare(said.lineCount, look.lk.shape.panel.question_lines, where + ": its lines");
                    verify(said.contentHeight <= row.height + 0.5, where + ": " + said.contentHeight + " in " + row.height);
                    verify(place(box, said).y >= place(box, row).y - 0.5, where + ": on its row");
                    const alert = Board.all(box.contentItem, o => o.visible && o.Accessible.name === asked.text && !o.Accessible.ignored);
                    compare(alert.length, 1, where + ": readers hear it whole");
                    box.visible = false;
                }
            }
        }
    }
}

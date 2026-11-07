// Quick-fill's password page and its found and foreign boxes over the hub's views: the text shows while typed but reaches
// no reader, the right line for each ask, Done and Back; each box found with Bring back, and a foreign box's Yes and No,
// each with the view's round; and Arabic flips the text only.
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
    readonly property var ic: NL.lk.icons.fill
    // A password the fixture types.
    readonly property string typed: "Tr0ub4dor&3"
    // Each ask's view, and words its line must hold.
    readonly property var lines: [
        ["pass-new", ["least 8 characters"]],
        ["pass-check", ["3 tries left"]],
        ["pass-open", ["bring your entries back"]],
        ["pass-own", ["bring your entries back", "saved again"]],
        ["pass-foreign", ["other computer"]],
        ["pass-keepass", ["KeePass"]]
    ]

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }

    TestCase {
        name: "quickfill_backup_ask"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in language `tag` (English by default) and `look` (Native by default), laid out.
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

        function test_the_password_shows_while_typed_and_the_line_takes_the_letters() {
            const box = open("pass-new");
            const line = button(box, "fill.password");
            compare(Kit.texts(line, root.typed).length, 1, "the text shows (owner)");
            compare(Board.all(box.contentItem, o => o.visible && o.caret === true).length, 0, "no caret until aimed");
            box.aimed = true;
            compare(Board.all(box.contentItem, o => o.visible && o.caret === true).length, 1, "one caret");
            compare(Board.all(line, o => o.visible && o.caret === true).length, 1, "on the line");
            compare(line.Accessible.role, Accessible.EditableText);
            compare(line.Accessible.name, box.view.sheet.text, "readers hear what it asks");
            compare(box.view.sheet.typed.text, root.typed, "the fixture typed it");
            compare(Kit.texts(open("pass-open").contentItem, root.typed).length, 0, "nothing typed, nothing shown");
            box.visible = false;
        }

        // The page shows the letters, as the design asks, but no accessible property ever holds them.
        function test_the_typed_password_reaches_no_reader() {
            const box = open("pass-new");
            for (const o of Board.all(box.contentItem, () => true)) {
                verify(String(o.Accessible.name).indexOf(root.typed) < 0, "not in a name");
                verify(String(o.Accessible.description).indexOf(root.typed) < 0, "not in a description");
            }
            const line = button(box, "fill.password");
            compare(line.Accessible.description, box.view.words.mask, "readers hear the mask");
            const letters = Kit.texts(line, root.typed);
            compare(letters.length, 1, "the letters show");
            verify(letters[0].Accessible.ignored, "and readers skip the label that holds them");
            const empty = open("pass-open");
            compare(button(empty, "fill.password").Accessible.description, "", "an empty line has nothing to hear");
            box.visible = empty.visible = false;
        }

        function test_each_ask_says_its_own_line() {
            for (const [name, holds] of root.lines) {
                const box = open(name);
                const text = box.view.sheet.text;
                holds.forEach(h => verify(text.indexOf(h) >= 0, name + ": " + text));
                const said = Board.all(box.contentItem, o => o.visible && o.font !== undefined && o.text === text);
                compare(said.length, 1, name + " says it once");
                box.visible = false;
            }
        }

        // Every ask's whole line fits the page in each look and language, however long it runs.
        function test_each_line_fits_the_page_in_every_look_and_language() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                for (const tag of ["en", "fr", "ar"]) {
                    const box = open(root.lines[0][0], tag, look);
                    for (const [name] of root.lines) {
                        box.view = Kit.view(tag, name);
                        verify(waitForPolish(box), name + " laid out");
                        const said = Kit.texts(box.contentItem, box.view.sheet.text)[0];
                        const where = look.lk.look.id + " " + tag + " " + name;
                        verify(!said.truncated, where + " is whole");
                        verify(said.contentHeight <= box.slotsH / 2, where + " keeps to the rows' place");
                    }
                    box.visible = false;
                }
            }
        }

        function test_done_and_back_send_their_clicks_with_the_round() {
            const box = open("pass-check");
            compare(button(box, "fill.done").tone, "enter");
            mouseClick(button(box, "fill.done"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["button done 0", "button back 0"]);
            compare(box.fill.rounds, [box.view.round, box.view.round]);
            box.visible = false;
        }

        function test_a_wrong_try_says_so_in_the_note_line() {
            const box = open("pass-wrong");
            verify(Kit.texts(box.contentItem, box.view.note)[0], "the note shows");
            verify(box.view.sheet.text.indexOf("3 tries left") >= 0, "and the tries left");
            box.visible = false;
        }

        function test_keepass_wears_imports_icon_and_a_backup_password_the_lock() {
            const icons = (b, icon) => Board.all(b.contentItem, o => o.visible && o.tint !== undefined && o.icon === icon).length;
            const keepass = open("pass-keepass");
            compare(icons(keepass, root.ic.keepass), 1);
            const backup = open("pass-new");
            compare(icons(backup, root.ic.lock), 1);
            keepass.visible = backup.visible = false;
        }

        // The ask reads from the right in the Arabic font, typed Latin stays as written, and each part keeps its place.
        function test_arabic_reads_right_to_left_with_the_layout_unmoved() {
            const latin = open("pass-new");
            const box = open("pass-new", "ar");
            const said = Kit.texts(box.contentItem, box.view.sheet.text)[0];
            compare(said.effectiveHorizontalAlignment, Text.AlignHCenter, "centred, as a question is");
            compare(said.font.family, box.res.fam.arabic);
            compare(Kit.texts(box.contentItem, root.typed)[0].effectiveHorizontalAlignment, Text.AlignLeft, "typed Latin");
            const at = (b, id) => button(b, id).mapToItem(b.body, 0, 0);
            for (const id of ["fill.back", "fill.password"]) {
                fuzzyCompare(at(box, id).x, at(latin, id).x, 0.01, id + " keeps its place");
                fuzzyCompare(at(box, id).y, at(latin, id).y, 0.01, id + " keeps its place");
            }
            const end = (b, id) => at(b, id).x + button(b, id).width;
            fuzzyCompare(end(box, "fill.done"), end(latin, "fill.done"), 0.01, "Done keeps its side");
            latin.visible = box.visible = false;
        }

        function test_each_box_found_says_where_and_when_with_bring_back() {
            const box = open("found");
            const rows = Kit.filled(box.contentItem);
            compare(rows.length, 2);
            const found = box.view.sheet.rows;
            [root.ic.place_cloud, root.ic.place_removable].forEach((icon, i) => {
                compare(Board.all(rows[i], o => o.visible && o.tint !== undefined && o.icon === icon).length, 1, "its kind's icon");
                verify(Kit.texts(rows[i], found[i].name.text)[0], "its place");
                const text = Kit.texts(rows[i], found[i].text)[0];
                verify(text && !text.truncated, "its whole sentence");
                compare(rows[i].Accessible.description, found[i].text, "readers hear it");
                compare(button(box, "fill.row-" + i + "-open").label, box.view.buttons["bring-back"]);
            });
            verify(Kit.texts(box.contentItem, box.view.buttons["bring-back"]).length >= 2, "the page's heading and its rows");
            const round = 77;
            const v = Kit.view("en", "found");
            v.round = round;
            box.view = v;
            verify(waitForPolish(box));
            mouseClick(button(box, "fill.row-1-open"));
            mouseClick(button(box, "fill.other-folder"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["row open 1", "button other-folder 0", "button back 0"]);
            compare(box.fill.rounds, [round, round, round]);
            box.visible = false;
        }

        // A foreign box is asked about once; Yes brings entries in, so it is no danger, and only Yes or No answers.
        function test_a_foreign_box_is_asked_about_and_answers_with_the_round() {
            const box = open("foreign");
            const sheet = box.view.sheet;
            const said = Kit.texts(box.contentItem, sheet.text);
            compare(said.length, 1, "it asks once");
            verify(Kit.texts(box.contentItem, sheet.name.text)[0], "its place heads the page");
            compare(Board.all(box.contentItem, o => o.visible && o.tint !== undefined && o.icon === root.ic.place_cloud).length, 1, "its kind's icon");
            compare(button(box, "fill.yes").tone, "enter");
            const round = 88;
            const v = Kit.view("en", "foreign");
            v.round = round;
            box.view = v;
            verify(waitForPolish(box));
            mouseClick(Kit.texts(box.contentItem, sheet.text)[0]);
            compare(box.fill.calls, [], "a click on the question answers nothing");
            mouseClick(button(box, "fill.no"));
            mouseClick(button(box, "fill.yes"));
            compare(box.fill.calls, ["answer 0", "answer 1"]);
            compare(box.fill.rounds, [round, round]);
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls.pop(), "button back 0");
            box.visible = false;
        }

        // A box whose date the system does not tell asks the same question without one.
        function test_a_foreign_box_with_no_date_asks_the_same_way() {
            const box = open("foreign-undated");
            const said = Kit.texts(box.contentItem, box.view.sheet.text);
            compare(said.length, 1, "it asks once");
            compare(Board.all(box.contentItem, o => o.visible && o.tint !== undefined && o.icon === root.ic.place_removable).length, 1, "its kind's icon");
            mouseClick(button(box, "fill.yes"));
            compare(box.fill.calls, ["answer 1"]);
            box.visible = false;
        }

        // In Arabic each question reads in the Arabic font, and each part keeps its place.
        function test_the_boxes_read_right_to_left_with_the_layout_unmoved() {
            const latin = open("found");
            const box = open("found", "ar");
            const rows = Kit.filled(box.contentItem);
            const text = Kit.texts(rows[0], box.view.sheet.rows[0].text)[0];
            compare(text.effectiveHorizontalAlignment, Text.AlignRight);
            compare(text.font.family, box.res.fam.arabic);
            const at = (b, id) => button(b, id).mapToItem(b.body, 0, 0);
            fuzzyCompare(at(box, "fill.back").x, at(latin, "fill.back").x, 0.01, "Back keeps its place");
            const end = (b, id) => at(b, id).x + button(b, id).width;
            fuzzyCompare(end(box, "fill.row-0-open"), end(latin, "fill.row-0-open"), 0.01, "Bring back keeps its side");
            fuzzyCompare(end(box, "fill.other-folder"), end(latin, "fill.other-folder"), 0.01, "Another folder keeps its side");
            latin.visible = box.visible = false;
            const foreign = open("foreign", "ar");
            const asked = Kit.texts(foreign.contentItem, foreign.view.sheet.text)[0];
            compare(asked.font.family, foreign.res.fam.arabic);
            foreign.visible = false;
        }
    }
}

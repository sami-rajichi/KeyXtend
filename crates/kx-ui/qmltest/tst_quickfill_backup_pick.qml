// Quick-fill's lists a place or a file is picked from, over the hub's views: the suggested places by kind, Another folder
// and Bring back; the paged drives, folders and files with Use this folder when picking a folder; the KeePass pick and
// its result note with counts; and Arabic flips the text only.
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

Item {
    id: root
    width: 200
    height: 100

    readonly property var langs: ({ en: En, fr: Fr, ar: Ar })
    readonly property var ic: NL.lk.icons.fill

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }
    Component { id: namesC; QuickFillNames {} }

    TestCase {
        name: "quickfill_backup_pick"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in language `tag` (English by default), laid out.
        function open(name, tag) {
            const ui = langs[tag || "en"];
            const props = { ui: createTemporaryObject(uiC, root), fill: createTemporaryObject(fillC, root), lk: NL.lk, res: Board.resOf(NL.lk, ui), s: 1, view: Kit.view(tag || "en", name), rows: En.settings.pageRows, rtl: ui.lang.rtl };
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
        // How many shown icons under `item` draw `icon`.
        function icons(item, icon) {
            return Board.all(item, o => o.visible && o.tint !== undefined && o.icon === icon).length;
        }

        function test_the_places_show_by_kind_and_each_is_chosen() {
            const box = open("places");
            verify(Kit.texts(box.contentItem, box.view.sheet.text)[0], "what the page asks");
            const rows = Kit.filled(box.contentItem);
            compare(rows.length, 3);
            [root.ic.place_cloud, root.ic.place_removable, root.ic.place_drive].forEach((icon, i) => {
                compare(icons(rows[i], icon), 1, "its kind's icon");
                verify(Kit.texts(rows[i], box.view.sheet.rows[i].name.text)[0], "its name");
                compare(button(box, "fill.row-" + i + "-open").label, "Choose");
            });
            compare(icons(rows[0], root.ic.pick), 1, "Choose wears its icon");
            mouseClick(button(box, "fill.row-1-open"));
            mouseClick(button(box, "fill.other-folder"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["row open 1", "button other-folder 0", "button back 0"]);
            compare(new Set(box.fill.rounds).size, 1);
            compare(box.fill.rounds[0], box.view.round);
            box.visible = false;
        }

        // Every Places page offers Bring back, after Another folder.
        function test_the_places_offer_bring_back_after_another_folder() {
            const box = open("places");
            const [other, bring] = [button(box, "fill.other-folder"), button(box, "fill.bring-back")];
            verify(other.mapToItem(box.body, 0, 0).x + other.width <= bring.mapToItem(box.body, 0, 0).x, "after Another folder");
            compare(icons(bring, root.ic.bring_back), 1);
            mouseClick(bring);
            compare(box.fill.calls, ["button bring-back 0"]);
            compare(box.fill.rounds, [box.view.round]);
            box.visible = false;
        }

        // Every Places page offers Import, after Bring back.
        function test_the_places_offer_import_after_bring_back() {
            const box = open("places");
            const [bring, keepass] = [button(box, "fill.bring-back"), button(box, "fill.import")];
            verify(bring.mapToItem(box.body, 0, 0).x + bring.width <= keepass.mapToItem(box.body, 0, 0).x, "after Bring back");
            compare(icons(keepass, root.ic.keepass), 1);
            mouseClick(keepass);
            compare(box.fill.calls, ["button import 0"]);
            compare(box.fill.rounds, [box.view.round]);
            box.visible = false;
        }

        function test_a_refused_pick_says_so_in_the_note_line() {
            const box = open("places-refused");
            verify(box.view.note.indexOf("Choose another folder") >= 0, box.view.note);
            verify(Kit.texts(box.contentItem, box.view.note)[0], "the note shows");
            compare(Kit.filled(box.contentItem).length, 3, "the places stay");
            box.visible = false;
        }

        function test_the_folder_list_is_paged_and_use_this_folder_sends_its_click() {
            const box = open("folders");
            compare(box.view.sheet.pick, createTemporaryObject(namesC, root).picks.folder);
            verify(Kit.texts(box.contentItem, box.view.sheet.text)[0], "what the list picks");
            verify(Kit.texts(box.contentItem, "Documents")[0], "the folder it shows");
            compare(Kit.filled(box.contentItem).length, 5, "a full page");
            verify(Kit.texts(box.contentItem, "1 of 2")[0], "where the list is");
            for (let i = 0; i < 5; i++)
                compare(icons(Kit.filled(box.contentItem)[i], root.ic.place_folder), 1, "a folder's icon");
            compare(button(box, "fill.row-0-open").label, box.view.sheet.rows[0].buttons[0].label);
            mouseClick(button(box, "fill.page-on"));
            mouseClick(button(box, "fill.row-2-open"));
            mouseClick(button(box, "fill.here"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["page 1", "row open 2", "button here 0", "button back 0"]);
            compare(new Set(box.fill.rounds).size, 1);
            box.visible = false;
        }

        // At the drives no folder is open, so Use this folder waits, and This PC names the list.
        function test_the_drives_come_first_and_use_this_folder_waits() {
            const box = open("drives");
            verify(box.view.sheet.drives, "at the drives");
            verify(Kit.texts(box.contentItem, box.view.sheet.folder.text)[0], "This PC");
            verify(!button(box, "fill.here").enabled, "no folder to use yet");
            const rows = Kit.filled(box.contentItem);
            [root.ic.place_drive, root.ic.place_drive, root.ic.place_removable].forEach((icon, i) => compare(icons(rows[i], icon), 1));
            box.visible = false;
        }

        // A list that picks a file has no Use this folder; a file is chosen, and a folder still opens.
        function test_a_backup_pick_chooses_a_file_and_opens_a_folder() {
            const box = open("files");
            compare(one(box.contentItem, "fill.here"), null, "no folder to pick");
            const rows = Kit.filled(box.contentItem);
            compare(icons(rows[0], root.ic.place_folder), 1);
            compare(icons(rows[0], root.ic.open), 1, "a folder opens");
            compare(icons(rows[1], root.ic.place_file), 1);
            compare(icons(rows[1], root.ic.pick), 1, "a file is chosen");
            compare(button(box, "fill.row-1-open").label, "Choose");
            mouseClick(button(box, "fill.row-1-open"));
            compare(box.fill.calls, ["row open 1"]);
            box.visible = false;
        }

        function test_the_keepass_pick_heads_with_import_and_has_no_use_this_folder() {
            const box = open("keepass");
            const names = createTemporaryObject(namesC, root);
            compare(box.view.sheet.pick, names.picks.keepass);
            verify(Kit.texts(box.contentItem, "Choose a KeePass file.")[0], "what it picks");
            compare(one(box.contentItem, "fill.here"), null, "no folder to pick");
            verify(icons(box.contentItem, root.ic.keepass) >= 1, "Import's icon heads it");
            mouseClick(button(box, "fill.row-1-open"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["row open 1", "button back 0"]);
            box.visible = false;
        }

        // The result counts what came in and the entries or values left out, in each language, as an alert readers hear.
        function test_the_import_result_shows_its_counts_in_the_note_line() {
            for (const tag of ["en", "fr", "ar"]) {
                const box = open("imported", tag);
                const note = box.view.note;
                verify(note.indexOf("12") >= 0 && note.indexOf("2", note.indexOf("12") + 2) >= 0, tag + ": " + note);
                verify(Kit.texts(box.contentItem, note)[0], tag + ": the note shows");
                const alert = Board.all(box.contentItem, o => o.visible && o.Accessible.name === note && !o.Accessible.ignored);
                compare(alert.length, 1, "readers hear it once");
                compare(alert[0].Accessible.role, Accessible.AlertMessage);
                box.visible = false;
            }
            compare(Kit.view("en", "imported").note, "12 entries came in. Entries or values left out: 2.");
        }

        // Each page's text reads from the right, an Arabic folder name in the Arabic font, and each part keeps its place.
        function test_arabic_reads_right_to_left_with_the_layout_unmoved() {
            const latin = open("folders");
            const box = open("folders", "ar");
            const head = Kit.texts(box.contentItem, box.view.sheet.text)[0];
            compare(head.effectiveHorizontalAlignment, Text.AlignRight);
            compare(head.font.family, box.res.fam.arabic);
            const own = Kit.texts(box.contentItem, "مستنداتي")[0];
            compare(own.effectiveHorizontalAlignment, Text.AlignRight, "an Arabic folder's name");
            compare(own.font.family, box.res.fam.arabic);
            const at = (b, id) => button(b, id).mapToItem(b.body, 0, 0);
            for (const id of ["fill.back", "fill.page-back"])
                fuzzyCompare(at(box, id).y, at(latin, id).y, 0.01, id + " keeps its line");
            fuzzyCompare(at(box, "fill.back").x, at(latin, "fill.back").x, 0.01, "Back keeps its side");
            const end = (b, id) => at(b, id).x + button(b, id).width;
            fuzzyCompare(end(box, "fill.here"), end(latin, "fill.here"), 0.01, "Use this folder keeps its side");
            fuzzyCompare(end(box, "fill.row-0-open"), end(latin, "fill.row-0-open"), 0.01, "Open keeps its side");
            latin.visible = box.visible = false;
        }
    }
}

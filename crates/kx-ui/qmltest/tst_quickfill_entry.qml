// Quick-fill's entry page and By label's values over the hub's views: each field with its label and value, a hidden one
// masked until Show reveals it, Insert, Edit and Back, and an Arabic entry that reads right to left in place.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }

    TestCase {
        name: "quickfill_entry"
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
        // Every shown text under `item`.
        function said(item) {
            return Board.all(item, o => o.visible && o.font !== undefined && o.text !== undefined).map(o => o.text);
        }

        function test_the_entry_page_shows_each_field_and_its_buttons() {
            const box = open("entry");
            verify(Kit.texts(box.contentItem, "GitHub")[0], "the entry's name");
            const fields = Kit.filled(box.contentItem);
            compare(fields.length, 3);
            verify(Kit.texts(fields[0], "User")[0] && Kit.texts(fields[0], "alex")[0], "a label over its value");
            mouseClick(button(box, "fill.field-0-insert"));
            mouseClick(button(box, "fill.field-2-insert"));
            mouseClick(button(box, "fill.edit"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["field insert 0", "field insert 2", "button edit 0", "button back 0"]);
            box.visible = false;
        }

        // An empty hidden value shows nothing and readers hear no mask, on its page and under its label; Show stays.
        function test_an_empty_hidden_value_shows_no_mask_on_a_read_page() {
            const box = open("blank");
            const mask = box.view.words.mask;
            const field = Kit.filled(box.contentItem)[1];
            compare(Kit.texts(field, mask).length, 0, "no mask");
            compare(field.Accessible.description, "", "and readers hear none");
            verify(button(box, "fill.field-1-show"), "Show stays");
            const pins = open("pins");
            const v = Kit.view("en", "pins");
            v.sheet.rows[0].value = box.view.sheet.fields[1].value;
            pins.view = v;
            const rows = Kit.filled(pins.contentItem);
            compare(Kit.texts(rows[0], mask).length, 0, "no mask under its label");
            compare(Kit.texts(rows[1], mask).length, 1, "the other keeps its own");
            compare(rows[0].Accessible.name, "", "and readers hear none");
            box.visible = pins.visible = false;
        }

        // A hidden value shows the mask until Show reveals it, and Hide masks it again; the mask tells no length.
        function test_a_hidden_value_is_masked_until_show_reveals_it() {
            const box = open("entry");
            const field = Kit.filled(box.contentItem)[1];
            verify(Kit.texts(field, box.view.words.mask)[0], "the mask");
            compare(box.view.sheet.fields[1].count, null, "no count on a read page");
            verify(said(box.contentItem).every(t => t.indexOf("s3cret") < 0), "no secret is drawn");
            const show = button(box, "fill.field-1-show");
            compare([show.label, show.icon], ["Show", NL.lk.icons.fill.show]);
            mouseClick(show);
            compare(box.fill.calls, ["field show 1"]);
            box.view = Kit.view("en", "shown");
            verify(waitForPolish(box), "laid out again");
            verify(Kit.texts(field, "s3cret-Pass!")[0], "shown now");
            compare(Kit.texts(field, box.view.words.mask).length, 0);
            const hide = button(box, "fill.field-1-show");
            compare([hide.label, hide.icon], ["Hide", NL.lk.icons.fill.hide]);
            box.visible = false;
        }

        // Readers hear each field's value after its label, or the mask while it is hidden; a hidden value is in no
        // accessible property.
        function test_a_reader_hears_the_shown_value_or_the_mask() {
            const box = open("entry");
            const fields = Kit.filled(box.contentItem);
            compare(fields.map(f => f.Accessible.description), ["alex", box.view.words.mask, "github.com"]);
            const heard = Board.all(box.contentItem, o => true).map(o => o.Accessible.name + " " + o.Accessible.description);
            verify(heard.every(t => t.indexOf("s3cret") < 0), "no secret is heard");
            box.view = Kit.view("en", "shown");
            verify(waitForPolish(box), "laid out again");
            compare(fields[1].Accessible.description, "s3cret-Pass!", "a shown value is heard");
            box.visible = false;
        }

        function test_a_labels_values_insert_and_hidden_ones_stay_masked() {
            const box = open("values");
            verify(Kit.texts(box.contentItem, "Email")[0], "the label heads the page");
            const rows = Kit.filled(box.contentItem);
            verify(Kit.texts(rows[0], "alex@example.com")[0] && Kit.texts(rows[0], "Alex Martin")[0], "a value and its entry");
            mouseClick(button(box, "fill.row-1-insert"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["row insert 1", "button back 0"]);
            const pins = open("pins");
            const masked = Kit.filled(pins.contentItem);
            verify(masked.every(r => Kit.texts(r, pins.view.words.mask).length === 1), "each PIN is masked");
            verify(said(pins.contentItem).every(t => t !== "4821" && t !== "1234"), "no PIN is drawn");
            box.visible = pins.visible = false;
        }

        // The Arabic entry's texts read from the right in the Arabic font, while every part keeps the Latin page's place.
        function test_an_arabic_entry_reads_right_to_left_while_the_layout_stays() {
            const latin = open("entry");
            const box = open("arabic");
            const name = Kit.texts(box.contentItem, "ليلى حداد");
            verify(name.length >= 2, "the name heads the page and fills a field");
            for (const t of name) {
                compare(t.effectiveHorizontalAlignment, Text.AlignRight);
                compare(t.font.family, box.res.fam.arabic);
            }
            const own = Kit.texts(box.contentItem, "رقم الخزانة")[0];
            compare(own.effectiveHorizontalAlignment, Text.AlignRight, "an own Arabic label too");
            // Measured in the box, which pops in as each panel opens, so its scale leaves a rounding trace.
            for (const id of ["fill.field-0-insert", "fill.back"]) {
                const [p, q] = [box, latin].map(b => button(b, id).mapToItem(b.body, 0, 0));
                fuzzyCompare(p.x, q.x, 0.01, id + " keeps its place");
                fuzzyCompare(p.y, q.y, 0.01, id + " keeps its place");
            }
            latin.visible = box.visible = false;
        }
    }
}

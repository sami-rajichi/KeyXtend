// Quick-fill's New and Edit over the hub's views: the kind chips first, then the name and the fields with the line that
// takes the letters, each line's one border, hint and dots, each field's Show or Hide, hidden switch, Paste and Make
// one, Add a field with the labels table or Your own label, Save and Cancel, and Delete entry's one question, which only
// Yes or No answers.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "quickfill_edit"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in English and look `lk` (Native light by default), open and laid out; the real
        // pointer passes through it.
        function open(name, lk) {
            const look = lk || NL.lk;
            const props = { ui: createTemporaryObject(uiC, root), fill: createTemporaryObject(fillC, root), lk: look, res: Board.resOf(look, En), s: 1, view: Kit.view("en", name), rows: En.settings.pageRows };
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
        // The shown carets under `item`: the line that takes the letters shows one.
        function carets(item) {
            return Board.all(item, o => o.visible && o.caret === true);
        }
        // The highlight ring of the page `box` shows.
        function ring(box) {
            return Board.all(box.contentItem, o => o.ring !== undefined && o.visible)[0].ring;
        }
        // The shown texts under `item` drawn in the character of `mask`.
        function dots(item, mask) {
            return Board.all(item, o => o.visible && o.font !== undefined && String(o.text).indexOf(mask.charAt(0)) >= 0);
        }

        // Each line on New and Edit marks itself with its own border, so no ring draws a second one around its row.
        function test_new_and_edit_draw_no_ring_and_readers_still_hear_the_selected_line() {
            for (const name of ["form", "edit"]) {
                const box = open(name);
                verify(!ring(box).visible, name + ": only the line's own border");
                const hi = Kit.filled(box.contentItem)[box.view.highlight];
                verify(hi.Accessible.selected, name + ": selected for readers");
                box.visible = false;
            }
        }

        function test_the_other_pages_keep_their_ring() {
            for (const name of ["list", "labels", "values", "entry", "add"]) {
                const box = open(name);
                verify(ring(box).visible, name);
                box.visible = false;
            }
        }

        // An empty hidden value says what the line is for, in the hint's colour, and shows and says no mask.
        function test_an_empty_hidden_line_shows_its_hint_and_no_mask() {
            const box = open("form");
            const line = button(box, "fill.field-1-value");
            const hint = Kit.texts(line, box.view.words.value)[0];
            verify(hint, "the hint");
            verify(Qt.colorEqual(hint.color, NL.lk.palette.pop_muted), "muted, so it never reads as a value");
            compare(dots(line, box.view.words.mask).length, 0, "no mask");
            compare(line.Accessible.description, "", "and readers hear none");
            compare(Kit.filled(box.contentItem)[1].Accessible.description, "", "on the row either");
            verify(button(box, "fill.field-1-show"), "Show stays");
            box.visible = false;
        }

        // A hidden value with text draws one dot per letter in the mask's spacing; readers hear only the fixed mask.
        function test_a_masked_line_shows_one_dot_per_letter_and_readers_hear_the_fixed_mask() {
            const box = open("edit");
            const mask = box.view.words.mask;
            const line = button(box, "fill.field-2-value");
            const count = box.view.sheet.fields[2].count;
            compare(count, "XK2-99".length, "the hub's count");
            const drawn = dots(line, mask);
            compare(drawn.map(o => o.text), [mask.charAt(0).repeat(count)]);
            // A font keeps its spacing in 64ths of a pixel.
            fuzzyCompare(drawn[0].font.letterSpacing, NL.lk.shape.fill.mask_em * line.f[3] * line.s, 1 / 64, "the mask's spacing");
            verify(Qt.colorEqual(drawn[0].color, NL.lk.palette.pop_muted), "muted");
            compare(line.Accessible.description, mask, "the fixed mask");
            compare(Kit.filled(box.contentItem)[2].Accessible.description, mask, "on the row too");
            box.visible = false;
        }

        // A line of dots too long for its room gives up its start, so the newest letter stays in sight.
        function test_a_long_masked_line_gives_up_its_start() {
            const box = open("edit");
            const line = button(box, "fill.field-2-value");
            const v = Kit.view("en", "edit");
            v.sheet.fields[2].count = 400;
            box.view = v;
            verify(waitForPolish(box), "laid out again");
            const drawn = dots(line, box.view.words.mask)[0];
            compare(drawn.text.length, 400);
            compare(drawn.elide, Text.ElideLeft);
            verify(drawn.truncated, "cut at its start");
            verify(drawn.width <= line.width, "inside the line");
            box.visible = false;
        }

        // Once a line takes the letters its hint hides, so only the caret waits there.
        function test_the_hint_hides_once_a_line_takes_the_letters() {
            const box = open("form");
            const line = button(box, "fill.field-1-value");
            const v = Kit.view("en", "form");
            v.sheet.typing = false;
            v.sheet.fields[1].typing = box.names.lines.value;
            box.view = v;
            box.aimed = true;
            compare(carets(line).length, 1, "the line takes the letters");
            compare(Kit.texts(line, box.view.words.value).length, 0, "no hint beside the caret");
            compare(dots(line, box.view.words.mask).length, 0, "and no mask");
            box.visible = false;
        }

        function test_new_shows_the_kind_chips_first() {
            const box = open("new");
            verify(Kit.texts(box.contentItem, box.view.sheet.ask)[0], "it asks what to save");
            const chips = box.view.sheet.kinds.map(k => button(box, "fill.kind-" + k.id));
            compare(chips.length, 6);
            compare(chips[2].label, "Wi-Fi");
            compare(Board.all(chips[2], o => o.visible && o.icon === NL.lk.icons.fill.kind_wifi).length, 1, "with its icon");
            mouseClick(chips[2]);
            mouseClick(button(box, "fill.cancel"));
            compare(box.fill.calls, ["kind wifi 0", "button cancel 0"]);
            compare(Kit.filled(box.contentItem).length, 0, "no field yet");
            const rows = new Set(chips.map(c => Math.round(c.mapToItem(box.contentItem, 0, 0).y)));
            compare(rows.size, NL.lk.shape.fill.kind_rows, "in the theme's rows");
            box.visible = false;
        }

        // The rows the chips take come from the theme, so three rows of two follow a token edit.
        function test_the_kind_chips_take_the_themes_rows() {
            const lk = JSON.parse(JSON.stringify(NL.lk));
            lk.shape.fill.kind_rows = 3;
            const box = open("new", lk);
            const chips = box.view.sheet.kinds.map(k => button(box, "fill.kind-" + k.id));
            const rows = new Set(chips.map(c => Math.round(c.mapToItem(box.contentItem, 0, 0).y)));
            compare(rows.size, 3);
            box.visible = false;
        }

        // A hovered line takes the hover tint, Highlight in high contrast, so its texts and icon take sel_ink there.
        function test_a_hovered_line_keeps_its_contrast_in_high_contrast() {
            for (const name of ["form", "edit"]) {
                const box = open(name, NC.lk);
                const line = button(box, "fill.name");
                mouseMove(line, line.width / 2, line.height / 2);
                const inks = Board.all(line, o => o.visible && o.font !== undefined && o.text).map(o => o.color);
                const tints = Board.all(line, o => o.visible && o.tint !== undefined && o.icon !== undefined).map(o => o.tint);
                verify(inks.length > 0, name + ": the line shows text");
                verify(inks.concat(tints).every(c => Qt.colorEqual(c, NC.lk.sel_ink)), name + ": in Highlight text");
                mouseMove(box.contentItem, 1, 1);
                box.visible = false;
            }
        }

        function test_the_line_that_takes_the_letters_shows_the_caret() {
            const form = open("form");
            form.aimed = true;
            const name = button(form, "fill.name");
            compare(carets(form.contentItem).length, 1, "one caret");
            compare(carets(name).length, 1, "on the name");
            const box = open("edit");
            box.aimed = true;
            compare(carets(box.contentItem).length, 1, "one caret");
            compare(carets(button(box, "fill.field-1-value")).length, 1, "on the password");
            mouseClick(button(box, "fill.field-0-value"));
            mouseClick(button(box, "fill.name"));
            compare(box.fill.calls.filter(c => !c.startsWith("caret")), ["field value 0", "button name 0"]);
            form.visible = box.visible = false;
        }

        // A form's line draws as taking the letters only while aimed (ADR-0040); its page still knows which line it is.
        function test_a_forms_line_draws_only_when_aimed() {
            const box = open("edit");
            const line = button(box, "fill.field-1-value");
            compare(carets(box.contentItem).length, 0, "no caret");
            verify(Qt.colorEqual(line.border.color, NL.lk.palette.pop_line), "no ring");
            box.aimed = true;
            compare(carets(line).length, 1, "the caret");
            verify(Qt.colorEqual(line.border.color, NL.lk.palette.ring), "the ring");
            box.aimed = false;
            compare(carets(box.contentItem).length, 0, "gone again");
            box.visible = false;
        }

        // A press on a form's line aims at once and its let-go picks that line; a question stops the pick, not the aim.
        function test_a_click_on_a_forms_line_asks_the_aim_and_a_question_waits() {
            const box = open("edit");
            Board.apart(box.contentItem);
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "aiming" });
            const value = button(box, "fill.field-0-value");
            mousePress(value);
            compare(spy.count, 1, "at the press");
            compare(box.fill.calls, [], "the pick waits for the let-go");
            mouseRelease(value);
            compare(spy.count, 1, "asked once");
            mouseClick(button(box, "fill.name"));
            compare(spy.count, 2);
            compare(box.fill.calls.filter(c => !c.startsWith("caret")), ["field value 0", "button name 0"]);
            const asked = open("confirm");
            const other = createTemporaryObject(spyC, root, { target: asked, signalName: "aiming" });
            mouseClick(button(asked, "fill.name"));
            compare(other.count, 1, "a press anywhere counts for the keys");
            compare(asked.fill.calls, [], "the question answers first");
            box.visible = asked.visible = false;
        }

        // On an Arabic keyboard a hidden line's caret waits on the right and stays there after the newest dot, which grows
        // to its left; Left goes back there too, so the dots count from the left.
        function test_a_masked_line_starts_on_the_keyboards_side_so_the_caret_stays_put() {
            const box = open("form");
            box.rtl = box.aimed = true;
            const v = Kit.view("en", "form");
            v.sheet.typing = false;
            v.sheet.fields[1].typing = box.names.lines.value;
            box.view = v;
            const line = button(box, "fill.field-1-value");
            const caret = carets(line)[0];
            const place = () => caret.mapToItem(line, 0, 0).x;
            const empty = place();
            verify(empty > line.width / 2, "on the right while empty");
            const typed = Kit.view("en", "form");
            typed.sheet.typing = false;
            typed.sheet.fields[1].typing = box.names.lines.value;
            typed.sheet.fields[1].value = null;
            typed.sheet.fields[1].count = 1;
            typed.sheet.caret = 1;
            box.view = typed;
            verify(waitForPolish(box), "laid out again");
            fuzzyCompare(place(), empty, 0.51, "after the first dot, it stays put");
            const back = JSON.parse(JSON.stringify(typed));
            back.sheet.caret = 0;
            box.view = back;
            verify(place() > line.width / 2, "still on the right with one dot");
            verify(place() < empty, "just before that dot, once Left took it back");
            const drawn = dots(line, box.view.words.mask);
            compare(drawn.length, 1);
            compare(drawn[0].horizontalAlignment, Text.AlignRight, "dots start on the right");
            box.visible = false;
        }

        // An empty name says its word once, as its caption, and readers still hear it.
        function test_an_empty_name_shows_its_caption_and_no_hint_beside_it() {
            const box = open("form");
            const name = button(box, "fill.name");
            const word = box.view.buttons["name"];
            compare(Kit.texts(name, word).length, 1);
            compare(name.name, word);
            box.visible = false;
        }

        function test_each_field_offers_show_the_hidden_switch_paste_and_make_one() {
            const box = open("edit");
            const ids = f => box.view.sheet.fields[f].buttons.map(b => b.id);
            compare(ids(1), ["show", "hidden", "paste", "make"]);
            compare(ids(0), ["hidden", "paste"]);
            const hidden = [0, 1].map(f => button(box, "fill.field-" + f + "-hidden"));
            compare(hidden.map(h => h.Accessible.checked), [false, true], "the switch reads as on or off");
            compare(hidden.map(h => h.icon), [NL.lk.icons.fill.hidden_off, NL.lk.icons.fill.hidden_on], "by more than colour");
            for (const id of ["show", "hidden", "paste", "make"])
                mouseClick(button(box, "fill.field-1-" + id));
            compare(box.fill.calls, ["field show 1", "field hidden 1", "field paste 1", "field make 1"]);
            box.visible = false;
        }

        // A hidden value not shown is masked on Edit too, and an own label is its own line.
        function test_an_own_label_is_its_own_line_and_a_hidden_value_stays_masked() {
            const box = open("edit");
            const own = Kit.filled(box.contentItem)[2];
            compare(dots(own, box.view.words.mask).length, 1, "masked");
            const texts = Board.all(box.contentItem, o => o.visible && o.font !== undefined).map(o => o.text);
            verify(texts.every(t => t !== "XK2-99"), "never drawn");
            mouseClick(button(box, "fill.field-2-label"));
            mouseClick(button(box, "fill.field-2-value"));
            compare(box.fill.calls.filter(c => !c.startsWith("caret")), ["field label 2", "field value 2"]);
            box.visible = false;
        }

        function test_save_cancel_add_a_field_and_delete_entry_send_their_buttons() {
            const box = open("edit");
            for (const id of ["add-field", "delete", "cancel", "save"])
                mouseClick(button(box, "fill." + id));
            compare(box.fill.calls, ["button add-field 0", "button delete 0", "button cancel 0", "button save 0"]);
            compare(one(open("form").contentItem, "fill.delete"), null, "New has no entry to delete");
            box.visible = false;
        }

        // Only a click on Yes deletes, as a destructive answer is never the default; the question covers the fields.
        function test_delete_entry_asks_once_and_only_yes_or_no_answers() {
            const box = open("confirm");
            const question = Kit.texts(box.contentItem, box.view.sheet.confirm)[0];
            verify(question, "the question");
            verify(!button(box, "fill.save").enabled, "the page waits for the answer");
            compare(Kit.filled(box.contentItem).length, 0, "the fields are covered");
            mouseClick(question);
            compare(box.fill.calls, [], "a click on the question answers nothing");
            mouseClick(button(box, "fill.no"));
            mouseClick(button(box, "fill.yes"));
            compare(box.fill.calls, ["answer 0", "answer 1"]);
            box.visible = false;
        }

        function test_add_a_field_lists_the_labels_and_your_own_label() {
            const box = open("add");
            const rows = Kit.filled(box.contentItem);
            compare(rows.length, 5);
            compare(button(box, "fill.row-1-open").label, "Add");
            mouseClick(button(box, "fill.row-1-open"));
            mouseClick(button(box, "fill.own-label"));
            mouseClick(button(box, "fill.back"));
            compare(box.fill.calls, ["row open 1", "button own-label 0", "button back 0"]);
            box.visible = false;
        }
    }
}

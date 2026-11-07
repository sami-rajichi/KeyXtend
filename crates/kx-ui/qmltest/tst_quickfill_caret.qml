// The caret on Quick-fill's New, Edit and password lines over the hub's views (ADR-0040): the view's one caret stands on
// the line that takes the letters, a click sends the line's own click and then its place, a hidden value's click carries
// only a number of dots, a question stops it, and no line ever takes focus.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
import "linekit.js" as Line
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
        name: "quickfill_caret"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing `view`, open, laid out and aimed at; the real pointer passes through it.
        function open(view) {
            const props = { ui: createTemporaryObject(uiC, root), fill: createTemporaryObject(fillC, root), lk: NL.lk, res: Board.resOf(NL.lk, En), s: 1, view: view, rows: En.settings.pageRows, aimed: true };
            const box = createTemporaryObject(boxC, root, props);
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        // The Edit view with the line `line` of field `at` taking the letters, or the name with no field, its caret at `caret`.
        function editing(at, line, caret) {
            const v = Kit.view("en", "edit");
            v.sheet.typing = at < 0;
            v.sheet.fields.forEach((f, i) => f.typing = i === at ? line : null);
            v.sheet.caret = caret;
            return v;
        }
        function line(box, id) {
            const l = one(box.contentItem, id);
            verify(l, id + " shows once");
            return l;
        }
        function carets(item) {
            return Board.all(item, o => o.visible && o.caret === true);
        }
        // Clicks `l` at place `at` of what it draws.
        function clickAt(l, at) {
            const ed = Line.editor(l);
            const p = ed.mapToItem(l, ed.positionToRectangle(at).x, 0);
            mouseClick(l, p.x + 1, l.height / 2);
        }
        // Clicks `l` at its dot `at`, counted from the left; dots are alike.
        function clickDot(l, at) {
            const ed = Line.editor(l);
            const p = ed.mapToItem(l, at * ed.width / ed.text.length, 0);
            mouseClick(l, p.x, l.height / 2);
        }

        function test_the_views_caret_stands_on_the_line_that_takes_the_letters() {
            const box = open(editing(0, "value", 1));
            const value = line(box, "fill.field-0-value");
            compare(carets(box.contentItem).length, 1, "one caret");
            compare(carets(value).length, 1, "on that line");
            const low = Line.xOf(value);
            box.view = editing(0, "value", 3);
            verify(Line.xOf(value) > low, "it moves as the view says");
            box.view = editing(-1, "", 2);
            const name = line(box, "fill.name");
            compare(carets(name).length, 1, "on the name");
            compare(carets(box.contentItem).length, 1, "and only there");
            const before = Line.xOf(name);
            box.view = editing(-1, "", 5);
            verify(Line.xOf(name) > before, "the name's moves too");
            box.visible = false;
        }

        // One caret serves the page, so a line that does not take the letters still shows its text's end.
        function test_a_line_that_does_not_take_the_letters_ignores_the_views_caret() {
            const box = open(editing(1, "value", 0));
            const other = line(box, "fill.field-0-value");
            compare(carets(other).length, 0);
            compare(Line.editor(other).visible, false, "its plain label draws it");
            compare(Kit.texts(other, box.view.sheet.fields[0].value.text).length, 1, "with its text");
            box.visible = false;
        }

        function test_a_masked_lines_caret_counts_its_dots() {
            const box = open(editing(2, "value", 1));
            const value = line(box, "fill.field-2-value");
            compare(Line.editor(value).text.length, box.view.sheet.fields[2].count, "one dot per letter");
            const low = Line.xOf(value);
            box.view = editing(2, "value", 4);
            const high = Line.xOf(value);
            verify(high > low, "on goes right");
            fuzzyCompare((high - low) / 3, Line.editor(value).width / Line.editor(value).text.length, 0.5, "by whole dots");
            box.visible = false;
        }

        // The line's own click comes first, so the module knows which line takes the place.
        function test_a_click_on_each_line_sends_its_own_click_then_its_place() {
            const box = open(editing(-1, "", 0));
            clickAt(line(box, "fill.name"), 3);
            compare(box.fill.calls, ["button name 0", "caret 3"], "the name");
            box.fill.calls = [];
            box.view = editing(0, "value", 0);
            clickAt(line(box, "fill.field-0-value"), 2);
            compare(box.fill.calls, ["field value 0", "caret 2"], "a value");
            box.fill.calls = [];
            box.view = editing(2, "label", 0);
            clickAt(line(box, "fill.field-2-label"), 4);
            compare(box.fill.calls, ["field label 2", "caret 4"], "an own label");
            box.visible = false;
        }

        // The click may land on a line the module has not told to take the letters yet, as its own click comes first.
        function test_a_click_on_a_line_that_does_not_take_the_letters_yet_sends_its_place() {
            const box = open(editing(1, "value", 0));
            const other = line(box, "fill.field-0-value");
            clickAt(other, 2);
            compare(box.fill.calls, ["field value 0", "caret 2"]);
            compare(new Set(box.fill.rounds).size, 1, "with the round drawn");
            box.visible = false;
        }

        function test_a_click_past_a_lines_end_lands_at_its_end() {
            const box = open(editing(0, "value", 0));
            const value = line(box, "fill.field-0-value");
            mouseClick(value, value.width - 2, value.height / 2);
            compare(box.fill.calls, ["field value 0", "caret " + box.view.sheet.fields[0].value.text.length]);
            box.visible = false;
        }

        // A hidden value stays hidden: its click carries a number of dots and its letters are never drawn.
        function test_a_click_on_a_masked_line_carries_only_its_dots() {
            const box = open(editing(2, "value", 0));
            const value = line(box, "fill.field-2-value");
            clickDot(value, 2);
            compare(box.fill.calls, ["field value 2", "caret 2"]);
            verify(Board.all(box.contentItem, o => o.font !== undefined).every(o => o.text !== "XK2-99"), "never drawn, even hidden");
            box.visible = false;
        }

        function test_a_click_on_the_password_line_sends_only_its_place() {
            const box = open(Kit.view("en", "pass-new"));
            const typed = line(box, "fill.password");
            clickAt(typed, 5);
            compare(box.fill.calls, ["caret 5"]);
            box.visible = false;
        }

        function test_the_password_lines_caret_moves_as_the_view_says() {
            const v = Kit.view("en", "pass-new");
            v.sheet.caret = 2;
            const box = open(v);
            const typed = line(box, "fill.password");
            const low = Line.xOf(typed);
            const on = Kit.view("en", "pass-new");
            on.sheet.caret = 6;
            box.view = on;
            verify(Line.xOf(typed) > low);
            box.visible = false;
        }

        // Readers hear the mask on a secret line, and the editor that draws it says nothing.
        function test_the_password_reaches_no_reader_through_the_editor() {
            const box = open(Kit.view("en", "pass-new"));
            const typed = line(box, "fill.password");
            compare(typed.Accessible.description, box.view.words.mask);
            verify(Line.editor(typed).Accessible.ignored);
            box.visible = false;
        }

        // A question answers first, on a line that still shows.
        function test_a_click_on_a_line_while_a_question_shows_sends_nothing() {
            const box = open(Kit.view("en", "confirm"));
            mouseClick(line(box, "fill.name"));
            compare(box.fill.calls, []);
            box.visible = false;
        }

        function test_no_line_ever_takes_focus() {
            const box = open(editing(1, "value", 2));
            for (const id of ["fill.name", "fill.field-0-value", "fill.field-1-value", "fill.field-2-label", "fill.field-2-value"])
                mouseClick(line(box, id));
            const editors = Board.all(box.contentItem, o => o.cursorPosition !== undefined);
            verify(editors.length > 0, "the page holds them");
            editors.forEach(e => compare(e.activeFocus, false));
            verify(box.flags & Qt.WindowDoesNotAcceptFocus, "the window takes none");
            box.visible = false;
        }
    }
}

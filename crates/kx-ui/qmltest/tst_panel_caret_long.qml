// A long panel line (ADR-0040): while it does not take the letters the newest show in either direction, a click lands on
// a letter that shows, and a press that turns it on moves no text under the pointer.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Clip
import "linekit.js" as Line
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 480
    height: 100

    readonly property string mask: "••••••••"
    // The fonts and the shared parts' names, once the bundled fonts load.
    property var res: null

    Component { id: lineC; PanelLine {} }
    Component { id: metricsC; TextMetrics {} }
    Component { id: uiC; FakeUi {} }
    Component { id: boxC; ClipBox {} }

    TestCase {
        name: "panel_caret_long"
        when: windowShown

        function initTestCase() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            root.res = Line.res(NL.lk, En);
            Pointer.through([Window.window]);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A line showing `said` with its caret at `at`, changed by `props`; it waits for the aim unless they say.
        function made(said, at, props) {
            const base = { said: said, at: at, hint: "hint", takes: false, rtl: false, lk: NL.lk, res: root.res, s: 1, width: root.width };
            return createTemporaryObject(lineC, root, Object.assign(base, props));
        }
        // A hidden line holding `count` dots, its caret at `at`, changed by `props`.
        function masked(at, count, props) {
            return made(null, at, Object.assign({ mask: root.mask, count: count, maskEm: NL.lk.shape.fill.mask_em }, props));
        }
        // Long text in each direction whose first and last words differ, so what shows tells which end stayed.
        function longs() {
            const long = (first, last) => first.repeat(40) + last.repeat(40);
            return [
                { first: "old ", last: "new ", said: Line.latin(long("old ", "new ")) },
                { first: "كتاب ", last: "قلم ", said: Line.arabic(long("كتاب ", "قلم ")) }
            ];
        }
        // What Qt keeps of `text` in `font` within `width` under `mode`.
        function elided(font, text, mode, width) {
            return createTemporaryObject(metricsC, root, { font: font, text: text, elide: mode, elideWidth: width }).elidedText;
        }
        // What the plain label of `line` shows.
        function shown(line) {
            const plain = Line.label(line);
            return elided(plain.font, plain.text, plain.elide, plain.width);
        }
        function hear(line) {
            const got = [];
            line.clicked.connect(() => got.push("clicked"));
            line.placed.connect(at => got.push(at));
            return got;
        }
        function clickAt(line, x) {
            mouseClick(line, x, line.height / 2);
        }
        function area(line) {
            return Board.all(line, o => o.acted !== undefined)[0];
        }
        // Moves the pointer onto `line` at `x`, as a hand does before a press.
        function point(line, x) {
            mouseMove(line, x, line.height / 2);
            tryVerify(() => area(line).containsMouse, 2000, "under the pointer");
        }
        // The place the editor holds under `x` of `line`.
        function under(line, x) {
            return line.placeAt(line.mapToItem(Line.editor(line), x, line.height / 2).x);
        }
        function inSight(line, where) {
            const x = Line.xOf(line);
            const [low, high] = [line.pad, line.width - line.pad - Line.caret(line).width];
            verify(x >= low - 0.5 && x <= high + 0.5, where + ": " + x + " inside " + low + " to " + high);
        }

        // Why the label's elide is one word for both directions: it follows the text's own order, not a side of the screen.
        function test_qt_elides_in_the_texts_own_order_whatever_its_direction() {
            for (const { first, last, said } of longs()) {
                const font = Line.label(made(said, 0)).font;
                const right = elided(font, said.text, Text.ElideRight, root.width / 2);
                verify(right.includes(first) && !right.includes(last), "ElideRight keeps the oldest: " + right);
                const left = elided(font, said.text, Text.ElideLeft, root.width / 2);
                verify(left.includes(last) && !left.includes(first), "ElideLeft keeps the newest: " + left);
            }
        }

        function test_a_long_line_that_does_not_take_the_letters_shows_the_newest() {
            for (const { first, last, said } of longs()) {
                const line = made(said, 0);
                const plain = Line.label(line);
                verify(plain.visible && plain.truncated, "it does not fit");
                verify(shown(line).includes(last) && !shown(line).includes(first), "the newest: " + shown(line));
            }
        }

        // The editor under the label stands at the text's end, so what a click finds there is what shows.
        function test_a_click_on_a_long_line_lands_on_a_letter_that_shows() {
            for (const { said } of longs()) {
                const line = made(said, 0);
                const got = hear(line);
                clickAt(line, line.width / 2);
                const place = got[1];
                verify(place > 0 && place < said.text.length, "inside the text: " + place);
                verify(shown(line).endsWith(said.text.slice(place)), "at " + place + " in: " + shown(line));
            }
            const written = longs()[1].said;
            const far = made(written, 0);
            const got = hear(far);
            clickAt(far, 1);
            compare(got, ["clicked", written.text.length], "past its left end is its newest letter");
        }

        // Dots count from the left on either keyboard, so the newest stay on the right.
        function test_a_long_mask_keeps_its_newest_dots_on_the_right_on_both_keyboards() {
            const dot = root.mask.charAt(0);
            for (const rtl of [false, true]) {
                const line = masked(0, 400, { rtl: rtl });
                verify(Line.label(line).truncated, "it does not fit");
                verify(shown(line).endsWith(dot) && !shown(line).startsWith(dot), "rtl " + rtl + ": " + shown(line));
                const got = hear(line);
                clickAt(line, line.width - line.pad - Line.caret(line).width);
                verify(got[1] > 395, "near the end, rtl " + rtl + ": " + got[1]);
            }
        }

        // The text stays while the press lasts, so the click lands on the letter seen there; only the click's place moves it.
        function test_a_press_that_turns_a_long_line_on_moves_no_text_under_the_pointer() {
            const lines = longs().map(l => () => made(l.said, l.said.text.length / 2)).concat([() => masked(200, 400, { rtl: true }), () => masked(200, 400)]);
            for (const make of lines) {
                const line = make();
                const ed = Line.editor(line);
                const x = line.width / 2;
                point(line, x);
                const stays = ed.x;
                const want = under(line, x);
                const got = hear(line);
                mousePress(line, x, line.height / 2);
                line.takes = true;
                compare(ed.x, stays, "the text stays at the flip");
                mouseRelease(line, x, line.height / 2);
                compare(got, ["clicked", want], "the letter under the pointer");
                line.at = want;
                compare(ed.x, stays, "and the caret lands in sight");
                inSight(line, "on the click's place");
                line.at = 0;
                verify(ed.x !== stays, "the next caret far away moves the text again");
                inSight(line, "on the next caret");
                line.visible = false;
            }
        }

        // A press on another part of the panel leaves no letter under the pointer to keep, so the caret comes into sight.
        function test_a_line_turned_on_with_the_pointer_elsewhere_brings_its_caret_into_sight() {
            for (const { said } of longs()) {
                const line = made(said, said.text.length / 2);
                mouseMove(root, 1, root.height - 1);
                tryVerify(() => !area(line).containsMouse, 2000, "the pointer is off the line");
                const stays = Line.editor(line).x;
                line.takes = true;
                verify(Line.editor(line).x !== stays, "the text moves");
                inSight(line, "at the flip");
            }
        }

        // A view can shorten the text while the pointer still holds it, and a line let go shows its end again.
        function test_a_kept_text_never_stands_past_its_end_and_comes_back_when_let_go() {
            const { said } = longs()[0];
            const line = made(said, said.text.length / 2);
            const ed = Line.editor(line);
            point(line, line.width / 2);
            const end = ed.x;
            line.takes = true;
            compare(ed.x, end, "it stays");
            line.takes = false;
            compare(ed.x, end, "and it shows the end again");
            line.at = 3;
            line.takes = true;
            compare(ed.x, end, "kept again");
            line.said = Line.latin("abc");
            fuzzyCompare(Line.spot(line, 0), line.pad, 0.51, "the short text stands from its start");
            inSight(line, "in the short text");
        }

        // End to end on the clipboard's search, whose press asks the aim as the app's does.
        function test_a_press_on_a_panels_long_search_lands_on_the_letter_under_it() {
            for (const { said } of longs()) {
                const v = Clip.view("en", "list");
                v.search = Object.assign({}, said, { hint: v.search.hint, caret: said.text.length / 2 });
                const box = createTemporaryObject(boxC, root, { ui: createTemporaryObject(uiC, root), lk: NL.lk, res: root.res, s: 1, view: v, rows: En.settings.pageRows, rtl: false });
                Pointer.through([box]);
                box.open();
                verify(waitForPolish(box), "laid out");
                tryVerify(() => box.body.scale === 1, 2000, "popped in");
                box.aiming.connect(() => box.aimed = true);
                const search = Clip.only(Board.all(box.contentItem, o => o.visible && o.Accessible.id === "clip.search"));
                const ed = Line.editor(search);
                const x = search.width / 2;
                point(search, x);
                const stays = ed.x;
                const want = under(search, x);
                clickAt(search, x);
                verify(box.aimed, "the press aimed it");
                compare(ed.x, stays, "no text moved");
                compare(box.ui.calls.filter(c => c.startsWith("clipAct")), ["clipAct caret " + want]);
                box.visible = false;
            }
        }
    }
}

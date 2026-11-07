// A click on a panel line sends the place it lands on (ADR-0040), and the editor that draws the letters looks as the
// plain label does, takes no focus and stays silent for readers.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "linekit.js" as Line
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 480
    height: 100

    readonly property string mask: "••••••••"
    readonly property string book: "كتاب"
    // Its lam and alef draw as one shape.
    readonly property string peace: "سلام"
    // A letter with a haraka, then a letter.
    readonly property string marked: "بَت"
    // The fonts and the shared parts' names, once the bundled fonts load.
    property var res: null

    Component { id: lineC; PanelLine {} }
    Component { id: metricsC; TextMetrics {} }

    TestCase {
        name: "panel_caret_click"
        when: windowShown

        function initTestCase() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            root.res = Line.res(NL.lk, En);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A line that takes the letters, showing `said` with its caret at `at`, changed by `props`.
        function made(said, at, props) {
            const base = { said: said, at: at, hint: "hint", takes: true, rtl: false, lk: NL.lk, res: root.res, s: 1, width: root.width };
            return createTemporaryObject(lineC, root, Object.assign(base, props));
        }
        // A hidden line holding `count` dots of `mask`, its caret at `at`, changed by `props`.
        function masked(at, count, mask, props) {
            return made(null, at, Object.assign({ mask: mask, count: count, maskEm: NL.lk.shape.fill.mask_em }, props));
        }
        // Every click on `line` as it is heard, in order.
        function hear(line) {
            const got = [];
            line.clicked.connect(() => got.push("clicked"));
            line.placed.connect(at => got.push(at));
            return got;
        }
        // Clicks `line` at `x` from its edge, half way down.
        function clickAt(line, x) {
            mouseClick(line, x, line.height / 2);
        }

        // Each part looks as the plain label does, so a line taking the letters changes nothing but the caret.
        function test_the_editor_draws_as_the_label_does_in_every_font() {
            for (const said of [Line.latin("abc"), Line.arabic(root.book)]) {
                const line = made(said, said.text.length);
                const ed = Line.editor(line);
                const plain = Line.label(line);
                verify(ed.visible && !plain.visible, "the editor while it takes the letters");
                compare(ed.font.family, plain.font.family);
                compare(ed.font.pixelSize, plain.font.pixelSize);
                compare(ed.font.letterSpacing, plain.font.letterSpacing);
                compare(ed.font.hintingPreference, plain.font.hintingPreference);
                compare(ed.renderType, plain.renderType);
                verify(Qt.colorEqual(ed.color, plain.color));
                line.takes = false;
                verify(!ed.visible && plain.visible, "the label while it does not");
                compare(Line.caret(line).visible, false);
            }
            const dots = masked(1, 3, root.mask);
            verify(Qt.colorEqual(Line.editor(dots).color, NL.lk.palette.pop_muted), "a mask is muted");
            compare(Line.editor(dots).font.letterSpacing, Line.label(dots).font.letterSpacing, "a mask keeps its spacing");
            compare(Line.editor(dots).text, "•••", "and shows only dots");
        }

        function test_the_label_and_the_editor_start_in_the_same_spot() {
            for (const [said, end] of [[Line.latin("abc"), 0], [Line.arabic(root.book), 1]]) {
                const line = made(said, said.text.length);
                const ed = Line.editor(line);
                const plain = Line.label(line);
                const at = o => o.mapToItem(line, 0, 0).x + end * o.width;
                fuzzyCompare(at(ed), at(plain), 1, said.text);
            }
        }

        // The click's order is the line's own, then the caret; a letter's near half is before it, the far half after it.
        function test_a_click_on_a_letter_sends_its_place_after_the_lines_click() {
            const line = made(Line.latin("WWWWWW"), 0);
            const got = hear(line);
            const step = Line.spot(line, 1) - Line.spot(line, 0);
            clickAt(line, Line.spot(line, 2) + step / 4);
            clickAt(line, Line.spot(line, 2) + 3 * step / 4);
            compare(got, ["clicked", 2, "clicked", 3]);
        }

        function test_a_click_past_the_end_lands_at_the_end_and_before_the_start_at_the_start() {
            const line = made(Line.latin("abc"), 0);
            const got = hear(line);
            clickAt(line, line.width - 2);
            clickAt(line, 1);
            compare(got, ["clicked", 3, "clicked", 0]);
        }

        // The whole line takes the click, above and below the letters too.
        function test_a_click_above_or_below_the_letters_still_lands_on_their_place() {
            const line = made(Line.latin("WWWWWW"), 0);
            const got = hear(line);
            const x = Line.spot(line, 2) + 2;
            mouseClick(line, x, 1);
            mouseClick(line, x, line.height - 1);
            compare(got.filter(g => g !== "clicked"), [2, 2]);
        }

        // A space at the end is a letter like any other, so the caret goes past it, in either direction.
        function test_a_trailing_space_moves_the_caret() {
            const plain = made(Line.latin("abc"), 3);
            const spaced = made(Line.latin("abc "), 4);
            verify(Line.xOf(spaced) > Line.xOf(plain), "on past the space");
            const got = hear(spaced);
            clickAt(spaced, spaced.width - 2);
            compare(got, ["clicked", 4], "and a click past it lands after it");
            const written = made(Line.arabic(root.book), root.book.length);
            const after = made(Line.arabic(root.book + " "), root.book.length + 1);
            verify(Line.xOf(after) < Line.xOf(written), "on past the space, going left");
        }

        function test_a_click_on_arabic_sends_its_place_from_the_right() {
            const line = made(Line.arabic(root.book), 0);
            const got = hear(line);
            for (const at of [0, 1, 2, 3, 4]) {
                const shift = at === 0 ? -1 : 1;
                clickAt(line, Line.spot(line, at) + shift);
            }
            clickAt(line, 1);
            compare(got.filter(g => g !== "clicked"), [0, 1, 2, 3, 4, 4], "past its left end is its end");
        }

        // Lam and alef draw as one shape, yet a click reaches the place between them.
        function test_a_click_on_a_lam_alef_lands_between_its_two_letters() {
            const line = made(Line.arabic(root.peace), 0);
            const got = hear(line);
            for (const at of [0, 1, 2, 3, 4]) {
                const shift = at === 0 ? -1 : 1;
                clickAt(line, Line.spot(line, at) + shift);
            }
            compare(got.filter(g => g !== "clicked"), [0, 1, 2, 3, 4]);
        }

        // A haraka sits on its letter, so no click lands between them, wherever it falls on the pair.
        function test_a_click_never_lands_between_a_letter_and_its_haraka() {
            const line = made(Line.arabic(root.marked), 0);
            const got = hear(line);
            const [left, right] = [Line.spot(line, 2), Line.spot(line, 0)];
            verify(right > left, "the pair has width");
            for (let x = left; x <= right; x += 1)
                clickAt(line, x);
            const places = got.filter(g => g !== "clicked");
            verify(places.every(p => p === 0 || p === 2), "only its edges: " + places);
            compare(places[0], 2, "its left edge is after it");
            compare(places[places.length - 1], 0, "its right edge is before it");
        }

        function test_a_click_on_mixed_text_lands_on_the_run_it_hits() {
            const text = "abc " + root.book;
            const line = made({ text: text, rtl: false, arabic: true }, 0);
            const got = hear(line);
            clickAt(line, Line.spot(line, 1) + 1);
            clickAt(line, Line.spot(line, 6) + 1);
            compare(got.filter(g => g !== "clicked"), [1, 6]);
        }

        // A hidden value's click carries only a number of dots, counted from the left on either keyboard.
        function test_a_click_on_a_mask_counts_dots_from_the_left() {
            for (const rtl of [false, true]) {
                const line = masked(0, 6, root.mask, { rtl: rtl });
                const got = hear(line);
                const ed = Line.editor(line);
                const each = ed.width / 6;
                const dot = n => ed.mapToItem(line, n * each, 0).x;
                clickAt(line, dot(2));
                clickAt(line, dot(5) + 1);
                clickAt(line, rtl ? 1 : line.width - 2);
                compare(got.filter(g => g !== "clicked"), [2, 5, rtl ? 0 : 6], "rtl " + rtl);
                got.forEach(g => verify(g === "clicked" || typeof g === "number", "only a number"));
            }
        }

        // A line a press has just aimed may not draw the caret yet; its click still lands where it was shown.
        function test_a_click_on_a_line_that_does_not_take_the_letters_yet_still_places_the_caret() {
            const line = made(Line.latin("WWWWWW"), 6, { takes: false });
            const got = hear(line);
            const wide = createTemporaryObject(metricsC, root, { font: Line.label(line).font, text: "WW" }).advanceWidth;
            clickAt(line, line.pad + wide + 1);
            compare(got, ["clicked", 2]);
        }

        function test_a_click_on_an_empty_line_places_the_caret_at_its_start() {
            const line = made(Line.latin(""), 0);
            const got = hear(line);
            clickAt(line, line.width / 2);
            compare(got, ["clicked", 0]);
        }

        // A line whose page asks a question does nothing at all.
        function test_a_click_on_a_line_that_may_not_aim_sends_nothing() {
            const line = made(Line.latin("abc"), 0, { aims: false });
            const got = hear(line);
            clickAt(line, 4);
            compare(got, []);
        }

        function test_the_editor_never_takes_focus_or_a_press() {
            const line = made(Line.latin("abcdef"), 2);
            const ed = Line.editor(line);
            compare(ed.enabled, false, "takes no event");
            compare(ed.readOnly, true);
            compare(ed.activeFocusOnPress, false);
            compare(ed.activeFocusOnTab, false);
            compare(ed.selectByMouse, false);
            compare(ed.cursorVisible, false, "the caret is ours");
            verify(ed.Accessible.ignored, "readers hear the line");
            for (const x of [line.pad, line.width / 2])
                clickAt(line, x);
            compare(ed.activeFocus, false, "after clicks");
            compare(ed.cursorVisible, false, "still not Qt's");
        }

        // A secret shows on screen but readers hear the mask, as the label before it did.
        function test_readers_hear_the_line_not_the_editor() {
            const line = made(Line.latin("hunter2"), 3, { secret: true, mask: root.mask });
            compare(line.Accessible.description, root.mask);
            verify(Line.editor(line).Accessible.ignored);
            const dots = masked(1, 3, root.mask);
            compare(dots.Accessible.description, root.mask);
        }
    }
}

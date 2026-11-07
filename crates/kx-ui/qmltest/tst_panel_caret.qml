// A panel line's caret on screen (ADR-0040): it stands where Qt's layout puts its place in Latin, Arabic and mixed text,
// counts dots on a mask, and stays in sight as the text moves under it.
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
    // A letter with a haraka, then a letter, and the same two with none.
    readonly property string marked: "بَت"
    readonly property string bare: "بت"
    // The fonts and the shared parts' names, once the bundled fonts load.
    property var res: null

    Component { id: lineC; PanelLine {} }
    Component { id: metricsC; TextMetrics {} }

    TestCase {
        name: "panel_caret"
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
        // Where the caret stands at each place from 0 to `last`.
        function walk(line, last) {
            const seen = [];
            for (let at = 0; at <= last; at++) {
                line.at = at;
                seen.push(Line.xOf(line));
            }
            return seen;
        }
        // The caret's first and last spots in the line: the text's box less the caret's own width.
        function ends(line) {
            return [line.pad, line.width - line.pad - Line.caret(line).width];
        }
        function inSight(line, where) {
            const [low, high] = ends(line);
            const x = Line.xOf(line);
            verify(x >= low - 0.5 && x <= high + 0.5, where + ": " + x + " inside " + low + " to " + high);
        }
        // The natural width of `text` in `line`'s font.
        function widthOf(line, text) {
            return createTemporaryObject(metricsC, root, { font: Line.label(line).font, text: text }).advanceWidth;
        }

        function test_the_caret_steps_along_latin_text_and_ends_where_the_text_ends() {
            const text = "abcdef";
            const line = made(Line.latin(text), 0);
            const steps = walk(line, text.length);
            fuzzyCompare(steps[0], line.pad, 0.01, "at the start");
            for (let i = 1; i < steps.length; i++)
                verify(steps[i] > steps[i - 1], "on at " + i + ": " + steps);
            fuzzyCompare(steps[text.length], line.pad + widthOf(line, text), 1, "after the last letter, as it stood before");
        }

        // Joined letters differ in width, so only Qt's layout places it; a line reading right to left starts on the right.
        function test_the_caret_steps_right_to_left_along_joined_arabic() {
            const line = made(Line.arabic(root.book), 0);
            const steps = walk(line, root.book.length);
            const [low, high] = ends(line);
            fuzzyCompare(steps[0], high, 0.51, "on the right, before the first letter");
            for (let i = 1; i < steps.length; i++)
                verify(steps[i] < steps[i - 1], "on to the left at " + i + ": " + steps);
            verify(steps[root.book.length] > low, "short text never reaches the far side");
        }

        // Lam and alef draw as one shape, yet each is a letter of its own, so the caret still stands between them.
        function test_the_caret_stands_between_the_two_letters_of_a_lam_alef() {
            const line = made(Line.arabic(root.peace), 0);
            const steps = walk(line, root.peace.length);
            for (let i = 1; i < steps.length; i++)
                verify(steps[i] < steps[i - 1], "on to the left at " + i + ": " + steps);
        }

        // A haraka sits on its letter and takes no room, so the caret after the pair stands where it would without it.
        function test_the_caret_passes_a_letter_and_its_haraka_as_one() {
            const line = made(Line.arabic(root.marked), 0);
            const steps = walk(line, root.marked.length);
            verify(steps[2] < steps[0] && steps[3] < steps[2], "on to the left: " + steps);
            const bare = made(Line.arabic(root.bare), 0);
            const plain = walk(bare, root.bare.length);
            fuzzyCompare(steps[2], plain[1], 1, "after the letter, with or without its haraka");
            fuzzyCompare(steps[3], plain[2], 1, "after the next");
        }

        // Each run moves the way it reads, wherever its letters stand in a line of the other direction.
        function test_the_caret_follows_each_run_of_mixed_text() {
            const text = "abc " + root.book;
            const line = made({ text: text, rtl: false, arabic: true }, 0);
            const steps = walk(line, text.length);
            for (const i of [1, 2, 3, 4])
                verify(steps[i] > steps[i - 1], "Latin and its space go right at " + i + ": " + steps);
            verify(steps[5] > steps[4], "the first Arabic letter stands at the run's far side: " + steps);
            for (const i of [6, 7, 8])
                verify(steps[i] < steps[i - 1], "Arabic goes left at " + i + ": " + steps);
        }

        function test_a_right_to_left_line_keeps_latin_runs_reading_right() {
            const text = root.book + " abc";
            const line = made(Line.arabic(text), 0);
            const steps = walk(line, text.length);
            for (const i of [1, 2, 3, 4, 5])
                verify(steps[i] < steps[i - 1], "Arabic and its space go left at " + i + ": " + steps);
            verify(steps[6] < steps[5], "the first Latin letter stands at the run's far side: " + steps);
            for (const i of [7, 8])
                verify(steps[i] > steps[i - 1], "Latin goes right at " + i + ": " + steps);
        }

        function test_the_caret_counts_dots_from_the_left_in_even_steps() {
            const count = 6;
            const line = masked(0, count, root.mask);
            const steps = walk(line, count);
            const each = steps[1] - steps[0];
            verify(each > 0, "on goes right");
            for (let i = 1; i < steps.length; i++)
                fuzzyCompare(steps[i] - steps[i - 1], each, 0.5, "the same step at " + i);
        }

        // Left goes back on a mask whatever the keyboard's language, so the dots count from the left there too, and the
        // caret after the newest dot stays on the language's side.
        function test_a_mask_on_a_right_to_left_keyboard_counts_from_the_left_and_stays_on_its_side() {
            const count = 6;
            const line = masked(0, count, root.mask, { rtl: true });
            const steps = walk(line, count);
            for (let i = 1; i < steps.length; i++)
                verify(steps[i] > steps[i - 1], "on goes right at " + i + ": " + steps);
            const empty = made(Line.latin(""), 0, { rtl: true });
            fuzzyCompare(steps[count], Line.xOf(empty), 0.51, "at the end, where the empty line waits");
        }

        // Dots hold no direction, so Qt lays them out by the OS keyboard's: a mask of its own direction shows the count
        // does not follow it.
        function test_a_mask_counts_from_the_left_however_qt_lays_it_out() {
            const count = 4;
            const steps = walk(masked(0, count, "ع"), count);
            for (let i = 1; i < steps.length; i++)
                verify(steps[i] > steps[i - 1], "on goes right at " + i + ": " + steps);
        }

        function test_a_long_line_keeps_the_caret_in_sight_at_both_ends_with_the_text_under_it() {
            const text = "abcdefghij".repeat(30);
            const line = made(Line.latin(text), 0);
            const ed = Line.editor(line);
            const [low, high] = ends(line);
            verify(ed.width > line.width, "longer than the line");
            fuzzyCompare(Line.xOf(line), low, 0.01, "at the start");
            fuzzyCompare(Line.spot(line, 0), low, 0.51, "the first letter shows");
            line.at = text.length;
            fuzzyCompare(Line.xOf(line), high, 0.51, "at the end, as it stood before");
            verify(Line.spot(line, 0) < 0, "the start has gone under");
            for (const at of [299, 150, 149, 1, 0, 1, 150, 300]) {
                line.at = at;
                inSight(line, "at " + at);
            }
        }

        function test_a_long_right_to_left_line_keeps_the_caret_in_sight_at_both_ends() {
            const text = (root.book + " ").repeat(40);
            const line = made(Line.arabic(text), 0);
            const [low, high] = ends(line);
            verify(Line.editor(line).width > line.width, "longer than the line");
            fuzzyCompare(Line.xOf(line), high, 0.51, "at the start, on the right");
            line.at = text.length;
            fuzzyCompare(Line.xOf(line), low, 0.51, "at the end, on the left");
            verify(Line.spot(line, 0) > line.width, "the start has gone under");
            for (const at of [text.length - 1, 80, 79, 1, 0, 1, 80]) {
                line.at = at;
                inSight(line, "at " + at);
            }
        }

        function test_a_long_mask_keeps_the_caret_in_sight_at_both_ends() {
            for (const rtl of [false, true]) {
                const line = masked(0, 400, root.mask, { rtl: rtl });
                const [low, high] = ends(line);
                fuzzyCompare(Line.xOf(line), low, 0.51, "first dot, rtl " + rtl);
                line.at = 400;
                fuzzyCompare(Line.xOf(line), high, 0.51, "after the last dot, rtl " + rtl);
                line.at = 200;
                inSight(line, "half way, rtl " + rtl);
            }
        }

        // A caret still in sight moves over the text, and a letter put at it pushes only what follows, so nothing jumps.
        function test_the_text_stays_put_while_the_caret_is_in_sight() {
            const text = "abcdefghij".repeat(30);
            const line = made(Line.latin(text), text.length);
            const ed = Line.editor(line);
            line.at = 200;
            line.at = 205;
            const under = ed.x;
            const at = Line.xOf(line);
            line.at = 206;
            compare(ed.x, under, "the caret alone moved");
            verify(Line.xOf(line) > at, "to the right");
            line.said = Line.latin(text.slice(0, 206) + "Z" + text.slice(206));
            line.at = 207;
            compare(ed.x, under, "a letter put at the caret moves no text before it");
            verify(Line.xOf(line) > at, "and the caret stays by it");
            inSight(line, "after the letter");
        }

        // A right-to-left line stays put from its right side: the letters before the caret keep their place.
        function test_a_right_to_left_line_stays_put_from_its_right_side() {
            const text = (root.book + " ").repeat(40);
            const line = made(Line.arabic(text), 0);
            const first = Line.spot(line, 0);
            line.at = 3;
            fuzzyCompare(Line.spot(line, 0), first, 0.01, "in sight, so the text stays");
            line.said = Line.arabic(text.slice(0, 3) + "ب" + text.slice(3));
            line.at = 4;
            fuzzyCompare(Line.spot(line, 0), first, 0.51, "the letters before it stay where they were");
        }

        function test_a_shorter_text_brings_a_moved_line_back() {
            const text = "abcdefghij".repeat(30);
            const line = made(Line.latin(text), text.length);
            line.said = Line.latin("abc");
            line.at = 3;
            fuzzyCompare(Line.spot(line, 0), line.pad, 0.51, "the whole text fits again, from its start");
            line.said = Line.latin("");
            line.at = 0;
            fuzzyCompare(Line.xOf(line), line.pad, 0.01, "an empty line waits on its side");
        }

        // A caret left at a letter's place still draws at the end of a same-length text of another width, and a new width
        // of the line moves it.
        function test_new_text_of_the_same_length_and_a_new_width_move_the_caret() {
            const line = made(Line.latin("iii"), 3);
            const thin = Line.xOf(line);
            line.said = Line.latin("WWW");
            verify(Line.xOf(line) > thin, "wider letters push it");
            const long = "abcdefghij".repeat(30);
            line.said = Line.latin(long);
            line.at = long.length;
            const right = Line.xOf(line);
            line.width = root.width / 2;
            verify(Line.xOf(line) < right, "a narrower line keeps it in sight");
            inSight(line, "narrower");
        }

        // The panel shrinks to fit the room, and the keyboard's language can change under an open mask.
        function test_a_new_size_or_keyboard_language_keeps_the_caret_at_its_place() {
            const line = made(Line.latin("abcdef"), 3);
            const small = Line.xOf(line);
            line.s = 1.5;
            verify(Line.xOf(line) > small, "larger letters push it");
            const dots = masked(3, 3, root.mask);
            const left = Line.xOf(dots);
            dots.rtl = true;
            fuzzyCompare(Line.xOf(dots), ends(dots)[1], 0.51, "the dots go to the language's side, the caret after them");
            verify(Line.xOf(dots) > left);
        }
    }
}

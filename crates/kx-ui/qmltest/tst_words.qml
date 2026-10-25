// The chip lane on the top bar, drawn from the lanes the hub writes: chips fit whole, best first from the left, Arabic
// in its font right to left; clicks reach the hub; the lock note, forget mode and its names, Undo, More and the ids
// gates find.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "lanes.js" as Lanes
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/words.js" as Words
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD
import "../../kx-theme/fixtures/look-native-light-contrast.js" as HC
import "pointer.js" as Pointer

Item {
    id: root
    width: 760
    height: 120

    readonly property var res: Board.resOf(NL.lk, En)

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: barC; TopBar {} }
    Component { id: chipC; Chip {} }
    Component { id: namesC; WordNames {} }

    TestCase {
        name: "words"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A top bar in look `lk` (Native light by default) drawing `lane`, its texts in a language that reads right to
        // left when `rtl`.
        function bar(lane, rtl, lk) {
            const props = { box: Face.board.bar, lk: lk || NL.lk, res: root.res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.lang.sizes, dark: false, lane: lane, rtl: !!rtl };
            const b = createTemporaryObject(barC, root, props);
            waitForRendering(b);
            return b;
        }
        // A keyboard window over fresh stand-ins whose hub gives `lane`, shown; the real pointer passes through it.
        function win(lane) {
            const ui = createTemporaryObject(uiC, root, { lane: lane });
            const w = createTemporaryObject(winC, root, { ui: ui, keyboard: createTemporaryObject(kbC, root) });
            Pointer.through([w]);
            w.visible = true;
            waitForRendering(w.contentItem);
            return w;
        }
        // Every lane chip under `item`, shown or not, in lane order.
        function chips(item) {
            return Board.all(item, o => o.chip !== undefined && o.Accessible.id.startsWith("words.chip-"));
        }
        function shown(item) {
            return chips(item).filter(c => c.visible);
        }
        // The shown controls under `item` with UI Automation id `id`.
        function ids(item, id) {
            return Board.all(item, o => o.visible && o.Accessible.id === id);
        }
        // The one shown control with UI Automation id `id`.
        function keyed(item, id) {
            const hits = ids(item, id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        function named(item, name) {
            return Board.all(item, o => o.visible && o.Accessible.name === name);
        }
        function label(chip) {
            return Board.all(chip, o => o.font !== undefined && o.text === chip.chip.text)[0];
        }

        function test_seven_chips_where_five_fit_show_five_whole_best_first() {
            const words = ["one", "two", "three", "four", "five", "six", "seven"];
            const b = bar(Lanes.words(words));
            const all = chips(b);
            compare(all.length, 7, "every chip the hub sent is made");
            const row = all[0].parent;
            const gap = NL.lk.shape.chip.gap_px;
            const five = all.slice(0, 5).reduce((w, c) => w + c.width, 4 * gap);
            const room = five + gap + all[5].width / 2;
            b.box = Object.assign({}, Face.board.bar, { w: Face.board.bar.w - (row.width - room) });
            tryVerify(() => Math.abs(row.width - room) < 0.01, 1000, "room for five and a half");
            waitForRendering(b);
            compare(shown(b).map(c => c.chip.text), words.slice(0, 5), "best first, the rest dropped");
            for (const c of shown(b))
                verify(c.x + c.width <= row.width, c.chip.text + " is whole");
        }

        // A chip that would not fit whole drops with every chip after it, even one short enough to fit.
        function test_a_long_word_drops_with_every_chip_after_it() {
            const long = "supercalifragilisticexpialidocious-antidisestablishmentarianism";
            const b = bar(Lanes.words(["the", "they", long, "then"]));
            compare(shown(b).map(c => c.chip.text), ["the", "they"]);
            const row = chips(b)[0].parent;
            for (const c of shown(b))
                verify(c.x + c.width <= row.width, c.chip.text + " is whole, never cut or squeezed");
            compare(chips(b)[2].width, chips(b)[2].full, "the long chip keeps its whole width");
        }

        // Arabic flips text only (ADR-0020): its chip reads right to left in the Arabic font, and the row keeps its order.
        function test_an_arabic_chip_takes_the_arabic_font_and_reads_right_to_left() {
            const b = bar(Lanes.some([0, 1, 2]), true);
            const [latin, arabic, domain] = shown(b);
            compare(label(arabic).font.family, root.res.fam.arabic);
            compare(label(arabic).font.pixelSize, Math.round(NL.lk.shape.chip.font_px[1]), "the Arabic size");
            compare(label(arabic).effectiveHorizontalAlignment, Text.AlignRight, "Arabic reads right to left");
            compare(label(latin).font.family, root.res.fam.latin);
            compare(label(latin).effectiveHorizontalAlignment, Text.AlignLeft);
            const xs = [latin, arabic, domain].map(c => c.mapToItem(b, 0, 0).x);
            verify(xs[0] < xs[1] && xs[1] < xs[2], "best first from the left: " + xs);
        }

        function test_a_click_on_chip_2_picks_it_on_the_drawn_round() {
            const w = win(Lanes.some([0, 1, 2]));
            mouseClick(keyed(w.contentItem, "words.chip-2"));
            compare(w.ui.calls.filter(c => c.startsWith("pickChip")), ["pickChip " + Words.lanes.chips.round + " 2"]);
        }

        // In a password box the lock and its note stand in for chips; the menu button stays in every kind.
        function test_the_lock_off_and_empty_lanes_keep_the_menu_button() {
            const w = win(Words.lanes.locked);
            const note = keyed(w.contentItem, "words.locked");
            compare(note.Accessible.name, En.texts["chip.locked"]);
            compare(named(w.contentItem, En.texts["chip.locked"]), [note], "found by its name");
            const lock = Board.all(note, o => o.tint !== undefined && o.icon !== undefined)[0];
            compare(lock.icon, NL.lk.icons.words.locked);
            verify(Qt.colorEqual(lock.tint, NL.lk.palette.legend_2_plate), "plate text, as a note, not a button");
            compare(shown(w.contentItem).length, 0, "no chips");
            keyed(w.contentItem, "words.menu");
            w.ui.lane = Words.lanes.off;
            compare(shown(w.contentItem).length, 0, "off shows no chips");
            compare(ids(w.contentItem, "words.locked").length, 0, "nor the note");
            compare(keyed(w.contentItem, "words.menu").Accessible.name, En.texts["words.menu"]);
            w.ui.lane = Words.lanes.empty;
            compare(keyed(w.contentItem, "words.lane").Accessible.name, En.texts["chip.none"]);
            w.ui.lane = Lanes.some([0]);
            compare(keyed(w.contentItem, "words.lane").Accessible.name, "", "a lane with chips has them to say");
        }

        // Forget mode, as the hub sends it, holds word chips only: each wears the eraser in the danger keys' colours and
        // its name says a click forgets it, and Cancel ends the row.
        function test_forget_mode_marks_the_chips_and_ends_the_row_with_cancel() {
            const lane = Words.lanes.forget;
            verify(lane.chips.every(c => c.kind === "word"), "words only");
            const w = win(lane);
            const marked = shown(w.contentItem);
            compare(marked.length, lane.chips.length);
            for (const [i, c] of marked.entries()) {
                verify(c.forget, c.chip.text);
                const icon = Board.all(c, o => o.tint !== undefined && o.icon !== undefined)[0];
                compare(icon.icon, NL.lk.icons.words.forget);
                compare(c.Accessible.name, En.texts["chip.forget-name"].replace("{$word}", lane.chips[i].text));
            }
            const cancel = keyed(w.contentItem, "words.cancel");
            compare(cancel.Accessible.name, En.texts["words.cancel"]);
            verify(!cancel.forget, "Cancel keeps the chip look");
            verify(cancel.mapToItem(null, 0, 0).x > marked[marked.length - 1].mapToItem(null, 0, 0).x, "it ends the row");
            mouseClick(cancel);
            compare(w.ui.calls.filter(c => c.startsWith("wordsAsk")), ["wordsAsk " + Words.asks.cancel + " 0"]);
        }

        // Undo and More show their icon and text, found by name; More opens the list, Undo is picked like a word.
        function test_undo_and_more_are_found_by_name_and_more_asks_for_the_list() {
            const w = win(Lanes.some([0, 4, 5]));
            const [undo] = named(w.contentItem, En.texts["chip.undo"]);
            const [more] = named(w.contentItem, En.texts["chip.more"]);
            compare([undo.chip.text, more.chip.text], [En.texts["chip.undo"], En.texts["chip.more"]]);
            compare([undo.chip.icon, more.chip.icon], [NL.lk.icons.words.undo, NL.lk.icons.words.more]);
            mouseClick(more);
            mouseClick(undo);
            const sent = w.ui.calls.filter(c => c.startsWith("pickChip") || c.startsWith("wordsAsk"));
            compare(sent, ["wordsAsk " + Words.asks.more + " 0", "pickChip " + Words.lanes.chips.round + " 1"]);
        }

        // Gates find every chip by its place and the domain and email chips wear the at-sign.
        function test_each_chip_has_its_gate_id_and_its_kind_icon() {
            const b = bar(Lanes.some([2, 3]));
            compare(shown(b).map(c => c.Accessible.id), ["words.chip-0", "words.chip-1"]);
            compare(shown(b).map(c => c.Accessible.name), Lanes.some([2, 3]).chips.map(c => c.name));
            compare(shown(b).map(c => c.chip.icon), [NL.lk.icons.words.domain, NL.lk.icons.words.email]);
            compare(ids(b, "words.menu").length, 1, "the menu button");
        }

        // QML's ask kinds are the hub's own, as kx_ui::asks names them.
        function test_the_ask_kinds_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            compare(Object.keys(names.asks).sort(), Object.keys(Words.asks).sort());
            for (const k in Words.asks)
                compare(names.asks[k], Words.asks[k], k);
        }

        // Chips draw the pairs kx-theme checks in every theme and mode, high contrast too: the word in legend and the icon in
        // legend_2_plate (legend in high contrast) on chip_bg; in forget mode both in danger_ink on danger.
        function test_chips_take_the_pairs_the_theme_checks_in_every_look() {
            for (const lk of [NL.lk, ND.lk, EL.lk, ED.lk, DL.lk, DD.lk, HC.lk]) {
                for (const forget of [false, true]) {
                    const chip = { text: "hello", icon: lk.icons.words.email };
                    const c = createTemporaryObject(chipC, root, { chip: chip, lk: lk, res: root.res, s: 1, radius: 4, forget: forget });
                    const p = lk.palette;
                    const where = lk.look.id + (lk.dark ? " dark" : " light") + (lk.contrast ? ", contrast" : "") + (forget ? ", forget" : "");
                    verify(Qt.colorEqual(c.fill, forget ? p.danger : p.chip_bg), where);
                    verify(Qt.colorEqual(label(c).color, forget ? p.danger_ink : p.legend), where);
                    const icon = Board.all(c, o => o.tint !== undefined && o.icon !== undefined)[0];
                    const mark = forget ? p.danger_ink : lk.contrast ? p.legend : p.legend_2_plate;
                    verify(Qt.colorEqual(icon.tint, mark), where);
                }
            }
        }
    }
}

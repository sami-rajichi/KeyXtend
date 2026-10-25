// The words menu on the top bar's row: its items take the chips' place in the keyboard window, with no new row or
// window, name Resume and On by the switches, send their asks to the hub, close, and show icons alone with tips when
// their texts do not all fit.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "lanes.js" as Lanes
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "fixtures/words.js" as Words
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "pointer.js" as Pointer

Item {
    id: root
    width: 760
    height: 120

    readonly property var res: Board.resOf(NL.lk, En)
    // The menu's items by their gate ids, in order.
    readonly property var items: ["words.menu-forget", "words.menu-pause", "words.menu-power", "words.menu-cancel"]

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: barC; TopBar {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "wordmenu"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
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
        function topBar(w) {
            return Board.all(w.contentItem, o => o.lane !== undefined && o.presets !== undefined)[0];
        }
        // Opens the menu in `w` and gives its items once they are laid out and drawn.
        function open(w) {
            mouseClick(keyed(w.contentItem, "words.menu"));
            waitForRendering(w.contentItem);
            return root.items.map(id => keyed(w.contentItem, id));
        }
        // The hub calls made since the last look.
        function asks(w) {
            const said = w.ui.calls.filter(c => c.startsWith("wordsAsk") || c.startsWith("pickChip"));
            w.ui.calls = [];
            return said;
        }

        function test_the_menu_opens_in_place_on_the_same_row() {
            const w = win(Lanes.some([0, 1]));
            const bar = topBar(w);
            const high = bar.height;
            const chip = keyed(w.contentItem, "words.chip-0");
            const menu = keyed(w.contentItem, "words.menu");
            const shown = open(w);
            const shelf = shown[0].parent.parent;
            tryCompare(shelf, "opacity", 1, 2000, "it pops in once");
            tryCompare(shelf, "y", 0, 2000);
            verify(!chip.visible, "the items take the chips' place");
            compare(bar.height, high, "no new row");
            verify(menu.lit, "the button shows the menu is open");
            for (const item of shown) {
                let top = item;
                while (top.parent)
                    top = top.parent;
                compare(top, w.contentItem, "inside the keyboard window, no pop-up window");
                const mid = item.mapToItem(bar, 0, item.height / 2).y;
                verify(Math.abs(mid - bar.height / 2) < 1, "on the bar's row: " + mid);
                const x = item.mapToItem(bar, 0, 0).x;
                verify(x + item.width <= menu.mapToItem(bar, 0, 0).x, "left of the menu button");
            }
            mouseClick(menu);
            compare(ids(w.contentItem, root.items[0]).length, 0, "a second click closes it");
            verify(keyed(w.contentItem, "words.chip-0").visible, "the chips come back");
        }

        // Each item sends its ask, by the switches the lane carries, and closes the menu; Cancel only closes.
        function test_each_item_sends_its_ask_and_closes_the_menu() {
            const w = win(Lanes.some([0]));
            const cases = [
                [{}, 0, En.texts["words.forget"], "wordsAsk " + Words.asks.forget + " 0"],
                [{}, 1, En.texts["words.pause"], "wordsAsk " + Words.asks.pause + " 1"],
                [{ learn: false }, 1, En.texts["words.resume"], "wordsAsk " + Words.asks.pause + " 0"],
                [{}, 2, En.texts["words.off"], "wordsAsk " + Words.asks.power + " 0"],
                [{ on: false }, 2, En.texts["words.on"], "wordsAsk " + Words.asks.power + " 1"]
            ];
            asks(w);
            for (const [over, at, name, sent] of cases) {
                w.ui.lane = Lanes.some([0], over);
                const item = open(w)[at];
                compare(item.Accessible.name, name);
                mouseClick(item);
                compare(asks(w), [sent], name);
                compare(ids(w.contentItem, root.items[at]).length, 0, name + " closes the menu");
            }
            const cancel = open(w)[3];
            compare(cancel.Accessible.name, En.texts["words.cancel"]);
            mouseClick(cancel);
            compare(asks(w), [], "Cancel sends nothing");
            compare(ids(w.contentItem, root.items[3]).length, 0, "and closes the menu");
        }

        // The lock, off and empty lanes keep the menu, which still opens in place.
        function test_the_menu_opens_from_every_kind_of_lane() {
            for (const lane of [Words.lanes.locked, Words.lanes.off, Words.lanes.empty]) {
                const w = win(lane);
                compare(open(w).length, 4, lane.kind);
                compare(ids(w.contentItem, "words.locked").length, 0, lane.kind + ": the menu takes the note's place");
            }
        }

        // An Arabic keyboard reads the note and the menu in Arabic, in the Arabic font, right to left, and nothing on the
        // row mirrors (ADR-0020).
        function test_an_arabic_keyboard_reads_the_note_and_the_menu_right_to_left() {
            const w = win(Words.lanes.locked);
            w.ui.lang = Ar.lang;
            w.ui.texts = Ar.texts;
            waitForRendering(w.contentItem);
            const note = keyed(w.contentItem, "words.locked");
            const text = Board.all(note, o => o.font !== undefined)[0];
            compare([note.Accessible.name, text.text], [Ar.texts["chip.locked"], Ar.texts["chip.locked"]]);
            compare(text.font.family, w.res.fam.arabic);
            compare(text.effectiveHorizontalAlignment, Text.AlignRight, "the note reads right to left");
            const items = open(w);
            const keys = ["words.forget", "words.pause", "words.off", "words.cancel"];
            compare(items.map(i => i.Accessible.name), keys.map(k => Ar.texts[k]));
            for (const i of items)
                compare(Board.all(i, o => o.font !== undefined)[0].font.family, w.res.fam.arabic, i.Accessible.name);
            const xs = items.map(i => i.mapToItem(null, 0, 0).x);
            verify(xs.every((x, n) => n === 0 || x > xs[n - 1]), "the items keep their order from the left: " + xs);
            verify(keyed(w.contentItem, "words.menu").mapToItem(null, 0, 0).x > xs[3], "the menu button stays at the right");
        }

        // Where every text fits, each item shows its icon and text; else its icon alone, with its name as a tip.
        function test_items_show_icons_alone_with_tips_when_their_texts_do_not_fit() {
            const props = { box: Face.board.bar, lk: NL.lk, res: root.res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.lang.sizes, dark: false, lane: Lanes.some([0]), rtl: false };
            const b = createTemporaryObject(barC, root, props);
            const menu = keyed(b, "words.menu");
            mouseClick(menu);
            waitForRendering(b);
            const tight = root.items.map(id => keyed(b, id));
            const room = tight[0].parent.parent.width;
            const full = tight.reduce((w, i) => w + i.full, (tight.length - 1) * NL.lk.shape.chip.gap_px);
            verify(full > room, "the default bar has no room for every text: " + full + " > " + room);
            verify(tight.every(i => i.bare), "icons alone");
            const spy = createTemporaryObject(spyC, root, { target: b, signalName: "tip" });
            mouseMove(tight[0], tight[0].width / 2, tight[0].height / 2);
            tryCompare(spy, "count", 1, 1000);
            compare(spy.signalArguments[0][0], En.texts["words.forget"], "its name as a tip");
            mouseMove(b, 1, 1);
            b.box = Object.assign({}, Face.board.bar, { w: Face.board.bar.w + full });
            tryVerify(() => root.items.every(id => !keyed(b, id).bare), 1000, "texts where they all fit");
            const words = root.items.map(id => keyed(b, id)).map(i => Board.all(i, o => o.font !== undefined)[0]);
            verify(words.every(t => t.visible && t.text.length > 0), "each shows its text");
        }
    }
}

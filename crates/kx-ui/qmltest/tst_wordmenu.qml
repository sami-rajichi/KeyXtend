// The words menu on the top bar's row: its items take the chips' place in the keyboard window, with no new row or
// window, name Resume and On by the switches, send their asks to the hub and close; each shows its icon and short label,
// which drop from the row's end first when room is short, and names itself as a tip.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "lanes.js" as Lanes
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
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
    // Room left over a fit, less than any label, so float rounding never hides one.
    readonly property real slack: 0.5

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
        // Waits until `item`'s window has placed every item: a drawn frame can predate the last change, and rows place
        // their items only at the next polish.
        function laidOut(item) {
            verify(waitForPolish(item.Window.window), "laid out");
        }
        // Opens the menu in `w` and gives its items once they are laid out.
        function open(w) {
            mouseClick(keyed(w.contentItem, "words.menu"));
            laidOut(w.contentItem);
            return root.items.map(id => keyed(w.contentItem, id));
        }
        // A top bar in `res`'s language and box `box`, drawing `lane`, with its menu open and laid out.
        function menuBar(res, rtl, lane, box) {
            const props = { box: box, lk: NL.lk, res: res, s: 1, preset: 0, presets: Face.board.presets.length, sizes: En.lang.sizes, dark: false, lane: lane, rtl: rtl };
            const b = createTemporaryObject(barC, root, props);
            mouseClick(keyed(b, "words.menu"));
            laidOut(b);
            return b;
        }
        // An item's label, the box its items pop in on, and the label keys the switches in `over` give, in order.
        function label(item) {
            return Board.all(item, o => o.font !== undefined)[0];
        }
        function shelf(items) {
            return items[0].parent.parent;
        }
        function shorts(over) {
            const learn = over.learn !== false, on = over.on !== false;
            return ["words.forget-short", learn ? "words.pause-short" : "words.resume-short", on ? "words.off-short" : "words.on-short", "words.cancel"];
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
                compare(label(item).text, En.texts[shorts(over)[at]], "its short label");
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

        // At the default size each item shows its icon and short label in English, French and Arabic, by either switch.
        function test_every_label_shows_at_the_default_size_in_each_language() {
            for (const t of [En, Fr, Ar]) {
                for (const over of [{}, { learn: false, on: false }]) {
                    const b = menuBar(Board.resOf(NL.lk, t), !!t.lang.rtl, Lanes.some([0], over), Face.board.bar);
                    const items = root.items.map(id => keyed(b, id));
                    const used = items.reduce((w, i) => w + i.full, (items.length - 1) * NL.lk.shape.chip.gap_px);
                    const where = t.lang.tag + (over.on === false ? ", Resume and On" : "") + ": " + used.toFixed(1) + " of " + shelf(items).width.toFixed(1) + " px";
                    verify(items.every(i => !i.bare), where);
                    compare(items.map(i => label(i).text), shorts(over).map(k => t.texts[k]), where);
                    verify(items.every(i => label(i).visible), where);
                }
            }
        }

        // Short of room, labels drop one at a time from the row's end, Cancel's first; every item names itself as a tip.
        function test_labels_drop_from_the_end_first_and_each_item_names_itself_as_a_tip() {
            const b = menuBar(root.res, false, Lanes.some([0]), Face.board.bar);
            const items = root.items.map(id => keyed(b, id));
            const spy = createTemporaryObject(spyC, root, { target: b, signalName: "tip" });
            mouseMove(items[0], items[0].width / 2, items[0].height / 2);
            const tipped = name => spy.signalArguments.some(a => a[0] === name);
            tryVerify(() => tipped(En.texts["words.forget"]), 1000, "a labelled item's tip is its full name");
            mouseMove(b, 1, 1);
            const gap = NL.lk.shape.chip.gap_px;
            for (let k = items.length - 1; k >= 0; k--) {
                const need = items.reduce((w, i, n) => w + (n < k ? i.full : i.slim), (items.length - 1) * gap);
                const room = shelf(items).width;
                b.box = Object.assign({}, b.box, { w: b.box.w - room + need + root.slack });
                tryVerify(() => Math.abs(shelf(items).width - need - root.slack) < root.slack, 1000, "room for " + k);
                laidOut(b);
                compare(items.map(i => i.bare), items.map((i, n) => n >= k), k + " labels");
            }
            spy.clear();
            mouseMove(items[3], items[3].width / 2, items[3].height / 2);
            tryVerify(() => tipped(En.texts["words.cancel"]), 1000, "an icon alone names itself too");
        }
    }
}

// Every tap on the chip lane, its menu and the email list asks the hub for the key sound once, as a key does: a chip,
// menu or list tap through its own call, which sounds in the hub, and a tap QML handles alone through `Ui.tapped`.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "lanes.js" as Lanes
import "fixtures/words.js" as Words
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    // The lane's round, which each chip tap names.
    readonly property int round: Words.lanes.chips.round

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "wordtaps"
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
        // The one shown control with UI Automation id `id`.
        function keyed(item, id) {
            const hits = Board.all(item, o => o.visible && o.Accessible.id === id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        // The calls since the last look that make the hub sound: a chip, an ask, or a tap QML handled alone.
        function sounds(w) {
            const said = w.ui.calls.filter(c => c.startsWith("pickChip") || c.startsWith("wordsAsk") || c === "tapped");
            w.ui.calls = [];
            return said;
        }
        // Clicks control `id` in `w` and checks it made exactly `call`.
        function once(w, id, call) {
            sounds(w);
            mouseClick(keyed(w.contentItem, id));
            waitForRendering(w.contentItem);
            compare(sounds(w), [call], id);
        }

        function test_each_chip_kind_asks_for_the_sound_once() {
            const w = win(Lanes.some([0, 2, 3]));
            for (const at of [0, 1, 2])
                once(w, "words.chip-" + at, "pickChip " + root.round + " " + at);
            const more = win(Lanes.some([4, 5]));
            once(more, "words.chip-0", "pickChip " + root.round + " 0");
            once(more, "words.chip-1", "wordsAsk " + Words.asks.more + " 0");
        }

        function test_forget_modes_cancel_asks_for_the_sound_once() {
            once(win(Words.lanes.forget), "words.cancel", "wordsAsk " + Words.asks.cancel + " 0");
        }

        // The menu button opening and closing, and Cancel, which only closes, sound through `Ui.tapped`.
        function test_the_menu_button_and_each_item_ask_for_the_sound_once() {
            const w = win(Lanes.some([0]));
            once(w, "words.menu", "tapped");
            once(w, "words.menu", "tapped");
            const items = [["forget", Words.asks.forget + " 0"], ["pause", Words.asks.pause + " 1"], ["power", Words.asks.power + " 0"]];
            for (const [id, ask] of items) {
                once(w, "words.menu", "tapped");
                once(w, "words.menu-" + id, "wordsAsk " + ask);
            }
            once(w, "words.menu", "tapped");
            once(w, "words.menu-cancel", "tapped");
        }

        function test_the_email_lists_rows_pages_and_cancel_ask_for_the_sound_once() {
            const w = win(Words.lanes.list);
            once(w, "words.list-on", "wordsAsk " + Words.asks.page + " 1");
            once(w, "words.list-row-1", "wordsAsk " + Words.asks.pick + " 1");
            once(w, "words.list-cancel", "wordsAsk " + Words.asks.cancel + " 0");
        }

        function test_a_hover_asks_for_nothing() {
            const w = win(Lanes.some([0]));
            sounds(w);
            mouseMove(keyed(w.contentItem, "words.chip-0"));
            mouseMove(keyed(w.contentItem, "words.menu"));
            mouseMove(w.contentItem, 1, 1);
            compare(sounds(w), []);
        }
    }
}

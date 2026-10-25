// The email list over the keys, drawn from the lanes the hub writes: at most five rows, left to right in every
// language, paged, picked and closed through the hub's asks, inside the keyboard window, and the keys under it rest.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "lanes.js" as Lanes
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "fixtures/words.js" as Words
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    readonly property var page: Words.lanes.list.list

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "wordlist"
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
        // The hub's list lane showing page `over` instead of its own.
        function listing(over) {
            return Object.assign({}, Words.lanes.list, { list: Object.assign({}, root.page, over) });
        }
        // The shown controls under `item` with UI Automation id `id`, and the one control with it.
        function ids(item, id) {
            return Board.all(item, o => o.visible && o.Accessible.id === id);
        }
        function keyed(item, id) {
            const hits = ids(item, id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        function rows(w) {
            return Board.all(w.contentItem, o => o.visible && o.Accessible.role === Accessible.ListItem && o.Accessible.id.startsWith("words.list-row-"));
        }
        function asks(w) {
            const said = w.ui.calls.filter(c => c.startsWith("wordsAsk") || c.startsWith("pickChip"));
            w.ui.calls = [];
            return said;
        }

        function test_the_list_shows_one_page_of_five_rows_over_the_keys() {
            compare(En.settings.pageRows, 5, "the [ui] page_rows default");
            const w = win(Words.lanes.list);
            const list = keyed(w.contentItem, "words.list");
            compare(list.Accessible.name, En.texts["chip.more"]);
            compare(rows(w).map(r => r.Accessible.name), root.page.items);
            compare(rows(w).map(r => r.Accessible.id), [0, 1, 2, 3, 4].map(i => "words.list-row-" + i));
            const boxes = Face.board.keys.concat([Face.board.dpad]);
            const left = Math.min(...boxes.map(b => b.x)), top = Math.min(...boxes.map(b => b.y));
            const right = Math.max(...boxes.map(b => b.x + b.w)), bottom = Math.max(...boxes.map(b => b.y + b.h));
            const at = list.mapToItem(w.contentItem, 0, 0);
            compare([at.x, at.y, list.width, list.height], [left, top, right - left, bottom - top], "over the keys and the pad");
            let topmost = list;
            while (topmost.parent)
                topmost = topmost.parent;
            compare(topmost, w.contentItem, "inside the keyboard window");
            w.ui.lane = listing({ items: root.page.items.concat(["extra@example.org", "more@example.org"]) });
            compare(rows(w).length, 5, "never more than a page");
            w.ui.lane = Lanes.some([0]);
            compare(ids(w.contentItem, "words.list").length, 0, "it goes with the list");
        }

        // The pager pages through the hub, a row picks by its place on the page, and Cancel closes.
        function test_the_list_pages_picks_and_cancels_through_the_hub() {
            const w = win(Words.lanes.list);
            asks(w);
            const back = keyed(w.contentItem, "words.list-back");
            const on = keyed(w.contentItem, "words.list-on");
            const number = Board.all(w.contentItem, o => o.visible && o.Accessible.role === Accessible.StaticText && o.Accessible.name === w.ui.pageText(1, 2));
            compare(number.length, 1, "the page number");
            verify(!back.can && on.can, "no page back from the first");
            mouseClick(back);
            mouseClick(on);
            compare(asks(w), ["wordsAsk " + Words.asks.page + " 1"]);
            w.ui.lane = listing({ items: root.page.items.slice(0, 2), page: 1 });
            waitForRendering(w.contentItem);
            compare(rows(w).length, 2, "the last page");
            mouseClick(keyed(w.contentItem, "words.list-back"));
            mouseClick(rows(w)[1]);
            mouseClick(keyed(w.contentItem, "words.list-cancel"));
            const sent = ["wordsAsk " + Words.asks.page + " -1", "wordsAsk " + Words.asks.pick + " 1", "wordsAsk " + Words.asks.cancel + " 0"];
            compare(asks(w), sent);
            compare(keyed(w.contentItem, "words.list-cancel").Accessible.name, En.texts["words.cancel"]);
        }

        function test_one_page_has_no_pager() {
            const w = win(listing({ items: root.page.items.slice(0, 3), pages: 1 }));
            compare(rows(w).length, 3);
            compare(ids(w.contentItem, "words.list-back").length, 0, "no pager");
            keyed(w.contentItem, "words.list-cancel");
        }

        // Emails read left to right in an Arabic keyboard too, and nothing mirrors (ADR-0020).
        function test_rows_read_left_to_right_in_every_language() {
            const w = win(Words.lanes.list);
            w.ui.lang = Ar.lang;
            w.ui.texts = Ar.texts;
            waitForRendering(w.contentItem);
            compare(keyed(w.contentItem, "words.list-cancel").Accessible.name, Ar.texts["words.cancel"]);
            const row = rows(w)[0];
            const text = Board.all(row, o => o.font !== undefined && o.text === root.page.items[0])[0];
            compare(text.effectiveHorizontalAlignment, Text.AlignLeft);
            const cancel = keyed(w.contentItem, "words.list-cancel");
            verify(cancel.mapToItem(null, 0, 0).x > row.mapToItem(null, 0, 0).x, "the buttons stay on the right");
        }

        // A click on the list reaches no key under it.
        function test_the_keys_under_the_list_rest() {
            const w = win(Words.lanes.list);
            const list = keyed(w.contentItem, "words.list");
            mouseClick(list, list.width / 2, 2);
            const keys = w.keyboard.calls.filter(c => c.startsWith("tap") || c.startsWith("press") || c.startsWith("pad"));
            compare(keys, [], "no key heard it");
        }
    }
}

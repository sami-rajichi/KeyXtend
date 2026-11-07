// The clipboard panel's aim and its search line's caret (ADR-0040): a press anywhere asks the aim once at the press, a
// reader's press asks it too, and the caret stands where the view says while a click sends its place.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as NL

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: boxC; ClipBox {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "clip_aim"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in English, laid out.
        function open(name) {
            const props = { ui: createTemporaryObject(uiC, root), lk: NL.lk, res: Board.resOf(NL.lk, En), s: 1, view: Kit.view("en", name), rows: En.settings.pageRows, rtl: En.lang.rtl };
            const box = createTemporaryObject(boxC, root, props);
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        function acts(box) {
            return box.ui.calls.filter(c => c.startsWith("clipAct"));
        }
        function button(box, id) {
            const b = one(box.contentItem, id);
            verify(b, id + " shows once");
            return b;
        }

        // A press anywhere on the panel asks the aim, once and at the press, whatever lies under it.
        function test_a_press_anywhere_asks_the_aim_at_the_press() {
            const box = open("list");
            Board.apart(box.contentItem);
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "aiming" });
            const row = Board.all(box.contentItem, o => o.visible && o.place !== undefined && o.hi !== undefined)[0];
            const parts = [[button(box, "clip.search"), "the search"], [row, "a row"], [button(box, "clip.page-on"), "a page button"], [button(box, "clip.pin"), "a button"]];
            let asked = 0;
            for (const [part, what] of parts) {
                mousePress(part);
                compare(spy.count, ++asked, what + " at the press");
                mouseRelease(part);
                compare(spy.count, asked, what + " once");
            }
            tryVerify(() => box.body.scale === 1, 2000, "popped in, so the panel reaches its corner");
            mousePress(box.contentItem, 1, box.height - 1);
            compare(spy.count, ++asked, "blank space");
            mouseRelease(box.contentItem, 1, box.height - 1);
            compare(spy.count, asked, "blank space once");
            box.visible = false;
        }

        // While a question shows, the press still counts for the keys, though only Yes or No answers it.
        function test_a_press_on_a_question_asks_the_aim_and_answers_nothing() {
            const box = open("confirm");
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "aiming" });
            mouseClick(Kit.texts(box.contentItem, box.view.confirm.text)[0]);
            compare(spy.count, 1);
            compare(acts(box), []);
            box.visible = false;
        }

        // A reader's press has no pointer, so the line itself asks for the aim.
        function test_a_readers_press_on_the_search_asks_the_aim_once() {
            const box = open("list");
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "aiming" });
            button(box, "clip.search").Accessible.pressAction();
            compare(spy.count, 1);
            compare(acts(box), [], "it sends the panel nothing");
            box.visible = false;
        }

        // The view holds the search's caret and QML only draws it; a click sends its place, and the editor takes no focus.
        function test_the_searchs_caret_stands_where_the_view_says_and_a_click_places_it() {
            const box = open("list");
            box.aimed = true;
            const search = button(box, "clip.search");
            const said = at => {
                const v = Kit.view("en", "list");
                v.search = { text: "WWWWWW", hint: v.search.hint, arabic: false, rtl: false, caret: at };
                box.view = v;
            };
            const place = () => Board.all(search, o => o.visible && o.caret === true)[0].mapToItem(search, 0, 0).x;
            said(2);
            const low = place();
            said(4);
            verify(place() > low, "it moves as the view says");
            const ed = Board.all(search, o => o.cursorPosition !== undefined)[0];
            const at = ed.mapToItem(search, ed.positionToRectangle(1).x, 0);
            mouseClick(search, at.x + 1, search.height / 2);
            mouseClick(search, search.width - 2, search.height / 2);
            compare(acts(box), ["clipAct caret 1", "clipAct caret 6"], "on a letter, then past the end");
            compare(box.ui.clipRounds, [box.view.round, box.view.round], "with the round drawn");
            compare(ed.activeFocus, false, "focus stays with the app");
            box.visible = false;
        }
    }
}

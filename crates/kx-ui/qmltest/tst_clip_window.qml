// The clipboard panel in the keyboard window: the hub's wake opens it above the keyboard with its shadow, the scroll pad
// pages it, it never takes focus, a press on it counts as the keyboard's, and a closed view takes it away.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Kit
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "clip_window"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // A shown keyboard window over stand-in bridges, the real pointer passing through it.
        function make() {
            const ui = createTemporaryObject(uiC, root);
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: createTemporaryObject(kbC, root) });
            Pointer.through([win]);
            win.visible = true;
            waitForRendering(win.contentItem);
            return win;
        }

        function test_the_wake_opens_the_panel_above_the_keyboard_and_a_closed_view_ends_it() {
            const win = make();
            compare(win.clips, null, "closed until the hub has a view");
            win.ui.unpaged = 1;
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible && win.clips.visible, 2000, "the panel and its shadow show");
            const pane = win.clips.pane;
            Pointer.through([pane]);
            verify(win.ui.calls.indexOf("popAt over centred") >= 0, "above the keyboard");
            const marks = () => win.ui.calls.filter(c => c === "markPages " + pane.title).length;
            tryVerify(() => marks() === 2 && pane.paged, 2000, "the guard asks again until the scroll pad pages it");
            verify(pane.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            const spy = createTemporaryObject(spyC, root, { target: win, signalName: "touched" });
            verify(waitForPolish(pane));
            mouseClick(one(pane.contentItem, "clip.close"));
            compare(spy.count, 1, "a press on it is the keyboard's");
            compare(win.ui.calls.filter(c => c.startsWith("clipAct")), ["clipAct close 0"]);
            win.ui.clipView = null;
            tryVerify(() => win.clips === null, 2000, "a closed view takes it away");
            win.visible = false;
        }

        function test_a_new_view_redraws_the_open_panel_in_place() {
            const win = make();
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000);
            const pane = win.clips.pane;
            win.ui.clipView = Kit.view("en", "confirm");
            compare(win.clips.pane, pane, "the same window");
            compare(Kit.texts(pane.contentItem, pane.view.confirm.text).length, 1, "the new view shows");
            win.visible = false;
        }
    }
}

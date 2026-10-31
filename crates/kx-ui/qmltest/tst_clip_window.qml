// The clipboard panel in the keyboard window: the hub's wake opens it above the keyboard with its shadow, the scroll pad
// pages it, it shrinks to fit a small screen, it never takes focus, a press on it counts as the keyboard's, and a closed
// view takes it away.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Kit
import "pointer.js" as Pointer
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-dolch-light.js" as DL

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
        // A shown keyboard window over stand-in bridges in `look` (Native by default), the real pointer passing through it.
        function make(look) {
            const ui = createTemporaryObject(uiC, root, { look: (look || NL).lk });
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
            verify(win.ui.calls.indexOf("popFit over centred") >= 0, "above the keyboard");
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

        // Ruling 53: on a 1366×768 screen at 100 % the panel fits above the face fixture's keyboard at the least fit or
        // more, with and without a note, in each look and language, so its bottom stays above the keys.
        function test_the_panel_fits_above_the_keyboard_on_a_small_screen() {
            const work = 768 - 48; // The screen less Windows' taskbar.
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                const win = make(look);
                const pn = win.lk.shape.panel;
                const room = work - 2 * pn.gap_px[1] - pn.gap_px[0] - win.drawn.height;
                for (const tag of ["en", "fr", "ar"]) {
                    for (const name of ["list", "confirm"])
                        fitsAbove(win, Kit.view(tag, name), room, look.lk.look.id + " " + tag + " " + name);
                }
                win.visible = false;
            }
        }
        // The panel showing `view` in `win` shrinks into `room` units as the hub would shrink it, named `where` in a
        // failure: its full size needs no less than the least fit, and its bottom then stays above the keys.
        function fitsAbove(win, view, room, where) {
            const pn = win.lk.shape.panel;
            win.ui.fitRoom = Infinity;
            win.ui.clipView = view;
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000, where + " opens");
            const pane = win.clips.pane;
            win.clips.place(true);
            verify(waitForPolish(pane));
            const full = pane.height;
            verify(full * pn.min_fit <= room, where + ": " + full + " tall needs more than " + room + " at the least fit");
            win.ui.fitRoom = room;
            win.clips.place(true);
            tryVerify(() => pane.height <= room + 1, 2000, where + ": " + pane.height + " tall at fit " + win.clips.fit);
            verify(win.clips.fit >= pn.min_fit, where + " never below the least fit");
            compare(pane.s, win.s * win.clips.fit, where + " is built at its fit");
            win.ui.clipView = null;
            tryVerify(() => win.clips === null, 2000, where + " closes");
        }
    }
}

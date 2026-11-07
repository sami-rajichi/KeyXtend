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

        // The keyboard alone knows the aim (ADR-0040): the state says it, and a click on the line asks for it.
        function test_the_state_draws_the_aim_and_a_click_on_the_line_asks_the_keyboard() {
            const win = make();
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000);
            const pane = win.clips.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            const search = one(pane.contentItem, "clip.search");
            const carets = () => Board.all(search, o => o.visible && o.caret === true).length;
            compare(carets(), 0, "the golden state is not aimed");
            win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: true });
            win.keyboard.reply = { board: false, state: true };
            mouseClick(search);
            compare(win.keyboard.calls.filter(c => c === "aimPanel").length, 1);
            compare(carets(), 1, "its answer redrew the state");
            win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: false });
            win.restate();
            compare(carets(), 0, "a click outside, at the next relabel");
            win.visible = false;
        }

        // A press on a row, a button or blank space asks the keyboard for the aim and redraws at the press, so the arrows
        // follow it; the press still acts at its let-go, and a click asks once (ADR-0040).
        function test_a_press_anywhere_on_the_panel_asks_the_keyboard_at_the_press() {
            const win = make();
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000);
            const pane = win.clips.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            Board.apart(pane.contentItem);
            const asked = () => win.keyboard.calls.filter(c => c === "aimPanel").length;
            const carets = () => Board.all(pane.contentItem, o => o.visible && o.caret === true).length;
            win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: true });
            win.keyboard.reply = { board: false, state: true };
            const row = Board.all(pane.contentItem, o => o.visible && o.place !== undefined && o.hi !== undefined)[0];
            mousePress(row);
            compare(asked(), 1, "a row, at the press");
            compare(carets(), 1, "its answer redrew the state at once");
            compare(win.ui.calls.filter(c => c.startsWith("clipAct")), [], "the tick waits for the let-go");
            mouseRelease(row);
            compare(asked(), 1, "and once");
            compare(win.ui.calls.filter(c => c.startsWith("clipAct")), ["clipAct tick 0"]);
            mouseClick(one(pane.contentItem, "clip.page-on"));
            compare(asked(), 2, "a button");
            tryVerify(() => pane.body.scale === 1, 2000, "popped in, so the panel reaches its corner");
            mouseClick(pane.contentItem, 1, pane.height - 1);
            compare(asked(), 3, "blank space");
            win.visible = false;
        }

        // An insert's later "take the aim" must win, so the aim is asked at the press and never after the act.
        function test_the_aim_comes_before_an_insert_so_the_insert_can_take_it() {
            const win = make();
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000);
            const pane = win.clips.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            const asked = () => win.keyboard.calls.filter(c => c === "aimPanel").length;
            const insert = one(pane.contentItem, "clip.insert");
            mousePress(insert);
            compare(asked(), 1, "asked at the press");
            compare(win.ui.calls.filter(c => c.startsWith("clipAct")), [], "before the insert");
            mouseRelease(insert);
            compare(win.ui.calls.filter(c => c.startsWith("clipAct")), ["clipAct insert 0"]);
            compare(asked(), 1, "and not asked again after it");
            win.visible = false;
        }

        // Shaky taps on Insert must not give the aim back: a press inside the wait asks nothing, and a later one does.
        function test_a_press_following_another_asks_no_aim() {
            const win = make();
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000);
            const pane = win.clips.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            Board.doubled(pane.contentItem);
            const asked = () => win.keyboard.calls.filter(c => c === "aimPanel").length;
            const carets = () => Board.all(pane.contentItem, o => o.visible && o.caret === true).length;
            const aim = on => win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: on });
            win.keyboard.reply = { board: false, state: true };
            aim(true);
            const insert = one(pane.contentItem, "clip.insert");
            mouseClick(insert);
            compare(asked(), 1, "the first press");
            compare(carets(), 1, "aimed");
            aim(false);
            win.restate();
            compare(carets(), 0, "the insert gave the aim back");
            mousePress(insert);
            compare(asked(), 1, "the second press asks nothing");
            compare(carets(), 0, "and the aim stays off");
            mouseRelease(insert);
            mouseClick(insert);
            compare(asked(), 1, "nor does a third");
            aim(true);
            Board.apart(pane.contentItem);
            mousePress(insert);
            compare(asked(), 2, "a later press asks again");
            compare(carets(), 1, "and takes the aim");
            mouseRelease(insert);
            win.visible = false;
        }

        // A smaller fit the panel keeps after a layout is placed where the hub puts a panel of that size, so it stays
        // centred on the keyboard.
        function test_a_kept_smaller_fit_takes_the_spot_for_its_own_size() {
            const win = make();
            win.ui.fitCentre = 600;
            win.ui.clipView = Kit.view("en", "list");
            tryVerify(() => !!win.clips && win.clips.pane.visible, 2000, "the panel shows");
            const panel = win.clips, pane = panel.pane;
            verify(waitForPolish(pane));
            const full = { w: pane.pn.width_px * win.s, h: pane.unit * win.s };
            win.ui.fitRoom = full.h * 0.9;
            panel.place(true);
            const kept = panel.fit;
            verify(kept < 1, "shrunk to the room");
            win.ui.fitRoom = Infinity;
            panel.place(false);
            compare(panel.fit, kept, "the smaller fit stays");
            const ask = win.ui.fitAsks[win.ui.fitAsks.length - 1];
            fuzzyCompare(ask.w, full.w * kept, 0.01, "the spot is asked at the kept fit");
            const want = win.spots.logical(win.ui.fitCentre - ask.w / 2, win.ui.popSpot.y);
            fuzzyCompare(pane.x, want.x, 1, "centred at that size");
            win.visible = false;
        }

        // On a 1366×768 screen at 100 % the panel fits above the face fixture's keyboard at the least fit or more
        // (spec §3.4), with and without a note, in each look and language, so its bottom stays above the keys.
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

// Quick-fill in the keyboard window: the QuickFill object's wake opens the panel above the keyboard with its shadow, the
// scroll pad pages it, it never takes focus, a press on it counts as the keyboard's, a new view redraws it in place, it
// fits above the keyboard on a small screen, and a closed view takes it away.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
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
    Component { id: fillC; FakeFill {} }
    Component { id: winC; KbWindow {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "quickfill_window"
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
            const props = { ui: ui, keyboard: createTemporaryObject(kbC, root), fill: createTemporaryObject(fillC, root) };
            const win = createTemporaryObject(winC, root, props);
            Pointer.through([win]);
            win.visible = true;
            waitForRendering(win.contentItem);
            return win;
        }

        function test_the_wake_opens_the_panel_above_the_keyboard_and_a_closed_view_ends_it() {
            const win = make();
            compare(win.fills, null, "closed until the object has a view");
            win.ui.unpaged = 1;
            win.fill.shown = Kit.view("en", "list");
            tryVerify(() => !!win.fills && win.fills.pane.visible && win.fills.visible, 2000, "the panel and its shadow show");
            const pane = win.fills.pane;
            Pointer.through([pane]);
            verify(win.ui.calls.indexOf("popFit over centred") >= 0, "above the keyboard");
            const marks = () => win.ui.calls.filter(c => c === "markPages " + pane.title).length;
            tryVerify(() => marks() === 2 && pane.paged, 2000, "the guard asks again until the scroll pad pages it");
            verify(pane.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            const spy = createTemporaryObject(spyC, root, { target: win, signalName: "touched" });
            verify(waitForPolish(pane));
            mouseClick(one(pane.contentItem, "fill.close"));
            compare(spy.count, 1, "a press on it is the keyboard's");
            compare(win.fill.calls, ["button close 0"]);
            win.fill.shown = null;
            tryVerify(() => win.fills === null, 2000, "a closed view takes it away");
            win.visible = false;
        }

        function test_a_new_view_redraws_the_open_panel_in_place() {
            const win = make();
            win.fill.shown = Kit.view("en", "list");
            tryVerify(() => !!win.fills && win.fills.pane.visible, 2000);
            const pane = win.fills.pane;
            win.fill.shown = Kit.view("en", "entry");
            compare(win.fills.pane, pane, "the same window");
            tryVerify(() => Kit.texts(pane.contentItem, "s3cret-Pass!").length === 0 && !!one(pane.contentItem, "fill.edit"), 2000, "the new view shows");
            win.visible = false;
        }

        // The keyboard alone knows the aim (ADR-0040): the state says it, and a click on the line asks for it.
        function test_the_state_draws_the_aim_and_a_click_on_the_line_asks_the_keyboard() {
            const win = make();
            win.fill.shown = Kit.view("en", "list");
            tryVerify(() => !!win.fills && win.fills.pane.visible, 2000);
            const pane = win.fills.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            const search = one(pane.contentItem, "fill.search");
            const carets = () => Board.all(search, o => o.visible && o.caret === true).length;
            compare(carets(), 0, "the golden state is not aimed");
            win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: true });
            win.keyboard.reply = { board: false, state: true };
            mouseClick(search);
            compare(win.keyboard.calls.filter(c => c === "aimPanel").length, 1);
            compare(carets(), 1, "its answer redrew the state");
            win.visible = false;
        }

        // A press on a row, a button or blank space asks the keyboard for the aim and redraws at the press, so the arrows
        // follow it; the press still acts at its let-go, and a click asks once (ADR-0040).
        function test_a_press_anywhere_on_the_panel_asks_the_keyboard_at_the_press() {
            const win = make();
            win.fill.shown = Kit.view("en", "list");
            tryVerify(() => !!win.fills && win.fills.pane.visible, 2000);
            const pane = win.fills.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            Board.apart(pane.contentItem);
            const asked = () => win.keyboard.calls.filter(c => c === "aimPanel").length;
            const carets = () => Board.all(pane.contentItem, o => o.visible && o.caret === true).length;
            win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: true });
            win.keyboard.reply = { board: false, state: true };
            const row = Kit.filled(pane.contentItem)[0];
            mouseClick(row, 4, row.height / 2);
            compare(asked(), 1, "a row");
            compare(carets(), 1, "its answer redrew the state");
            compare(win.fill.calls, [], "a row itself still does nothing");
            mouseClick(one(pane.contentItem, "fill.page-on"));
            compare(asked(), 2, "a button");
            tryVerify(() => pane.body.scale === 1, 2000, "popped in, so the panel reaches its corner");
            mouseClick(pane.contentItem, 1, pane.height - 1);
            compare(asked(), 3, "blank space");
            win.visible = false;
        }

        // A fill's later "take the aim" must win, so the aim is asked at the press and never after the fill.
        function test_the_aim_comes_before_a_fill_so_the_fill_can_take_it() {
            const win = make();
            win.fill.shown = Kit.view("en", "list");
            tryVerify(() => !!win.fills && win.fills.pane.visible, 2000);
            const pane = win.fills.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            const asked = () => win.keyboard.calls.filter(c => c === "aimPanel").length;
            const fill = one(pane.contentItem, "fill.row-0-both");
            mousePress(fill);
            compare(asked(), 1, "asked at the press");
            compare(win.fill.calls, [], "before the fill");
            mouseRelease(fill);
            compare(win.fill.calls, ["row both 0"]);
            compare(asked(), 1, "and not asked again after it");
            win.visible = false;
        }

        // Shaky taps on a fill must not give the aim back: a press inside the wait asks nothing, and a later one does.
        function test_a_press_following_another_asks_no_aim() {
            const win = make();
            win.fill.shown = Kit.view("en", "list");
            tryVerify(() => !!win.fills && win.fills.pane.visible, 2000);
            const pane = win.fills.pane;
            Pointer.through([pane]);
            verify(waitForPolish(pane));
            Board.doubled(pane.contentItem);
            const asked = () => win.keyboard.calls.filter(c => c === "aimPanel").length;
            const carets = () => Board.all(pane.contentItem, o => o.visible && o.caret === true).length;
            const aim = on => win.keyboard.state = Object.assign({}, win.keyboard.state, { aimed: on });
            win.keyboard.reply = { board: false, state: true };
            aim(true);
            const fill = one(pane.contentItem, "fill.row-0-both");
            mouseClick(fill);
            compare(asked(), 1, "the first press");
            compare(carets(), 1, "aimed");
            aim(false);
            win.restate();
            compare(carets(), 0, "the fill gave the aim back");
            mousePress(fill);
            compare(asked(), 1, "the second press asks nothing");
            compare(carets(), 0, "and the aim stays off");
            mouseRelease(fill);
            mouseClick(fill);
            compare(asked(), 1, "nor does a third");
            aim(true);
            Board.apart(pane.contentItem);
            mousePress(fill);
            compare(asked(), 2, "a later press asks again");
            compare(carets(), 1, "and takes the aim");
            mouseRelease(fill);
            win.visible = false;
        }

        // On a 1366×768 screen at 100 % the panel fits above the face fixture's keyboard at the least fit or more
        // (spec §3.4), on its list and its tallest pages, in each look and language.
        function test_the_panel_fits_above_the_keyboard_on_a_small_screen() {
            const work = 768 - 48; // The screen less Windows' taskbar.
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                const win = make(look);
                const pn = win.lk.shape.panel;
                const room = work - 2 * pn.gap_px[1] - pn.gap_px[0] - win.drawn.height;
                for (const tag of ["en", "fr", "ar"]) {
                    for (const name of ["list", "edit", "noted"])
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
            win.fill.shown = view;
            tryVerify(() => !!win.fills && win.fills.pane.visible, 2000, where + " opens");
            const pane = win.fills.pane;
            win.fills.place(true);
            verify(waitForPolish(pane));
            const full = pane.height;
            verify(full * pn.min_fit <= room, where + ": " + full + " tall needs more than " + room + " at the least fit");
            win.ui.fitRoom = room;
            win.fills.place(true);
            tryVerify(() => pane.height <= room + 1, 2000, where + ": " + pane.height + " tall at fit " + win.fills.fit);
            verify(win.fills.fit >= pn.min_fit, where + " never below the least fit");
            win.fill.shown = null;
            tryVerify(() => win.fills === null, 2000, where + " closes");
        }
    }
}

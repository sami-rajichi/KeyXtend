// The keyboard window's top bar and bubble over stand-in bridges: what their buttons send, one size button per
// preset, and the ids UI Automation finds them by, as the gates do.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: spyC; SignalSpy {} }
    Component { id: bubC; Bubble {} }

    TestCase {
        name: "bar"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A keyboard window over fresh stand-ins, laid out and not shown; the real pointer passes through it.
        function make() {
            const ui = createTemporaryObject(uiC, root);
            const kb = createTemporaryObject(kbC, root);
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            verify(win, "the window is made");
            Pointer.through([win]);
            return win;
        }
        // Every item under `item` for which `ok` holds.
        function all(item, ok, found) {
            found = found || [];
            if (ok(item))
                found.push(item);
            for (let i = 0; i < item.children.length; i++)
                all(item.children[i], ok, found);
            return found;
        }
        // The one control UI Automation finds by `id`.
        function keyed(win, id) {
            const hits = all(win.contentItem, o => o.Accessible.id === id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        // The one button a screen reader calls `name`, for the controls that carry no id.
        function named(win, name) {
            const hits = all(win.contentItem, o => o.Accessible.role === Accessible.Button && o.Accessible.name === name);
            compare(hits.length, 1, name);
            return hits[0];
        }
        // The top bar's size buttons, in order.
        function sizeButtons(win) {
            return all(win.contentItem, o => o.part !== undefined && o.text.length > 0);
        }
        // A new bubble beside `win`, sharing its look and texts.
        function bubble(win) {
            return createTemporaryObject(bubC, root, { lk: win.lk, res: win.res, tipMs: win.cfg.tipMs, lift: t => true });
        }

        function test_top_bar_buttons_reach_the_keyboard_and_the_look() {
            const win = make();
            win.visible = true;
            waitForRendering(win.contentItem);
            const spy = createTemporaryObject(spyC, root, { target: win, signalName: "minimise" });
            mouseClick(keyed(win, "bar.smaller"));
            mouseClick(named(win, En.texts["sizename.l"]));
            for (const part of ["bigger", "mode", "theme", "minimise"])
                mouseClick(keyed(win, "bar." + part));
            const sent = win.keyboard.calls.filter(c => c === "smaller" || c === "bigger" || c.startsWith("preset"));
            compare(sent, ["smaller", "preset 2", "bigger"]);
            verify(win.ui.calls.indexOf("flipMode") >= 0, "the mode flips");
            verify(win.ui.calls.indexOf("stepTheme") >= 0, "the theme steps");
            compare(spy.count, 1, "minimise");
            win.visible = false;
        }

        function test_one_size_button_per_preset_and_none_lit_off_preset() {
            const win = make();
            compare(sizeButtons(win).map(b => b.text), En.lang.sizes.map(z => z.label));
            compare(sizeButtons(win).filter(b => b.lit).map(b => b.Accessible.name), [En.texts["sizename.s"]]);
            win.visible = true;
            waitForRendering(win.contentItem);
            // A smaller step lands between presets, and settings may hold fewer presets than the size texts name.
            win.keyboard.board = Object.assign({}, Face.board, { preset: null, presets: Face.board.presets.slice(0, 2) });
            win.keyboard.reply = { board: true, state: false };
            mouseClick(keyed(win, "bar.smaller"));
            compare(win.board.preset, null);
            const now = sizeButtons(win);
            compare(now.map(b => b.Accessible.name), [En.texts["sizename.s"], En.texts["sizename.m"]], "one per preset");
            compare(now.filter(b => b.lit).length, 0, "none lit off preset");
            win.visible = false;
        }

        function test_the_bubble_and_the_move_catcher_carry_their_texts() {
            const win = make();
            const bub = bubble(win);
            compare(bub.title, En.texts["window.bubble"]);
            compare(bub.name, En.texts["bar.bubble"]);
            verify(named(win, En.texts["catch"]), "the move catcher");
        }

        // The gates find the top bar's buttons and the bubble by these ids, whatever the language.
        function test_top_bar_buttons_and_the_bubble_give_ui_automation_their_bar_ids() {
            const win = make();
            for (const part of ["grip", "theme", "smaller", "bigger", "minimise", "close"])
                compare(keyed(win, "bar." + part).Accessible.name, En.texts["bar." + part]);
            const disc = all(bubble(win).contentItem, o => o.Accessible.role === Accessible.Button)[0];
            compare(disc.Accessible.id, "bar.bubble");
        }
    }
}

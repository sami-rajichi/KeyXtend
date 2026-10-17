// The keyboard window's top bar, scroll pad and bubble over stand-in bridges: what their buttons send, one size button
// per preset, and the ids UI Automation finds them by, as the gates do.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light-contrast.js" as H
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
        // A keyboard window over fresh stand-ins, its keyboard made with `props`, laid out and not shown; the real
        // pointer passes through it.
        function make(props) {
            const ui = createTemporaryObject(uiC, root);
            const kb = createTemporaryObject(kbC, root, props || {});
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
            mouseClick(keyed(win, "size.l"));
            for (const part of ["bigger", "mode", "theme", "minimise"])
                mouseClick(keyed(win, "bar." + part));
            const sent = win.keyboard.calls.filter(c => c === "smaller" || c === "bigger" || c.startsWith("preset"));
            compare(sent, ["smaller", "preset 2", "bigger"]);
            verify(win.ui.calls.indexOf("flipMode") >= 0, "the mode flips");
            verify(win.ui.calls.indexOf("stepTheme") >= 0, "the theme steps");
            compare(spy.count, 1, "minimise");
            win.visible = false;
        }

        // The scroll pad's taps and the pointer on it reach the keyboard: an arrow by its id, Stop as empty.
        function test_the_scroll_pad_sends_its_taps_and_the_pointer_on_it() {
            const dpad = { dim: false, tip: null, dir: "up", fast: false };
            const win = make({ state: Object.assign({}, Face.states.en, { dpad: dpad }) });
            win.visible = true;
            waitForRendering(win.contentItem);
            const pad = keyed(win, Face.board.dpad.name), stop = keyed(win, "dpad.stop");
            const hit = item => all(item, o => o.containsMouse !== undefined)[0];
            const up = all(pad, o => o.fill !== undefined && o.modelData !== undefined && o.modelData.id === "up")[0];
            const icon = all(up, o => o.icon !== undefined && o.tint !== undefined)[0];
            const at = icon.mapToItem(hit(up), icon.width / 2, icon.height / 2);
            const said = () => win.keyboard.calls.filter(c => c.startsWith("pad"));
            tryCompare(stop, "scale", 1, 2000, "Stop shows while the pad scrolls");
            mouseMove(hit(up), at.x, at.y);
            tryVerify(() => said().length === 1, 2000, "the pointer came on");
            mouseClick(hit(up), at.x, at.y);
            mouseClick(hit(stop));
            mouseMove(win.contentItem, 1, 1);
            tryVerify(() => said().length === 4, 2000, "the pointer left");
            compare(said(), ["padHover true", "pad up", "pad ", "padHover false"]);
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
            compare(keyed(win, "catch").Accessible.name, En.texts["catch"], "the catcher over the keyboard");
            const covers = all(win.contentItem, o => o.covers !== undefined)[0].covers;
            compare(covers.count, Qt.application.screens.length, "one per screen");
            const cover = covers.objectAt(0);
            compare(cover.title, En.texts["catch"], "UI Automation names the full-screen cover");
            const area = all(cover.contentItem, o => o.Accessible.role === Accessible.Button)[0];
            compare(area.Accessible.id, "catch");
            compare(area.Accessible.name, En.texts["catch"], "a screen reader names the cover's button");
        }

        // Every other control's id is its text key too: the size buttons, the resize corner, the D-pad.
        function test_size_buttons_the_corner_and_the_dpad_give_ui_automation_their_text_keys() {
            const win = make();
            for (const z of En.lang.sizes)
                compare(keyed(win, "size." + z.id).Accessible.name, z.name);
            compare(keyed(win, "bar.corner").Accessible.name, En.texts["bar.corner"]);
            compare(keyed(win, Face.board.dpad.name).Accessible.name, En.texts[Face.board.dpad.name]);
        }

        // The unlit letters take the bar's weight token, the lit one its lit weight.
        function test_the_size_letters_take_their_weights_from_the_tokens() {
            const win = make();
            const letters = sizeButtons(win).map(b => all(b, o => o.font !== undefined && o.text === b.text)[0]);
            compare(letters.map(l => l.font.weight), sizeButtons(win).map(b => b.lit ? win.lk.shape.bar.lit_weight : win.lk.shape.bar.weight));
        }

        // The bubble's disc is the plate, a window, so high contrast draws its icon in WindowText, not ButtonText.
        function test_the_bubble_icon_takes_the_plate_text_colour_in_high_contrast() {
            const win = make();
            const tint = lk => {
                const b = createTemporaryObject(bubC, root, { lk: lk, res: win.res, tipMs: win.cfg.tipMs, lift: t => true });
                return all(b.contentItem, o => o.icon === lk.icons.bar.bubble && o.tint !== undefined)[0].tint;
            };
            verify(Qt.colorEqual(tint(win.lk), win.lk.palette.legend), "the legend colour");
            verify(Qt.colorEqual(tint(H.lk), H.lk.palette.legend_2_plate), "WindowText on Window");
        }

        // The gates find the top bar's buttons and the bubble by these ids, whatever the language.
        function test_top_bar_buttons_and_the_bubble_give_ui_automation_their_bar_ids() {
            const win = make();
            for (const part of ["grip", "theme", "smaller", "bigger", "minimise", "close"])
                compare(keyed(win, "bar." + part).Accessible.name, En.texts["bar." + part]);
            const disc = all(bubble(win).contentItem, o => o.Accessible.role === Accessible.Button)[0];
            compare(disc.Accessible.id, "bar.bubble");
            compare(disc.Accessible.name, En.texts["bar.bubble"], "a screen reader names the bubble");
        }
    }
}

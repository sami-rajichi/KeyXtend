// The keyboard window over stand-in bridges fed from the face's golden files, the hub's texts and the real look:
// what screen readers find and hear, what a click sends, and how the Arabic UI and a stopped keyboard draw.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "window"
        when: windowShown

        // A keyboard window over fresh stand-ins, laid out and not shown.
        function make(props) {
            const ui = createTemporaryObject(uiC, root, props || {});
            const kb = createTemporaryObject(kbC, root);
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            verify(win, "the window is made");
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
        // The controls a screen reader meets: buttons with a name.
        function buttons(win) {
            return all(win.contentItem, o => o.Accessible.role === Accessible.Button && o.Accessible.name.length > 0);
        }
        // Every control a screen reader calls `name`.
        function found(win, name) {
            return buttons(win).filter(o => o.Accessible.name === name);
        }
        // The one control a screen reader calls `name`.
        function named(win, name) {
            const hits = found(win, name);
            compare(hits.length, 1, name);
            return hits[0];
        }
        // What the texts call board key `k`, or its character from the English state.
        function nameOf(k) {
            return k.name ? En.texts[k.name] : Face.states.en.keys[k.id].name;
        }

        function test_keys_are_found_by_accessible_name() {
            const win = make();
            const names = Face.board.keys.map(nameOf);
            for (const n of names)
                compare(found(win, n).length, names.filter(m => m === n).length, n);
            verify(named(win, En.texts["key.esc"]), "a named key");
            verify(named(win, "q"), "a character key by its character");
            verify(named(win, En.texts["bar.grip"]), "a top-bar button");
            verify(named(win, En.texts["key.dpad"]), "the D-pad");
        }

        function test_only_latches_and_caps_are_checkable() {
            const win = make();
            const checkable = buttons(win).filter(o => o.Accessible.checkable).map(o => o.Accessible.name).sort();
            const latches = Face.board.keys.filter(k => k.checkable).map(nameOf).sort();
            compare(checkable, latches);
            verify(latches.indexOf(En.texts["key.caps"]) >= 0, "Caps Lock is one");
            for (const k of Face.board.keys.filter(k => k.side))
                verify(!named(win, nameOf(k)).Accessible.checkable, "side key " + k.id);
        }

        function test_dimmed_keys_announce_coming_in_a_later_version() {
            const win = make();
            const later = En.texts["tip.later"];
            for (const k of Face.board.keys)
                for (const hit of found(win, nameOf(k)))
                    compare(hit.Accessible.description, k.dim ? later : "", k.id);
            compare(named(win, En.texts["key.dpad"]).Accessible.description, later, "the D-pad");
        }

        function test_every_text_is_drawn_in_grayscale() {
            const win = make();
            const texts = all(win.contentItem, o => o.renderType !== undefined && o.font !== undefined);
            verify(texts.length > 50, "the legends: " + texts.length);
            for (const t of texts)
                compare(t.renderType, Text.CurveRendering, t.text);
        }

        function test_a_click_sends_the_key_and_the_back_part_of_the_language_key() {
            const win = make();
            win.visible = true;
            waitForRendering(win.contentItem);
            mouseClick(named(win, "q"));
            const lang = named(win, En.texts["key.lang"]);
            mouseClick(lang, lang.width * 0.1, lang.height / 2);
            const taps = win.keyboard.calls.filter(c => c.startsWith("tap"));
            compare(taps, ["tap c-0x0010 false", "tap lang true"]);
            mouseClick(named(win, En.texts["key.mic"]));
            compare(win.keyboard.calls.filter(c => c.startsWith("tap")).length, 2, "a dimmed key sends nothing");
            win.visible = false;
        }

        function test_the_arabic_ui_mirrors_the_top_bar_and_not_the_keys() {
            const en = make();
            const ar = make({ settings: Ar.settings, texts: Ar.texts });
            const grip = named(ar, Ar.texts["bar.grip"]);
            const close = named(ar, Ar.texts["bar.close"]);
            verify(grip.mapToItem(null, 0, 0).x > close.mapToItem(null, 0, 0).x, "grip on the right");
            const q = [named(en, "q"), named(ar, "q")].map(k => k.mapToItem(null, 0, 0).x);
            compare(q[1], q[0], "the keys stay where the board puts them");
            compare(ar.title, Ar.texts["window.keyboard"]);
        }

        function test_a_shown_window_takes_no_focus_and_is_cut_guarded_and_frosted() {
            const win = make({ frosted: true });
            verify(win.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            win.visible = true;
            tryVerify(() => win.frosted, 2000, "the backdrop is on");
            const calls = win.keyboard.calls;
            verify(calls.indexOf("shape " + win.title) >= 0 && calls.indexOf("guard") >= 0, calls.join());
            const share = win.lk.shape.backdrop.plate;
            // The plate is the one visible fill with the window's radius; the move ring beside it is hidden and clear.
            const plate = all(win.contentItem, o => o.radius === win.lk.look.window_radius_px && o.visible && o.color.a > 0)[0];
            tryVerify(() => Math.abs(plate.color.a - share) < 0.01, 2000, "the plate lets the backdrop through: " + plate.color.a);
            win.visible = false;
        }

        function test_top_bar_buttons_reach_the_keyboard_and_the_look() {
            const win = make();
            win.visible = true;
            waitForRendering(win.contentItem);
            const spy = createTemporaryObject(spyC, root, { target: win, signalName: "minimise" });
            mouseClick(named(win, En.texts["bar.smaller"]));
            mouseClick(named(win, En.texts["sizename.l"]));
            mouseClick(named(win, En.texts["bar.mode"]));
            mouseClick(named(win, En.texts["bar.minimise"]));
            const sent = win.keyboard.calls.filter(c => c === "smaller" || c.startsWith("preset"));
            compare(sent, ["smaller", "preset 2"]);
            verify(win.ui.calls.indexOf("flipMode") >= 0, "the mode flips");
            compare(spy.count, 1, "minimise");
            win.visible = false;
        }

        function test_a_stopped_keyboard_draws_nothing_until_it_runs() {
            const ui = createTemporaryObject(uiC, root);
            const kb = createTemporaryObject(kbC, root, { board: null, state: null });
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            verify(!win.running && buttons(win).length === 0, "no keys");
            kb.board = Face.board;
            kb.state = Face.states.en;
            ui.settingsChanged();
            verify(win.running, "a settings change reads the board again");
            verify(named(win, En.texts["key.esc"]));
        }
    }
}

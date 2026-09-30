// The keyboard window over stand-in bridges fed from the face's golden files, the hub's texts and the real look:
// what screen readers find and hear, what a click sends, how labels fit, and how the Arabic UI and a stopped keyboard draw.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D

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
        name: "window"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A keyboard window over fresh stand-ins, laid out and not shown.
        function make(props, kbProps) {
            const ui = createTemporaryObject(uiC, root, props || {});
            const kb = createTemporaryObject(kbC, root, kbProps || {});
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
        // The face's board at `size`, off preset: the module scales every box by the size (kx-mod-keyboard geom.rs).
        function scaled(size) {
            const box = b => Object.assign({}, b, { x: b.x * size, y: b.y * size, w: b.w * size, h: b.h * size });
            const b = Face.board;
            return Object.assign({}, b, { size: size, preset: null, plate: box(b.plate), bar: box(b.bar), dpad: box(b.dpad), keys: b.keys.map(box) });
        }
        // The top bar's size buttons, in order.
        function sizeButtons(win) {
            return all(win.contentItem, o => o.part !== undefined && o.text.length > 0);
        }
        // The window's tooltip.
        function tipOf(win) {
            for (let i = 0; i < win.data.length; i++)
                if (win.data[i].item && win.data[i].item.hint)
                    return win.data[i].item;
            return null;
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

        function test_a_press_on_the_keyboard_is_seen_and_still_reaches_the_key() {
            const win = make();
            const spy = createTemporaryObject(spyC, root, { target: win, signalName: "touched" });
            win.visible = true;
            waitForRendering(win.contentItem);
            mouseClick(named(win, "q"));
            compare(spy.count, 1, "seen, so the tray menu closes");
            compare(win.keyboard.calls.filter(c => c.startsWith("tap")), ["tap c-0x0010 false"]);
            mouseClick(named(win, En.texts["bar.theme"]));
            compare(spy.count, 2, "the top bar too");
            verify(win.ui.calls.indexOf("stepTheme") >= 0, "and its button still works");
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
            mouseClick(named(win, En.texts["bar.bigger"]));
            mouseClick(named(win, En.texts["bar.mode"]));
            mouseClick(named(win, En.texts["bar.theme"]));
            mouseClick(named(win, En.texts["bar.minimise"]));
            const sent = win.keyboard.calls.filter(c => c === "smaller" || c === "bigger" || c.startsWith("preset"));
            compare(sent, ["smaller", "preset 2", "bigger"]);
            verify(win.ui.calls.indexOf("flipMode") >= 0, "the mode flips");
            verify(win.ui.calls.indexOf("stepTheme") >= 0, "the theme steps");
            compare(spy.count, 1, "minimise");
            win.visible = false;
        }

        function test_one_size_button_per_preset_and_none_lit_off_preset() {
            const win = make();
            compare(sizeButtons(win).map(b => b.text), En.settings.sizes.map(z => z.label));
            compare(sizeButtons(win).filter(b => b.lit).map(b => b.Accessible.name), [En.texts["sizename.s"]]);
            win.visible = true;
            waitForRendering(win.contentItem);
            // A smaller step lands between presets, and settings may hold more presets than the size texts name.
            win.keyboard.board = Object.assign({}, Face.board, { preset: null, presets: Face.board.presets.concat([2]) });
            win.keyboard.reply = { board: true, state: false };
            mouseClick(named(win, En.texts["bar.smaller"]));
            compare(win.board.preset, null);
            const now = sizeButtons(win);
            compare(now.length, 4, "one per preset");
            compare(now.filter(b => b.lit).length, 0, "none lit off preset");
            compare([now[3].text, now[3].Accessible.name], ["4", "4"], "an unnamed preset shows its number");
            win.visible = false;
        }

        function test_every_label_fits_its_cap_in_each_theme_language_and_size() {
            for (const lk of [N.lk, E.lk, D.lk])
                for (const t of [En, Fr, Ar])
                    for (const size of [Face.sizeMin, 1])
                        labelsFit(lk, t, size);
        }
        // Each label in look `lk`, texts `t` and `size` stays inside its cap, above the smallest size; at size 1 English
        // and Arabic keep the P1 size.
        function labelsFit(lk, t, size) {
            const win = make({ settings: t.settings, texts: t.texts, look: lk }, { board: scaled(size) });
            const where = lk.look.latin_fonts[0] + " " + t.settings.lang + " " + size + ": ";
            tryCompare(win.res.fam, "latin", lk.look.latin_fonts[0], 5000, where + "the theme's font");
            // Rows lay out when drawn.
            win.visible = true;
            waitForRendering(win.contentItem);
            const pad = lk.shape.legend.label_pad_px * size;
            const keys = all(win.contentItem, o => o.kv !== undefined && o.later !== undefined && o.label.length > 0);
            verify(keys.length > 0, where + "labels");
            for (const k of keys) {
                const l = all(k, o => o.text === k.label && o.fontInfo !== undefined && o.visible)[0];
                const at = where + k.label;
                const left = l.mapToItem(k, (l.width - l.contentWidth) / 2, 0).x;
                verify(left >= pad - 0.01 && left + l.contentWidth <= k.width - pad + 0.01, at + " fits: " + l.contentWidth);
                verify(l.fontInfo.pixelSize >= lk.shape.legend.label_min_px, at + " not below the smallest size");
                if (size === 1 && t !== Fr)
                    compare(l.fontInfo.pixelSize, l.font.pixelSize, at + " keeps the P1 size");
            }
            win.visible = false;
        }

        function test_the_bubble_and_the_move_catcher_carry_their_texts() {
            const win = make();
            const bub = createTemporaryObject(bubC, root, { lk: win.lk, res: win.res });
            compare(bub.title, En.texts["window.bubble"]);
            compare(bub.name, En.texts["bar.bubble"]);
            verify(named(win, En.texts["catch"]), "the move catcher");
        }

        function test_a_dimmed_key_and_the_dpad_tip_say_their_name_then_when_they_come() {
            const win = make();
            win.visible = true;
            waitForRendering(win.contentItem);
            const tip = tipOf(win);
            for (const name of [En.texts["key.mic"], En.texts["key.dpad"]]) {
                const it = named(win, name);
                mouseMove(it, it.width / 2, it.height / 2);
                compare(tip.name, name + "\n" + En.texts["tip.later"]);
            }
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

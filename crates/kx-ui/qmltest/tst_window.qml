// The keyboard window over stand-in bridges fed from the face's golden files, the hub's texts and the real look:
// what screen readers find and hear, what a click sends, and how the Arabic UI and a stopped keyboard draw (labels
// fitting their caps: tst_fit; the top bar: tst_bar). Keys are found by their UI Automation id, as the gates do.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
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
        name: "window"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A keyboard window over fresh stand-ins, laid out and not shown; the real pointer passes through it.
        function make(props, kbProps) {
            const ui = createTemporaryObject(uiC, root, props || {});
            const kb = createTemporaryObject(kbC, root, kbProps || {});
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
        // The controls a screen reader meets: buttons with a name.
        function buttons(win) {
            return all(win.contentItem, o => o.Accessible.role === Accessible.Button && o.Accessible.name.length > 0);
        }
        // The one control UI Automation finds by `id`: a board key's id, or a top-bar button's.
        function keyed(win, id) {
            const hits = all(win.contentItem, o => o.Accessible.id === id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        // The one control a screen reader calls `name`, for the D-pad, which carries no id.
        function named(win, name) {
            const hits = buttons(win).filter(o => o.Accessible.name === name);
            compare(hits.length, 1, name);
            return hits[0];
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

        function test_each_key_is_found_by_its_id_and_carries_its_screen_reader_name() {
            const win = make();
            for (const k of Face.board.keys)
                compare(keyed(win, k.id).Accessible.name, nameOf(k), k.id);
            compare(keyed(win, "c-0x0010").Accessible.name, "q", "a character key by its character");
            compare(keyed(win, "bar.grip").Accessible.name, En.texts["bar.grip"], "a top-bar button");
            verify(named(win, En.texts["key.dpad"]), "the D-pad");
        }

        function test_only_latches_and_caps_are_checkable() {
            const win = make();
            const checkable = buttons(win).filter(o => o.Accessible.checkable).map(o => o.Accessible.id).sort();
            const latches = Face.board.keys.filter(k => k.checkable).map(k => k.id).sort();
            compare(checkable, latches);
            verify(latches.indexOf("caps") >= 0, "Caps Lock is one");
            for (const k of Face.board.keys.filter(k => k.side))
                verify(!keyed(win, k.id).Accessible.checkable, "side key " + k.id);
        }

        function test_dimmed_keys_announce_coming_in_a_later_version() {
            const win = make();
            const later = En.texts["tip.later"];
            for (const k of Face.board.keys)
                compare(keyed(win, k.id).Accessible.description, k.dim ? later : "", k.id);
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
            mouseClick(keyed(win, "c-0x0010"));
            const lang = keyed(win, "lang");
            mouseClick(lang, lang.width * 0.1, lang.height / 2);
            // A key that repeats while held sends its press and let-go instead of a tap.
            const taps = () => win.keyboard.calls.filter(c => /^(tap|press|letGo)\b/.test(c));
            compare(taps(), ["press c-0x0010", "letGo c-0x0010", "tap lang true"]);
            mouseClick(keyed(win, "mic"));
            compare(taps().length, 3, "a dimmed key sends nothing");
            win.visible = false;
        }

        function test_a_press_on_the_keyboard_is_seen_and_still_reaches_the_key() {
            const win = make();
            const spy = createTemporaryObject(spyC, root, { target: win, signalName: "touched" });
            win.visible = true;
            waitForRendering(win.contentItem);
            mouseClick(keyed(win, "c-0x0010"));
            compare(spy.count, 1, "seen, so the tray menu closes");
            compare(win.keyboard.calls.filter(c => c.startsWith("letGo")), ["letGo c-0x0010"]);
            mouseClick(keyed(win, "bar.theme"));
            compare(spy.count, 2, "the top bar too");
            verify(win.ui.calls.indexOf("stepTheme") >= 0, "and its button still works");
            win.visible = false;
        }

        // Arabic flips text only (ADR-0020): the top bar and the keys keep their places.
        function test_the_arabic_ui_keeps_the_top_bar_and_the_keys_in_place() {
            const en = make();
            const ar = make({ settings: Ar.settings, lang: Ar.lang, texts: Ar.texts });
            const at = (win, id) => keyed(win, id).mapToItem(null, 0, 0).x;
            verify(at(ar, "bar.grip") < at(ar, "bar.close"), "grip on the left");
            compare(at(ar, "bar.grip"), at(en, "bar.grip"), "the bar keeps its places");
            compare(at(ar, "c-0x0010"), at(en, "c-0x0010"), "the keys stay where the board puts them");
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

        function test_a_dimmed_key_and_the_dpad_tip_say_their_name_then_when_they_come() {
            const win = make();
            win.visible = true;
            waitForRendering(win.contentItem);
            const tip = tipOf(win);
            const said = [[keyed(win, "mic"), "key.mic"], [named(win, En.texts["key.dpad"]), "key.dpad"]];
            for (const [it, name] of said) {
                mouseMove(it, it.width / 2, it.height / 2);
                compare(tip.name, En.texts[name] + "\n" + En.texts["tip.later"]);
            }
            win.visible = false;
        }

        // With one layout installed the language key stays still and says so: readers hear it after the name, and the
        // Carousel, which has no icon and so no tip, shows it in one.
        function test_a_lone_layout_makes_the_language_key_say_so() {
            const st = Face.states.en;
            const lang = Object.assign({}, st.lang, { prev: null, next: null, at: 0, count: 1 });
            const keys = Object.assign({}, st.keys, { lang: Object.assign({}, st.keys.lang, { tip: "tip.one-lang" }) });
            const win = make({}, { state: Object.assign({}, st, { lang: lang, keys: keys }) });
            win.visible = true;
            waitForRendering(win.contentItem);
            const key = keyed(win, "lang");
            compare(key.Accessible.description, En.texts["tip.one-lang"]);
            mouseMove(key, key.width / 2, key.height / 2);
            compare(tipOf(win).name, En.texts["key.lang"] + "\n" + En.texts["tip.one-lang"]);
            win.visible = false;
        }

        // The labels follow the layout in front as soon as the keyboard shows again, not one relabel period later.
        function test_the_labels_follow_the_layout_at_once_each_time_the_keyboard_shows() {
            const wait = 2000;
            const win = make({}, { board: Object.assign({}, Face.board, { relabel_ms: 10 * wait }) });
            const relabels = () => win.keyboard.calls.filter(c => c === "relabel").length;
            for (const n of [1, 2]) {
                win.visible = true;
                tryVerify(() => relabels() === n, wait, "show " + n);
                win.visible = false;
            }
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
            verify(keyed(win, "esc"));
        }
    }
}

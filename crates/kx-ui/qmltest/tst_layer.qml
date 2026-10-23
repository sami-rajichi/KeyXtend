// The shortcuts layer in the keyboard window, over stand-in bridges fed from the face's golden files: the Shortcuts key
// lights while it shows, each cell shows its icon alone or an F-key its text, tips name the keys, the Next key's dots
// follow the page, the pad hides, new cells fade and rise in unless motion is reduced, and a dimmed Power key says why.
// Controls are found by their UI Automation id, as tst_window does.
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light-reduced.js" as Reduced
import "pointer.js" as Pointer
import "board.js" as Board

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }

    TestCase {
        name: "layer"
        when: windowShown

        // The window the test made last, hidden after it even when an assertion stops it.
        property Window made: null

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function cleanup() {
            if (made)
                made.visible = false;
            made = null;
        }
        // A shown keyboard window over fresh stand-ins on `page` ("main" for the usual side keys), whose keyboard
        // answers every click with a new board and state; the real pointer passes through it.
        function make(page, props) {
            const ui = createTemporaryObject(uiC, root, props || {});
            const answers = { reply: { board: true, state: true }, board: boardOf(page), state: stateOf(page) };
            const kb = createTemporaryObject(kbC, root, answers);
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            verify(win, "the window is made");
            Pointer.through([win]);
            made = win;
            // Read by a call from now on, as the real keyboard is, so a stand-in's change reaches no binding.
            turn(win, page);
            win.visible = true;
            waitForRendering(win.contentItem);
            return win;
        }
        // The face's board and state on `page`.
        function boardOf(page) {
            return page === "main" ? Face.board : Face.layers[page];
        }
        function stateOf(page) {
            return page === "main" ? Face.states.en : Face.states[page];
        }
        // The keyboard's board and state on `page`, which its next answer reads.
        function serve(win, page) {
            win.keyboard.board = boardOf(page);
            win.keyboard.state = stateOf(page);
        }
        // The window reads `page` at once, as after a click on the Shortcuts key or Next.
        function turn(win, page) {
            serve(win, page);
            win.redraw(JSON.stringify({ board: true, state: true }));
        }
        // The one control UI Automation finds by `id`, or none.
        function found(win, id) {
            return Board.all(win.contentItem, o => o.Accessible.id === id);
        }
        function keyed(win, id) {
            const hits = found(win, id);
            compare(hits.length, 1, id);
            return hits[0];
        }
        // The keys the window draws, in the order readers meet them.
        function keys(win) {
            return Board.all(win.contentItem, o => o.kv !== undefined && o.later !== undefined);
        }
        // The shortcut cells the board draws now.
        function cells(win) {
            return win.board.keys.filter(k => k.side && k.icon_only);
        }
        // What shows on key `key`: its visible texts and icons.
        function shown(key) {
            const visible = o => {
                for (let p = o; p && p !== key; p = p.parent)
                    if (!p.visible)
                        return false;
                return true;
            };
            const texts = Board.all(key, o => o.renderType !== undefined && o.text.length > 0 && visible(o)).map(o => o.text);
            const icons = Board.all(key, o => o.tint !== undefined && o.icon !== undefined && o.icon.length > 0 && visible(o)).map(o => o.icon);
            return { texts: texts, icons: icons };
        }
        // Clicks `key` and waits until its press frame lets it act.
        function click(win, key) {
            mouseClick(key);
            tryCompare(win.presses, "waiting", null);
        }
        // The page dots under the Next key, in order.
        function dots(win) {
            return Board.all(keyed(win, "layer-next"), o => o.lit !== undefined && o.radius !== undefined);
        }
        // The Scroll pad, by its arrows.
        function pad(win) {
            return Board.all(win.contentItem, o => o.arrows !== undefined)[0];
        }

        function test_the_shortcuts_key_is_lit_while_the_layer_shows() {
            const win = make("main");
            const key = keyed(win, "shortcuts");
            verify(!key.on && !key.Accessible.checked, "off with the usual side keys");
            serve(win, "layer-1");
            click(win, key);
            verify(win.keyboard.calls.indexOf("tap shortcuts false") >= 0, "the click reached the keyboard");
            const lit = keyed(win, "shortcuts");
            verify(lit.on && lit.Accessible.checked, "lit like a latched modifier");
            tryVerify(() => Qt.colorEqual(lit.fillColour, win.lk.palette.on), 2000, "in the on colour");
            compare(found(win, "rclick").length, 0, "the side keys it hides are gone");
            serve(win, "main");
            click(win, lit);
            verify(!keyed(win, "shortcuts").Accessible.checked, "a second click goes back");
        }

        function test_each_cell_shows_its_icon_alone_and_an_f_key_its_text() {
            const win = make("layer-1");
            const all = cells(win);
            compare(all.length, Face.layers["layer-1"].keys.filter(k => k.icon_only).length);
            for (const k of all) {
                const icon = win.lk.icons.shortcuts[k.icon.split(".")[1]];
                verify(icon, k.id + " has an icon in the look");
                compare(shown(keyed(win, k.id)), { texts: [], icons: [icon] }, k.id);
            }
            turn(win, "layer-3");
            const f5 = keyed(win, "sc-f5");
            compare(shown(f5), { texts: [En.texts["shortcut.f5"]], icons: [] }, "F5 shows F5");
            compare(f5.Accessible.name, En.texts["shortcut.f5"]);
        }

        function test_a_cells_tip_names_its_keys_and_an_f_keys_tip_its_name() {
            const win = make("layer-1");
            const tip = Board.tipOf(win);
            const kv = win.board.keys.find(k => k.id === "sc-copy-all");
            const key = keyed(win, kv.id);
            mouseMove(key, key.width / 2, key.height / 2);
            compare(tip.name, win.ui.keysTip(kv.name, JSON.stringify(kv.steps)), "the hub's tip");
            verify(tip.name.indexOf(En.texts["label.ctrl"]) > 0, "it names Ctrl: " + tip.name);
            verify(tip.name.startsWith(En.texts["shortcut.copy-all"]), "after the name: " + tip.name);
            compare(key.Accessible.name, En.texts["shortcut.copy-all"], "readers hear the name");
            turn(win, "layer-3");
            const f5 = keyed(win, "sc-f5");
            mouseMove(f5, f5.width / 2, f5.height / 2);
            compare(tip.name, En.texts["shortcut.f5"], "its name alone, not F5 (F5)");
        }

        function test_the_dots_light_the_page_and_next_wraps_them_back_to_the_first() {
            const win = make("layer-1");
            const sh = win.lk.shape.dots;
            const lit = () => dots(win).map(d => d.lit);
            compare(lit(), [true, false, false], "one dot per page, the first lit");
            compare(dots(win).map(d => d.width), [sh.on_px, sh.px, sh.px], "the lit one larger");
            compare(dots(win).map(d => d.opacity), [1, sh.off, sh.off]);
            const next = keyed(win, "layer-next");
            compare(next.Accessible.name, En.texts["key.layer-next"], "readers hear its name");
            compare(shown(next), { texts: [], icons: [win.lk.icons.layer.next] }, "a chevron and no numbers");
            // The chevron sits where every side icon does, and its dots start next_at of its box down.
            const glyph = k => Board.all(k, o => o.tint !== undefined && o.icon !== undefined && o.icon.length > 0)[0];
            const inKey = k => glyph(k).mapToItem(k, 0, 0);
            const [chevron, icon] = [inKey(next), inKey(keyed(win, "sc-copy"))];
            compare([chevron.x, chevron.y], [icon.x, icon.y], "where every side icon sits");
            const row = dots(win)[0].parent.parent.mapToItem(glyph(next), 0, 0);
            fuzzyCompare(row.y, glyph(next).height * sh.next_at, 0.01, "the dots from next_at of its box down");
            turn(win, "layer-3");
            compare(lit(), [false, false, true], "the last page");
            serve(win, "layer-1");
            click(win, keyed(win, "layer-next"));
            verify(win.keyboard.calls.indexOf("tap layer-next false") >= 0, "Next reached the keyboard");
            compare(lit(), [true, false, false], "back to the first page");
        }

        function test_the_dots_show_only_where_the_board_marks_them() {
            const win = make("layer-1");
            verify(dots(win).length > 0, "on the Next key, which the board marks");
            const page = Face.layers["layer-1"];
            const keys = page.keys.map(k => k.dots ? Object.assign({}, k, { dots: false }) : k);
            win.keyboard.board = Object.assign({}, page, { keys: keys });
            win.redraw(JSON.stringify({ board: true }));
            compare(dots(win).length, 0, "none on a key it does not mark");
            compare(shown(keyed(win, "layer-next")).icons, [win.lk.icons.layer.next], "its chevron alone");
        }

        function test_a_flip_makes_only_the_side_block_again_and_keeps_the_reading_order() {
            const win = make("main");
            const esc = keyed(win, "esc");
            for (const page of ["layer-1", "main"]) {
                turn(win, page);
                verify(keyed(win, "esc") === esc, "the main keys stay on " + page);
                compare(keys(win).map(k => k.Accessible.id), win.board.keys.map(k => k.id), "the board's order on " + page);
            }
        }

        function test_a_page_with_as_many_keys_turns_its_cells_from_icons_to_text_and_back() {
            const win = make("layer-1");
            // Page 1 less a cell has as many keys as page 3, so its keys stay and change their faces.
            const page = Face.layers["layer-1"];
            const like = Object.assign({}, page, { keys: page.keys.filter(k => k.id !== "sc-emoji") });
            compare(like.keys.length, Face.layers["layer-3"].keys.length);
            win.keyboard.board = like;
            win.redraw(JSON.stringify({ board: true }));
            const kept = keys(win);
            turn(win, "layer-3");
            verify(keys(win).every((k, i) => k === kept[i]), "the same keys");
            const f5 = keyed(win, "sc-f5");
            compare(shown(f5), { texts: [En.texts["shortcut.f5"]], icons: [] });
            const label = Board.all(f5, o => o.renderType !== undefined && o.text === En.texts["shortcut.f5"])[0];
            fuzzyCompare(label.mapToItem(f5, label.width / 2, 0).x, f5.width / 2, 0.5, "its label centred");
            win.keyboard.board = like;
            win.keyboard.state = Face.states["layer-1"];
            win.redraw(JSON.stringify({ board: true }));
            const copy = keyed(win, "sc-copy");
            compare(shown(copy), { texts: [], icons: [win.lk.icons.shortcuts.copy] }, "an icon again");
            compare(copy.Accessible.name, En.texts["shortcut.copy"]);
        }

        function test_the_pad_hides_while_the_layer_shows() {
            const win = make("main");
            verify(pad(win).visible, "the usual side block");
            turn(win, "layer-1");
            verify(!pad(win).visible, "hidden under the layer");
            turn(win, "main");
            verify(pad(win).visible, "back with the usual side keys");
        }

        function test_new_cells_fade_and_rise_in_and_the_shortcuts_key_stays() {
            const win = make("main");
            turn(win, "layer-1");
            const ids = cells(win).map(k => k.id).concat(["layer-next"]);
            for (const id of ids)
                verify(keyed(win, id).swapping, id + " swaps in");
            verify(!keyed(win, "shortcuts").swapping, "the Shortcuts key stays");
            tryVerify(() => ids.every(id => !keyed(win, id).swapping && keyed(win, id).opacity === 1), 2000, "in place");
            turn(win, "layer-1");
            verify(!ids.some(id => keyed(win, id).swapping), "the same cells read again stay still");
            turn(win, "layer-3");
            verify(keyed(win, "sc-f5").swapping && !keyed(win, "layer-next").swapping, "only the new page's cells");
            turn(win, "main");
            verify(keyed(win, "rclick").swapping && pad(win).swapping, "the usual side keys and the pad come back");
        }

        function test_with_reduced_motion_the_swap_does_not_animate() {
            const win = make("main", { look: Reduced.lk });
            compare(win.lk.motion.moves.layer.ms, 0, "reduced");
            turn(win, "layer-1");
            for (const id of cells(win).map(k => k.id).concat(["layer-next"])) {
                const key = keyed(win, id);
                verify(!key.swapping && key.opacity === 1, id + " shows at once");
            }
            turn(win, "main");
            verify(!pad(win).swapping && pad(win).opacity === 1, "the pad too");
        }

        function test_a_dimmed_power_key_says_why() {
            const win = make("main");
            const key = keyed(win, "power");
            verify(key.dim, "the power module is not live");
            compare(key.Accessible.description, En.texts["tip.power-off"]);
            // It shows its icon in every theme, so its long French label never has to fit its cap.
            compare(shown(key), { texts: [], icons: [win.lk.icons.keys.power] });
            mouseMove(key, key.width / 2, key.height / 2);
            compare(Board.tipOf(win).name, En.texts["key.power"] + "\n" + En.texts["tip.power-off"]);
            click(win, key);
            compare(win.keyboard.calls.filter(c => c.indexOf("tap power") === 0).length, 0, "it sends nothing");
        }
    }
}

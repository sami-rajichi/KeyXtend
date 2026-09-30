// The language key's Cycle and List designs and the language list over stand-in bridges: the list pages five rows,
// the board's arrows, Enter and Esc drive it, a row or page button click reaches the keyboard, and Cycle shows one dot
// per layout.
import QtQuick
import QtTest
import "../qml"
import "fixtures/assets.js" as Assets
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", ui: "Segoe UI" },
        icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        bar: id => id,
        dpad: id => id,
        space: parts => parts.join(" · ")
    })

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: listC; LangList {} }
    Component { id: cycleC; LangCycle {} }
    Component { id: keyC; Key {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "lang"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
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
        // A list view of `n` layouts, the third current, row `hi` highlighted, paged by the board's arrows.
        function view(n, hi) {
            const row = i => ({ short: "L" + i, name: ["Language " + i, "Layout " + i], rtl: false, arabic: false, current: i === 2, icon: i === 2 ? "lang.current" : null });
            const face = Face.states.list.list;
            return { rows: [...Array(n).keys()].map(row), hi: hi, keys: face.keys, page_back: face.page_back, page_on: face.page_on };
        }
        function makeList(v, rtl) {
            const ui = createTemporaryObject(uiC, root);
            return createTemporaryObject(listC, root, { ui: ui, lk: N.lk, res: root.res, s: 1, view: v, rows: En.settings.pageRows, rtl: !!rtl, title: "list" });
        }
        // The rows a screen reader meets, and every row slot, padding too.
        function rowsOf(list) {
            return all(list.contentItem, o => o.Accessible.role === Accessible.ListItem && !o.Accessible.ignored);
        }
        function slots(list) {
            return all(list.contentItem, o => o.place !== undefined && o.hi !== undefined);
        }
        function button(item, name) {
            return all(item, o => o.Accessible.role === Accessible.Button && o.Accessible.name === name)[0];
        }
        // The page number the pager shows, the one text a screen reader meets besides the rows.
        function pageNumber(list) {
            return all(list.contentItem, o => o.Accessible.role === Accessible.StaticText && !o.Accessible.ignored)[0];
        }

        function test_the_list_pages_five_rows_and_shows_the_page_of_the_highlight() {
            compare(En.settings.pageRows, 5, "the [ui] page_rows default");
            const list = makeList(view(7, 0));
            compare(rowsOf(list).map(r => r.Accessible.name), [0, 1, 2, 3, 4].map(i => root.res.space(["L" + i, "Language " + i, "Layout " + i])));
            const number = pageNumber(list);
            verify(number.visible, "a pager with two pages");
            compare(number.text, list.ui.pageText(1, 2));
            list.view = view(7, 6);
            compare(rowsOf(list).length, 2, "the second page");
            compare(slots(list).length, 5, "padded to a whole page, so the box keeps its size");
            compare(slots(list).filter(r => r.hi).map(r => r.place), [6], "the ring on the highlight");
            compare(number.text, list.ui.pageText(2, 2));
        }

        function test_the_arabic_ui_puts_the_pager_on_the_left_and_mirrors_the_rows() {
            for (const rtl of [false, true]) {
                const list = makeList(view(7, 0), rtl);
                list.open();
                waitForRendering(list.contentItem);
                const back = button(list.contentItem, En.texts["page.back"]);
                const row = rowsOf(list)[0];
                const [pagerX, rowX] = [back, row].map(o => o.mapToItem(list.contentItem, 0, 0).x);
                verify(rtl ? pagerX < rowX : pagerX > rowX, "pager " + pagerX + ", rows " + rowX);
                const badge = all(row, o => o.text === "L0")[0];
                const name = all(row, o => o.text === "Language 0")[0];
                const [b, n] = [badge, name].map(o => o.mapToItem(row, 0, 0).x);
                verify(rtl ? b > n : b < n, "badge " + b + ", name " + n);
                list.visible = false;
            }
        }

        function test_a_pager_taller_than_its_rows_stays_inside_the_box() {
            const ui = createTemporaryObject(uiC, root);
            const list = createTemporaryObject(listC, root, { ui: ui, lk: N.lk, res: root.res, s: 1, view: view(7, 0), rows: 2, rtl: false, title: "list" });
            list.open();
            waitForRendering(list.contentItem);
            const back = button(list.contentItem, En.texts["page.back"]);
            const on = button(list.contentItem, En.texts["page.on"]);
            verify(back.mapToItem(list.contentItem, 0, 0).y >= 0, "the top button shows");
            verify(on.mapToItem(list.contentItem, 0, 0).y + on.height <= list.height, "the bottom button shows");
            list.visible = false;
        }

        function test_a_short_list_has_one_page_and_no_pager() {
            const list = makeList(view(3, 1));
            compare(slots(list).length, 3, "no padding");
            verify(!pageNumber(list).visible);
            compare(slots(list).filter(r => r.hi).map(r => r.place), [1]);
            const current = slots(list).filter(r => r.row.current)[0];
            verify(current.Accessible.checked, "readers hear the current layout");
            const tick = all(current, o => o.tint !== undefined && o.icon !== undefined)[0];
            compare([tick.icon, tick.tint], [N.lk.icons.lang.current, Qt.color(N.lk.palette.ring)]);
        }

        function test_a_row_click_picks_it_and_the_page_buttons_send_the_board_keys() {
            const list = makeList(view(7, 6));
            const pick = createTemporaryObject(spyC, root, { target: list, signalName: "pick" });
            const page = createTemporaryObject(spyC, root, { target: list, signalName: "page" });
            list.open();
            waitForRendering(list.contentItem);
            mouseClick(rowsOf(list)[1]);
            compare(pick.signalArguments[0][0], 6, "counted over the whole list");
            verify(!button(list.contentItem, En.texts["page.on"]).enabled, "readers hear there is no next page");
            mouseClick(button(list.contentItem, En.texts["page.on"]));
            compare(page.count, 0, "no page after the last");
            mouseClick(button(list.contentItem, En.texts["page.back"]));
            compare(page.signalArguments[0][0], "left");
            list.visible = false;
        }

        function test_the_boards_arrows_enter_and_esc_drive_the_open_list() {
            const ui = createTemporaryObject(uiC, root);
            const board = Object.assign({}, Face.board, { lang_key: "list" });
            const kb = createTemporaryObject(kbC, root, { board: board, state: Face.states.list });
            const win = createTemporaryObject(winC, root, { ui: ui, keyboard: kb });
            win.visible = true;
            waitForRendering(win.contentItem);
            for (const id of ["up", "down", "left", "right", "enter", "esc"])
                mouseClick(button(win.contentItem, En.texts["key." + id]));
            // A character key types as usual: the key at US "q" in this French state.
            mouseClick(button(win.contentItem, Face.states.list.keys["c-0x0010"].name));
            const sent = kb.calls.filter(c => c.startsWith("panelKey") || c.startsWith("tap"));
            compare(sent, ["panelKey up", "panelKey down", "panelKey left", "panelKey right", "panelKey enter", "panelKey esc", "tap c-0x0010 false"]);
            tryVerify(() => ui.calls.indexOf("popAt over left") >= 0, 2000, "the list opens by the language key");
            const panel = [...win.data].find(o => o.item && o.item.win === win).item;
            const list = [...panel.data].find(o => o.view !== undefined);
            tryVerify(() => list.visible && panel.visible, 2000, "the list and its shadow show");
            waitForRendering(list.contentItem);
            mouseClick(rowsOf(list)[0]);
            verify(kb.calls.some(c => c.startsWith("langPick")), kb.calls.join());
            const placed = () => ui.calls.filter(c => c === "popAt over left").length;
            const before = placed();
            win.at = Qt.point(win.at.x + 10, win.at.y);
            compare(placed(), before + 1, "placed again as the keyboard moves");
            win.visible = false;
        }

        function test_the_cycle_design_shows_a_globe_the_name_and_one_dot_per_layout() {
            const lang = Face.states.en.lang;
            const c = createTemporaryObject(cycleC, root, { lang: lang, lk: N.lk, res: root.res, s: 1, ink: "black", room: 40 });
            const dots = all(c, o => o.radius !== undefined && o.width === N.lk.shape.lang.cycle.dot_px);
            compare(dots.length, lang.count, "one dot per layout");
            compare(dots.map(d => d.opacity), [...Array(lang.count).keys()].map(i => i === lang.at ? 1 : N.lk.shape.lang.cycle.dot_off));
            compare(all(c, o => o.tint !== undefined && o.icon !== undefined)[0].icon, N.lk.icons.lang.cycle);
            compare(all(c, o => o.text === lang.cur.short).length, 1, "the current name");
            c.lang = Face.states.ar.lang;
            compare(all(c, o => o.text === Face.states.ar.lang.cur.short).length, 1, "the next name rises in");
        }

        function test_many_layouts_wrap_their_dots_inside_the_cap() {
            const lang = Object.assign({}, Face.states.en.lang, { count: 8, at: 5 });
            const c = createTemporaryObject(cycleC, root, { lang: lang, lk: N.lk, res: root.res, s: 1, ink: "black", room: 20 });
            const dots = () => all(c, o => o.radius !== undefined && o.width === N.lk.shape.lang.cycle.dot_px);
            compare(dots().length, 8);
            tryVerify(() => dots().every(d => d.mapToItem(c, 0, 0).y + d.height <= 20), 2000, "no dot below the cap");
            verify(new Set(dots().map(d => d.x)).size > 1, "more than one column");
        }

        function test_each_design_draws_the_language_key_its_own_way() {
            const kv = Face.board.keys.find(k => k.kind === "lang");
            const lang = Face.states.en.lang;
            const shows = design => {
                const key = createTemporaryObject(keyC, root, { kv: kv, ks: Face.states.en.keys[kv.id], lk: N.lk, lang: lang, design: design, res: root.res, s: 1 });
                const visible = o => {
                    for (let p = o; p && p !== key; p = p.parent)
                        if (!p.visible)
                            return false;
                    return true;
                };
                return all(key, o => o.icon !== undefined && o.tint !== undefined && o.icon.length > 0 && visible(o)).map(o => o.icon);
            };
            compare(shows("carousel"), []);
            compare(shows("cycle"), [N.lk.icons.lang.cycle]);
            compare(shows("list"), [N.lk.icons.lang.list]);
        }
    }
}

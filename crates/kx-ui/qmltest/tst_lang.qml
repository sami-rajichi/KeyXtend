// The language key's List design and the language list over stand-in bridges: the list pages five rows, the board's
// arrows, Enter and Esc drive it, and a row or page button click reaches the keyboard.
import QtQuick
import QtTest
import "../qml"
import "fixtures/assets.js" as Assets
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", mark: "Segoe UI", ui: "Segoe UI" },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        bar: id => id,
        dpad: id => id,
        uid: (group, id) => group + "." + id,
        space: parts => parts.join(" · ")
    })

    Component { id: uiC; FakeUi {} }
    Component { id: kbC; FakeKeyboard {} }
    Component { id: winC; KbWindow {} }
    Component { id: listC; LangList {} }
    Component { id: spyC; SignalSpy {} }
    // Measures text in a given font.
    Component { id: metricsC; FontMetrics {} }

    TestCase {
        name: "lang"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A list view of `n` layouts, the third current, row `hi` highlighted on page `at` of `count` (one by default),
        // paged by the board's arrows.
        function view(n, hi, at, count) {
            const row = i => ({ short: "L" + i, name: ["Language " + i, "Layout " + i], rtl: false, arabic: false, current: i === 2, icon: i === 2 ? "lang.current" : null });
            const face = Face.states.list.list;
            const page = paged(at || 0, count || 1);
            return { rows: [...Array(n).keys()].map(row), hi: hi, page: page, keys: face.keys, page_back: face.page_back, page_on: face.page_on };
        }
        // Page `at` of `count`, as the keyboard's state holds it.
        function paged(at, count) {
            return { at: at, count: count, back: at > 0, on: at < count - 1 };
        }
        function makeList(v, res) {
            const ui = createTemporaryObject(uiC, root);
            return createTemporaryObject(listC, root, { ui: ui, lk: N.lk, res: res || root.res, s: 1, view: v, rows: En.settings.pageRows, title: "list" });
        }
        // The rows a screen reader meets, and every row slot, padding too.
        function rowsOf(list) {
            return Board.all(list.contentItem, o => o.Accessible.role === Accessible.ListItem && !o.Accessible.ignored);
        }
        function slots(list) {
            return Board.all(list.contentItem, o => o.place !== undefined && o.hi !== undefined);
        }
        function button(item, name) {
            return Board.all(item, o => o.Accessible.role === Accessible.Button && o.Accessible.name === name)[0];
        }
        // The page number the pager shows, the one text a screen reader meets besides the rows.
        function pageNumber(list) {
            return Board.all(list.contentItem, o => o.Accessible.role === Accessible.StaticText && !o.Accessible.ignored)[0];
        }

        function test_the_list_pages_five_rows_and_shows_the_page_of_the_highlight() {
            compare(En.settings.pageRows, 5, "the [ui] page_rows default");
            const list = makeList(view(7, 0, 0, 2));
            compare(rowsOf(list).map(r => r.Accessible.name), [0, 1, 2, 3, 4].map(i => root.res.space(["L" + i, "Language " + i, "Layout " + i])));
            const number = pageNumber(list);
            verify(number.visible, "a pager with two pages");
            compare(number.text, list.ui.pageText(1, 2));
            list.view = view(7, 6, 1, 2);
            compare(rowsOf(list).length, 2, "the second page");
            compare(slots(list).length, 5, "padded to a whole page, so the box keeps its size");
            compare(slots(list).filter(r => r.hi).map(r => r.place), [6], "the ring on the highlight");
            compare(number.text, list.ui.pageText(2, 2));
        }

        // Arabic flips text only (ADR-0020): the pager stays on the right and each badge before its name.
        function test_the_arabic_ui_keeps_the_lists_places_and_arabic_names_read_right_to_left() {
            const ar = Object.assign({}, root.res, { tr: k => Ar.texts[k] !== undefined ? Ar.texts[k] : k });
            const v = view(7, 0, 0, 2);
            v.rows[1] = { short: "ع", name: ["العربية", "Arabic 101"], rtl: true, arabic: true, current: false, icon: null };
            const list = makeList(v, ar);
            list.open();
            waitForRendering(list.contentItem);
            const back = button(list.contentItem, Ar.texts["page.back"]);
            const row = rowsOf(list)[0];
            const [pagerX, rowX] = [back, row].map(o => o.mapToItem(list.contentItem, 0, 0).x);
            verify(pagerX > rowX, "pager " + pagerX + ", rows " + rowX);
            const badge = Board.all(row, o => o.text === "L0")[0];
            const name = Board.all(row, o => o.text === "Language 0")[0];
            const [b, n] = [badge, name].map(o => o.mapToItem(row, 0, 0).x);
            verify(b < n, "badge " + b + ", name " + n);
            const arabic = Board.all(rowsOf(list)[1], o => o.text === "العربية")[0];
            compare(arabic.effectiveHorizontalAlignment, Text.AlignRight, "Arabic reads right to left");
            compare(name.effectiveHorizontalAlignment, Text.AlignLeft);
            list.visible = false;
        }

        function test_a_pager_taller_than_its_rows_stays_inside_the_box() {
            const ui = createTemporaryObject(uiC, root);
            const list = createTemporaryObject(listC, root, { ui: ui, lk: N.lk, res: root.res, s: 1, view: view(7, 0, 0, 4), rows: 2, title: "list" });
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
            const tick = Board.all(current, o => o.tint !== undefined && o.icon !== undefined)[0];
            compare([tick.icon, tick.tint], [N.lk.icons.lang.current, Qt.color(N.lk.palette.ring)]);
        }

        // The keyboard works the page out, so the list shows the page it names and dims a button it says cannot turn.
        function test_the_list_shows_the_page_the_keyboard_works_out() {
            const v = view(7, 0, 1, 2);
            v.page.back = false;
            const list = makeList(v);
            list.open();
            waitForRendering(list.contentItem);
            compare(rowsOf(list).map(r => r.row.short), ["L5", "L6"], "the second page, whatever the highlight");
            compare(pageNumber(list).text, list.ui.pageText(2, 2));
            verify(!button(list.contentItem, En.texts["page.back"]).enabled, "the keyboard says no page lies back");
            list.visible = false;
        }

        function test_a_row_click_picks_it_and_the_page_buttons_send_the_board_keys() {
            const list = makeList(view(7, 6, 1, 2));
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

        function test_a_press_on_the_list_is_seen_and_still_picks() {
            const list = makeList(view(3, 0));
            const spy = createTemporaryObject(spyC, root, { target: list, signalName: "touched" });
            const pick = createTemporaryObject(spyC, root, { target: list, signalName: "pick" });
            list.open();
            waitForRendering(list.contentItem);
            mouseClick(rowsOf(list)[1]);
            compare(spy.count, 1, "seen, so the tray menu closes");
            compare(pick.signalArguments[0][0], 1, "and the row is still picked");
            list.visible = false;
        }

        // Screen readers press a row, or toggle it since rows are checkable; both pick it.
        function test_a_reader_press_or_toggle_picks_a_row() {
            const list = makeList(view(3, 0));
            const pick = createTemporaryObject(spyC, root, { target: list, signalName: "pick" });
            list.open();
            waitForRendering(list.contentItem);
            rowsOf(list)[1].Accessible.pressAction();
            rowsOf(list)[2].Accessible.toggleAction();
            compare(pick.signalArguments.map(a => a[0]), [1, 2], "press and toggle each pick their row");
            list.visible = false;
        }

        // The bundled Arabic fallback font stands taller than the Latin one, as Arabic fonts do.
        function test_an_arabic_row_fits_its_name_and_layout() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            const arabic = N.lk.look.arabic_fonts[N.lk.look.arabic_fonts.length - 1];
            const sh = N.lk.shape.list;
            const tall = createTemporaryObject(metricsC, root, { font: Qt.font({ family: arabic, pixelSize: sh.name_px }) });
            const latin = createTemporaryObject(metricsC, root, { font: Qt.font({ family: root.res.fam.latin, pixelSize: sh.name_px }) });
            verify(tall.height > latin.height, "the case this test is for: " + tall.height + " " + latin.height);
            const res = Object.assign({}, root.res, { fam: { latin: root.res.fam.latin, arabic: arabic, mark: arabic, ui: root.res.fam.ui } });
            const row = { short: "ع", name: ["العربية", "Arabic 101"], rtl: true, arabic: true, current: false, icon: null };
            const v = { rows: [row], hi: 0, page: paged(0, 1), keys: [], page_back: null, page_on: null };
            const ui = createTemporaryObject(uiC, root);
            const list = createTemporaryObject(listC, root, { ui: ui, lk: N.lk, res: res, s: 1, view: v, rows: 5, title: "list" });
            const slot = slots(list)[0];
            const name = Board.all(slot, o => o.text === row.name[0])[0];
            const sub = Board.all(slot, o => o.text === root.res.space(row.name.slice(1)))[0];
            const room = slot.height - 2 * sh.row_pad_px[0];
            verify(name.implicitHeight + sub.implicitHeight <= room, "no clipping: " + (name.implicitHeight + sub.implicitHeight) + " in " + room);
            const latinOnly = createTemporaryObject(listC, root, { ui: ui, lk: N.lk, res: res, s: 1, view: view(3, 0), rows: 5, title: "list" });
            verify(latinOnly.rowH < list.rowH, "a list with no Arabic row keeps its height");
        }

        // An OS may wrap its names in isolates; a row draws them through `res.space`, which drops them as the hub does.
        function test_a_row_draws_marked_os_names_cleaned() {
            const marks = /[\u2066-\u2069]/g;
            const res = Object.assign({}, root.res, { space: parts => root.res.space(parts.map(p => p.replace(marks, ""))) });
            const v = view(1, 0);
            v.rows[0].name = ["\u2068العربية\u2069", "\u2066Arabic (102)\u2069"];
            const drawn = Board.all(makeList(v, res).contentItem, o => o.font !== undefined && o.visible).map(o => o.text);
            verify(["العربية", "Arabic (102)"].every(n => drawn.indexOf(n) >= 0) && drawn.every(t => t.search(marks) < 0), JSON.stringify(drawn));
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
            // A character key types as usual, by its press and let-go: the key at US "q" in this French state.
            mouseClick(button(win.contentItem, Face.states.list.keys["c-0x0010"].name));
            const sent = kb.calls.filter(c => /^(panelKey|tap|press|letGo)\b/.test(c));
            compare(sent, ["panelKey up", "panelKey down", "panelKey left", "panelKey right", "panelKey enter", "panelKey esc", "press c-0x0010", "letGo c-0x0010"]);
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
    }
}

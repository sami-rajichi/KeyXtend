// The pop-ups' and the plate's other controls act at let-go on the press they got (ADR-0024): the language list's
// rows and page buttons, the tray menu, the notice buttons, chips and the D-pad. A slip before the release still acts
// once, only once, and nothing it slips onto acts.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/assets.js" as Assets
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N
import "pointer.js" as Pointer

Item {
    id: root
    width: 420
    height: 240

    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", mark: "Segoe UI", ui: "Segoe UI", tray: "Segoe UI" },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        tray: k => En.texts[k] !== undefined ? En.texts[k] : k,
        bar: id => En.texts["bar." + id],
        dpad: id => En.texts["dpad." + id],
        uid: (group, id) => group + "." + id,
        space: parts => parts.join(" · "),
        noteTip: (name, note) => name + "\n" + note
    })

    Component { id: uiC; FakeUi {} }
    Component { id: listC; LangList {} }
    Component { id: menuC; TrayMenu {} }
    Component { id: noteC; Notice {} }
    Component { id: chipC; Chip {} }
    Component { id: padC; DPad {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "letgo_popups"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function spy(target, signalName) {
            return createTemporaryObject(spyC, root, { target: target, signalName: signalName });
        }
        // The middle of `other`, in `item`'s units.
        function onto(item, other) {
            return other.mapToItem(item, other.width / 2, other.height / 2);
        }
        // A point left of `item`'s window, in `item`'s units.
        function outside(item) {
            const w = item.Window;
            return item.mapFromItem(w.contentItem, -w.width / 2, w.height / 2);
        }
        // Presses `item` at `from` (its middle by default), slips the still-down pointer to `to` and lets go there.
        function slip(item, to, from) {
            const at = from || Qt.point(item.width / 2, item.height / 2);
            mousePress(item, at.x, at.y);
            mouseMove(item, to.x, to.y, -1, Qt.LeftButton);
            mouseRelease(item, to.x, to.y);
        }
        function named(item, role, name) {
            return Board.all(item, o => o.Accessible.role === role && o.Accessible.name === name)[0];
        }

        function test_a_list_row_and_a_page_button_act_once_after_a_slip_onto_another() {
            const face = Face.states.list.list;
            const row = i => ({ short: "L" + i, name: ["Language " + i, "Layout " + i], rtl: false, arabic: false, current: false, icon: null });
            const first = { at: 0, count: 2, back: false, on: true };
            const view = { rows: [...Array(7).keys()].map(row), hi: 0, page: first, keys: face.keys, page_back: face.page_back, page_on: face.page_on };
            const list = createTemporaryObject(listC, root, { ui: createTemporaryObject(uiC, root), lk: N.lk, res: root.res, s: 1, view: view, rows: En.settings.pageRows, title: "list" });
            Pointer.through([list]);
            const [pick, page] = [spy(list, "pick"), spy(list, "page")];
            list.open();
            waitForRendering(list.contentItem);
            const rows = Board.all(list.contentItem, o => o.Accessible.role === Accessible.ListItem && !o.Accessible.ignored);
            slip(rows[1], onto(rows[1], rows[2]));
            compare([pick.count, pick.signalArguments[0][0]], [1, 1], "the row pressed, once");
            const on = named(list.contentItem, Accessible.Button, En.texts["page.on"]);
            slip(on, outside(on));
            compare([page.count, page.signalArguments[0][0]], [1, face.page_on], "the page button pressed, once");
            list.visible = false;
        }

        // Quit with a slip onto Show quits once and nothing else; Show with a slip onto Quit only shows.
        function test_a_tray_menu_item_acts_once_after_a_slip_onto_the_other() {
            const ui = createTemporaryObject(uiC, root);
            const spot = (x, y, w, h) => {
                const p = JSON.parse(ui.menuAt(x, y, w, h));
                return Qt.point(p.x, p.y);
            };
            const menu = createTemporaryObject(menuC, root, { lk: N.lk, res: root.res, spot: spot, graceMs: En.settings.trayGraceMs });
            Pointer.through([menu]);
            const [show, quit] = [spy(menu, "showKeyboard"), spy(menu, "quitApp")];
            for (const at of [0, 1]) {
                menu.toggle(300, 500);
                waitForRendering(menu.contentItem);
                const items = Board.all(menu.contentItem, o => o.Accessible.role === Accessible.MenuItem);
                slip(items[at], onto(items[at], items[1 - at]));
                verify(!menu.visible, "closed on the pick");
            }
            compare([show.count, quit.count], [1, 1]);
        }

        function test_a_notice_button_acts_once_after_a_slip_onto_the_other_or_off_the_banner() {
            const waiting = [{ id: 7, text: "Notice 7", retry: true }, { id: 8, text: "Notice 8", retry: false }];
            const ui = createTemporaryObject(uiC, root, { notices: waiting });
            const note = createTemporaryObject(noteC, root, { ui: ui, lk: N.lk, res: root.res, s: 1, widest: N.lk.shape.panel.width_px, spot: (w, h) => Qt.point(20, 30) });
            Pointer.through([note]);
            tryVerify(() => note.visible, 2000, "it shows");
            waitForRendering(note.contentItem);
            const retry = named(note.contentItem, Accessible.Button, En.texts["button.retry"]);
            slip(retry, onto(retry, named(note.contentItem, Accessible.Button, En.texts["button.ok"])));
            compare(ui.calls, ["retry 7"], "Try again, once");
            waitForRendering(note.contentItem);
            const ok = named(note.contentItem, Accessible.Button, En.texts["button.ok"]);
            slip(ok, outside(ok));
            compare(ui.calls, ["retry 7", "dismiss 8"]);
        }

        // Each chip is found by its word, as a screen reader finds it.
        function test_a_chip_acts_once_after_a_slip_onto_the_next_chip() {
            const words = [En.texts["key.clip"], En.texts["key.fill"]];
            const make = (word, x) => createTemporaryObject(chipC, root, { chip: { text: word, icon: N.lk.icons.keys.clip }, lk: N.lk, res: root.res, s: 1, radius: 4, x: x });
            const first = make(words[0], 0);
            make(words[1], first.width * 2);
            const [a, b] = words.map(w => named(root, Accessible.Button, w));
            verify(a === first && b && b !== a, "each chip by its word");
            const [sa, sb] = [spy(a, "clicked"), spy(b, "clicked")];
            slip(a, onto(a, b));
            compare([sa.count, sb.count], [1, 0]);
        }

        // A working pad, as P5 brings: an arrow slipped off its quarter, or Stop slipped off the pad, acts once.
        function test_a_dpad_arrow_and_stop_act_once_after_a_slip() {
            const box = Object.assign({}, Face.board.dpad, { dim: false, tip: null, dir: null, fast: false });
            const pad = createTemporaryObject(padC, root, { box: box, lk: N.lk, res: root.res, s: 1 });
            const taps = spy(pad, "tapped");
            waitForRendering(pad);
            // An arrow's turned shape, which holds its pointer area, or Stop.
            const arrow = id => Board.all(pad, o => id === "stop" ? o.Accessible.id === "dpad.stop" : o.fill !== undefined && o.modelData !== undefined && o.modelData.id === id)[0];
            const hit = a => Board.all(a, o => o.containsMouse !== undefined)[0];
            const glyph = a => Board.all(a, o => o.icon !== undefined && o.tint !== undefined)[0];
            const up = hit(arrow("up"));
            const start = onto(up, glyph(arrow("up")));
            slip(up, onto(up, glyph(arrow("down"))), start);
            compare(taps.signalArguments.map(a => a[0]), ["up"], "up scrolls, down never");
            pad.box = Object.assign({}, box, { dir: "up" });
            const stop = arrow("stop");
            tryCompare(stop, "scale", 1, 2000, "Stop pops in");
            const area = hit(stop);
            slip(area, Qt.point(-pad.width, area.height / 2));
            compare(taps.signalArguments.map(a => a[0]), ["up", ""], "Stop, once");
        }
    }
}

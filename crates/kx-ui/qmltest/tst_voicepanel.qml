// The Voice panel over the hub's views (design "The gear and the Voice panel"): the home list with each page's value,
// five row slots on every page, the pager and the wheel, History's ticks with Insert (2) and its question, an empty
// History, and a window that takes clicks but never focus, in English, French and Arabic, each part found by its
// accessible name or id.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "fixtures/voicepanel.js" as Panel
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    Component { id: namesC; VoiceNames {} }

    VoiceCase {
        name: "voicepanel"
        host: root

        // Every item of each list `f` gives for the items of `all`, in one list, as V4 has no flatMap.
        function flat(all, f) {
            return all.reduce((acc, x) => acc.concat(f(x)), []);
        }
        function test_the_names_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            const panels = Object.values(Panel.views.en).map(v => v.panel);
            compare([...new Set(panels.map(p => p.sheet.is))].sort(), Object.values(names.sheets).sort(), "every page body");
            const tones = flat(panels, p => p.foot.concat(p.back ? [p.back] : [], [p.close]).map(b => b.tone));
            compare([...new Set(tones)].sort(), Object.keys(names.tones).sort(), "every tone");
            const kinds = flat(panels.filter(p => p.sheet.is === names.sheets.options), p => p.sheet.rows.map(r => r.kind));
            compare([...new Set(kinds)].sort(), Object.values(names.options).sort(), "every option kind");
            const marks = flat(panels, p => flat(p.sheet.rows || [], r => (r.marks || []).concat(r.mark ? [r.mark] : [])));
            for (const id of names.lit.concat(names.outlined, [names.writing]))
                verify(marks.some(m => m.id === id), id + " is a mark the hub writes");
        }

        function test_the_home_list_shows_each_page_with_its_value_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        function test_the_home_list_shows_each_page_with_its_value(d) {
            const box = make("home", d.t);
            const p = box.view;
            compare(box.title, p.window);
            verify(Kit.texts(box.contentItem, p.title).length === 1, "the title");
            verify(Kit.texts(box.contentItem, p.sub).length === 1, "the language line");
            const shown = rows(box);
            compare(shown.map(r => r.Accessible.name), p.sheet.rows.map(r => r.label));
            compare(shown.map(r => r.Accessible.description), p.sheet.rows.map(r => r.value));
            compare(Board.all(box.contentItem, o => o.visible && o.Accessible.id === "voice.back").length, 0, "no Back at home");
            verify(Kit.texts(box.contentItem, p.line).length === 1, "Try again's line");
            mouseClick(shown[1]);
            mouseClick(one(box, "voice.again"));
            mouseClick(one(box, "voice.close"));
            compare(acts(box), ["act open device 0 " + box.round, "act button again 0 " + box.round, "act button close 0 " + box.round]);
            box.visible = false;
        }

        function test_every_page_keeps_five_row_slots_and_pages_with_its_buttons() {
            const full = make("models");
            const short = make("modelsMore");
            compare(rows(short).length, short.view.sheet.rows.length, "two rows readers find");
            compare(Board.all(short.contentItem, o => o.place !== undefined && o.entry !== undefined).length, En.settings.pageRows, "five slots");
            const list = Board.all(short.contentItem, o => o.Accessible.id === "voice.list")[0];
            const fullList = Board.all(full.contentItem, o => o.Accessible.id === "voice.list")[0];
            compare(list.height, fullList.height, "a short page keeps its height");
            compare(full.width, Math.ceil(N.lk.shape.voice.panel_px), "its own width token");
            verify(Kit.texts(short.contentItem, short.ui.pageText(2, 2)).length === 1, "page 2 of 2");
            verify(!one(short, "voice.page-on").enabled, "no page after the last");
            mouseClick(one(short, "voice.page-back"));
            mouseWheel(list, list.width / 2, list.height / 2, 0, N.lk.shape.panel.wheel_notch);
            mouseWheel(list, list.width / 2, list.height / 2, 0, -N.lk.shape.panel.wheel_notch);
            const r = short.round;
            compare(acts(short), ["act page  -1 " + r, "act wheel  -1 " + r], "the wheel never pages past the last");
            full.visible = short.visible = false;
        }

        function test_history_numbers_its_ticks_and_inserts_two_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        function test_history_numbers_its_ticks_and_inserts_two(d) {
            const box = make("history", d.t);
            const p = box.view;
            const insert = one(box, "voice.insert");
            compare(insert.label, p.foot[3].label);
            if (d.t === "en")
                compare(insert.label, "Insert (2)");
            compare(buttons(box.foot).map(b => b.Accessible.id), ["voice.copy", "voice.pin", "voice.delete", "voice.insert"], "Insert last");
            const shown = rows(box);
            compare(shown.map(r => r.Accessible.name), p.sheet.rows.map(r => r.name));
            compare(shown.map(r => r.Accessible.checked), [true, true, false]);
            for (const [at, n] of [[0, "2"], [1, "1"]])
                verify(Kit.texts(shown[at], n).length === 1, "tick " + n);
            compare(Board.all(shown[1], o => o.visible && o.icon === N.lk.icons.voice.panel.pinned).length, 1, "the pinned text");
            const arabic = Kit.texts(shown[2], p.sheet.rows[2].text)[0];
            compare(arabic.effectiveHorizontalAlignment, Text.AlignRight, "an Arabic text reads from the right");
            compare(arabic.font.family, box.res.fam.arabic);
            for (const id of ["copy", "pin", "delete", "insert", "delete-all"])
                mouseClick(one(box, "voice." + id));
            mouseClick(shown[2]);
            const r = box.round;
            compare(acts(box), ["copy", "pin", "delete", "insert", "delete-all"].map(id => "act button " + id + " 0 " + r).concat(["act row  2 " + r]));
            box.visible = false;
        }

        function test_unpin_shows_its_own_icon() {
            const v = Kit.view("en", "history");
            v.panel.sheet.unpin = true;
            const box = make("history");
            compare(one(box, "voice.pin").icon, N.lk.icons.voice.panel.buttons.pin);
            box.view = v.panel;
            compare(one(box, "voice.pin").icon, N.lk.icons.voice.panel.unpin);
            box.visible = false;
        }

        function test_delete_all_asks_in_the_footer_and_waits_for_the_answer() {
            const box = make("historyAsk");
            const p = box.view;
            const ask = Kit.only(Board.all(box.contentItem, o => o.visible && o.Accessible.role === Accessible.AlertMessage));
            compare(ask.Accessible.name, p.line, "readers hear the question");
            const all = one(box, "voice.delete-all");
            verify(!all.enabled && all.opacity < 1, "Delete all shows dimmed while it asks");
            const list = Board.all(box.contentItem, o => o.Accessible.id === "voice.list")[0];
            mouseWheel(list, list.width / 2, list.height / 2, 0, -N.lk.shape.panel.wheel_notch);
            mouseClick(one(box, "voice.yes"));
            mouseClick(one(box, "voice.no"));
            compare(acts(box), ["act confirm  1 " + box.round, "act confirm  0 " + box.round], "the wheel waits too");
            box.visible = false;
        }

        function test_an_empty_history_says_so_and_dims_its_buttons() {
            const box = make("historyEmpty");
            verify(heard(box.contentItem, box.view.sheet.empty).length === 1, "its line");
            compare(rows(box).length, 0);
            for (const b of buttons(box.foot).concat([one(box, "voice.delete-all")])) {
                verify(!b.enabled, b.Accessible.id + " cannot act");
                compare(b.opacity, N.lk.shape.panel.off_share, b.Accessible.id + " shows dimmed, never hidden");
                mouseClick(b);
            }
            compare(acts(box), []);
            box.visible = false;
        }

        // It never takes focus, so the app in front keeps its caret; Back and readers' presses act as clicks.
        function test_the_window_takes_clicks_but_never_focus() {
            const box = make("device");
            verify(box.flags & Qt.WindowDoesNotAcceptFocus, "never takes focus");
            compare(one(box, "voice.back").Accessible.name, box.view.back.label);
            mouseClick(one(box, "voice.back"));
            rows(box)[2].Accessible.toggleAction();
            one(box, "voice.use").Accessible.pressAction();
            const r = box.round;
            compare(acts(box), ["act button back 0 " + r, "act row  2 " + r, "act button use 0 " + r]);
            box.visible = false;
        }
    }
}

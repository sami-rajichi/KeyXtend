// The Snip panel over the stand-in Snip object (design "The Snip panel"): above the keyboard at its size, its pages and
// keys named in English, French and Arabic, what the keyboard way picked does, the folder or why none shows, steppers
// at an end saying why, each choice sending its step, presses aiming the letters at Snip, and hiding while Hidden lasts.
import QtQuick
import QtTest
import "../qml"
import "snipkit.js" as Kit
import "fixtures/snip.js" as Snip
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    Component { id: snipC; FakeSnip {} }
    Component { id: winC; FakeWin {} }
    Component { id: appC; SnipApp {} }

    TestCase {
        name: "snippanel"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Snip's windows over a stand-in showing view `name` with `more` in language `tag`, the keyboard at size `s`,
        // once the panel shows; aims counted.
        function make(name, tag, more, s) {
            const snip = createTemporaryObject(snipC, root);
            const win = createTemporaryObject(winC, root, { s: s || 1 });
            const aims = [];
            const app = createTemporaryObject(appC, root, {
                snip: snip, lk: N.lk, res: Kit.resOf(N.lk, tag || "en"), win: win, aim: () => aims.push(true)
            });
            snip.page = Kit.view(name, more);
            const pane = app.panel.pane;
            tryVerify(() => pane.visible, 2000, "the panel shows");
            waitForRendering(pane.contentItem);
            return { snip: snip, win: win, app: app, pane: pane, item: pane.contentItem, aims: aims };
        }
        function names(list) {
            return list.map(o => o.Accessible.name);
        }

        function test_the_capture_page_names_its_keys_and_says_what_the_way_does() {
            for (const tag of Kit.langs) {
                const s = make("panel-capture", tag);
                compare(s.pane.title, Kit.text(tag, "window.snip-panel"));
                verify(s.pane.flags & Qt.WindowDoesNotAcceptFocus, "it never takes focus");
                compare(names(Kit.keys(s.item, "snip.tab.")), Snip.ids.tabs.map(id => Kit.text(tag, "snip.panel-tab-" + id)), tag);
                verify(Kit.key(s.item, "snip.tab.capture").Accessible.checked, "its page is on");
                compare(Kit.key(s.item, "snip.tab.capture").Accessible.role, Accessible.RadioButton, "one page of many");
                compare(names(Kit.keys(s.item, "snip.keyboard.")), Snip.ids.keyboards.map(id => Kit.text(tag, "snip.panel-keyboard-" + id)));
                verify(Kit.key(s.item, "snip.keyboard.bubble").Accessible.checked);
                compare(Kit.texts(s.item, Kit.text(tag, "snip.panel-say-bubble")).length, 1, "what Bubble does");
                compare(Kit.texts(s.item, Kit.text(tag, "snip.panel-folder-none")).length, 1, "no folder yet");
                compare(names(Kit.keys(s.item, "snip.folder.")), Snip.ids.folders.map(id => Kit.text(tag, "snip.panel-folder-" + id)));
                compare(Kit.key(s.item, "snip.panel.close").says, Kit.text(tag, "snip.way-close"));
                compare(Kit.texts(s.item, Kit.text(tag, "snip.way-settings")).length, 1, "its name");
            }
        }

        function test_it_opens_above_the_keyboard_at_its_size() {
            const s = make("panel-capture", "en", {}, 1.5);
            compare([s.pane.x, s.pane.y], [s.win.spot.x, s.win.spot.y], "where a pop-up goes");
            compare(s.pane.width, Math.ceil(N.lk.shape.snip.panel_px * 1.5), "at the keyboard's size");
        }

        function test_it_follows_the_keyboard_as_it_moves_or_changes_size() {
            const s = make("panel-capture");
            s.win.spot = Qt.point(410, 260);
            s.win.at = Qt.point(5, 5);
            compare([s.pane.x, s.pane.y], [410, 260], "placed again as the keyboard moves");
            s.win.spot = Qt.point(150, 90);
            s.win.drawn = Qt.size(640, 220);
            compare([s.pane.x, s.pane.y], [150, 90], "and as it changes size");
        }

        function test_its_folder_shows_and_each_key_sends_its_id() {
            const folder = "D:\\Snips";
            const page = Object.assign({}, Snip.views["panel-capture"].panel, { folder: folder });
            const s = make("panel-capture", "ar", { panel: page });
            compare(Kit.texts(s.item, folder).length, 1, "the folder as it is");
            mouseClick(Kit.key(s.item, "snip.keyboard.hidden"));
            mouseClick(Kit.key(s.item, "snip.folder.open"));
            mouseClick(Kit.key(s.item, "snip.tab.video"));
            mouseClick(Kit.key(s.item, "snip.panel.close"));
            const r = Snip.views["panel-capture"].round;
            compare(s.snip.heard("act"), [
                { act: "keyboard", id: "hidden", arg: 0, round: r },
                { act: "folder", id: "open", arg: 0, round: r },
                { act: "tab", id: "video", arg: 0, round: r },
                { act: "close", id: "", arg: 0, round: r }
            ]);
            compare(s.aims.length, 4, "each press aims the letters at Snip");
        }

        function test_the_video_page_steps_and_says_why_at_an_end() {
            for (const tag of Kit.langs) {
                const s = make("panel-video", tag);
                const p = Snip.views["panel-video"].panel;
                const says = id => Kit.key(s.item, id).Accessible.description;
                compare(says("snip.less.countdown"), Kit.text(tag, "snip.why-least"), tag);
                compare(says("snip.more.countdown"), "");
                compare(says("snip.more.longest"), Kit.text(tag, "snip.why-most"));
                compare(Kit.key(s.item, "snip.less.countdown").Accessible.name, Kit.text(tag, "snip.less-countdown"));
                compare(Kit.texts(s.item, String(p.countdown_s)).length, 1, "the countdown in seconds");
                verify(Kit.texts(s.item, String(p.max_min)).length >= 1, "the longest video in minutes");
                compare(names(Kit.keys(s.item, "snip.fps.")), p.rates.map(String));
                verify(Kit.key(s.item, "snip.fps." + p.rates.indexOf(p.fps)).Accessible.checked, "the rate it records at");
                compare(names(Kit.keys(s.item, "snip.mic.")), [Kit.text(tag, "snip.panel-off"), Kit.text(tag, "snip.panel-on")]);
                verify(Kit.key(s.item, "snip.mic.0").Accessible.checked && Kit.key(s.item, "snip.sound.1").Accessible.checked);
                const ys = Snip.ids.rows.map(id => Kit.texts(s.item, Kit.text(tag, "snip.panel-row-" + id))[0].mapToItem(s.item, 0, 0).y);
                verify(ys.every((y, i) => i === 0 || y > ys[i - 1]), "its rows in the hub's order: " + ys);
            }
        }

        function test_each_video_key_sends_its_step() {
            const s = make("panel-video");
            mouseClick(Kit.key(s.item, "snip.less.countdown"));
            mouseClick(Kit.key(s.item, "snip.more.countdown"));
            mouseClick(Kit.key(s.item, "snip.less.longest"));
            mouseClick(Kit.key(s.item, "snip.fps.0"));
            mouseClick(Kit.key(s.item, "snip.mic.1"));
            mouseClick(Kit.key(s.item, "snip.sound.0"));
            const r = Snip.views["panel-video"].round;
            const step = (id, arg) => ({ act: "step", id: id, arg: arg, round: r });
            compare(s.snip.heard("act"), [step("countdown", 1), step("longest", -1), step("fps", -1), step("mic", 1), step("sound", -1)]);
        }

        function test_a_system_dialog_drops_it_from_the_top() {
            const s = make("panel-capture", "en", { low: true });
            verify(!(s.pane.flags & Qt.WindowStaysOnTopHint), "the panel leaves the top");
            verify(!(s.app.panel.flags & Qt.WindowStaysOnTopHint), "and its shadow");
        }

        function test_it_hides_while_hidden_lasts_and_never_without_a_keyboard() {
            const s = make("panel-capture");
            s.app.hush = true;
            verify(!s.pane.visible && !s.app.panel.visible, "it hides with its shadow");
            s.app.hush = false;
            tryVerify(() => s.pane.visible, 2000, "and comes back");
            s.app.win = null;
            verify(!s.pane.visible, "no keyboard window, no panel");
        }
    }
}

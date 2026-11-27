// The keyboard stepping aside while Snip captures, over the real App and stand-ins (spec "Keyboard during a capture",
// edge 16): Bubble turns it into its bubble at once and a bubble click mid-recording brings it back, while Hidden hides
// all of ours but Snip's capture windows for as long as it lasts, the Snip panel too, then brings back only what still
// has something to show; the tray's Snip row opens Snip, and a press on Snip's windows aims the keyboard's letters.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "snipkit.js" as Kit
import "voicekit.js" as VKit
import "fixtures/face.js" as Face
import "fixtures/sel.js" as Sel
import "fixtures/snip.js" as Snip
import "fixtures/ui-en.js" as En
import "fixtures/voice.js" as Voice

Item {
    id: root
    width: 200
    height: 100

    Component {
        id: appC
        App {
            // The hub loads Snip's texts with the keyboard's.
            ui: FakeUi {
                texts: Kit.lang("en").texts
            }
            keyboard: FakeKeyboard {}
            voice: FakeVoice {}
            snip: FakeSnip {}
        }
    }

    TestCase {
        name: "snipaside"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The app over fresh stand-ins, once its keyboard shows, with room for its bubble.
        function make() {
            const app = createTemporaryObject(appC, root);
            app.keyboard.bubble = { x: 40, y: 50 };
            tryVerify(() => app.face.visible, 2000, "the keyboard shows");
            return app;
        }
        // The hub asks the keyboard to step aside as `way` (`bubble`, `hidden`, or null to come back), in `mode`.
        function aside(app, way, mode) {
            app.ui.mode = Object.assign({ away: false, minimised: false }, mode || {}, { aside: way });
            app.ui.asideAsked();
        }
        // Snip records, once its record bar shows, which Qt shows just after its shadow.
        function records(app) {
            app.snip.page = Kit.view("record");
            tryVerify(() => app.snips.item.record.bar.visible, 2000, "the record bar shows");
        }
        // The hub hands over notice `id`.
        function notify(app, id) {
            app.ui.notices = [{ id: id, text: "Notice " + id, retry: false }];
            app.ui.noticesWaiting();
        }
        // The hub's overlay state with Right-click on and the pointer over the keyboard; its outline box; a strip.
        function pointing(app) {
            const f = app.face;
            const p = f.spots.physical(f.x + f.width / 2, f.y + f.height / 2);
            app.ui.assist = Object.assign({}, En.assist, { live: true, rclick: true, mode: "rclick", point: p });
        }
        function scrolls(app) {
            app.ui.scroll = Object.assign({}, En.scroll, { live: true, area: { x: 20, y: 20, w: 300, h: 200 } });
        }
        function selects(app) {
            app.ui.selView = JSON.parse(JSON.stringify(Sel.views.en.both));
        }
        function edges(app) {
            return Array.from({ length: app.outline.edges.count }, (_, i) => app.outline.edges.objectAt(i));
        }
        // True when no show lands on window `w` now: a hushed shadow turns a show into a hide, and Qt holds back a window
        // whose owner is hidden.
        function held(w) {
            return w.hush === true || (!!w.transientParent && !w.transientParent.visible);
        }
        // Fires tip `tip`'s hover wait now, as its timer would.
        function hovered(tip) {
            [...tip.data].find(o => o && o.interval !== undefined && o.triggered !== undefined).triggered();
        }
        // Every window of ours that may show, with its shadow, the Voice panel's and the language list's included.
        function ours(app) {
            const [a, f, p] = [app, app.face, app.dictation.panel];
            const all = [a.face, a.kbGlow, a.bubble, a.bubbleGlow, a.menu, a.menuGlow, a.notice, a.noticeGlow];
            const more = [a.strip, a.strip.chips, a.dictation, a.dictation.bar, Board.tipOf(f), Board.tipOf(a.bubble)];
            const panels = [p, p ? p.pane : null, f.panel, f.panel ? f.panel.list : null];
            return all.concat(more, panels, edges(app)).filter(w => !!w);
        }

        function test_the_app_makes_snips_windows_and_the_tray_row_opens_snip() {
            const app = make();
            verify(app.snips.item, "Snip's windows are made");
            const capture = Object.keys(Snip.capture).length;
            compare(app.snip.heard("shield").length, capture, "each capture window shielded as it is made");
            app.menu.openSnip();
            compare(app.snip.heard("open").length, 1);
        }

        function test_a_press_on_snips_windows_aims_the_keyboards_letters_at_snip() {
            const app = make();
            app.snip.page = Kit.view("strip");
            const strip = app.snips.item.strip.strip;
            tryVerify(() => strip.visible, 2000, "the strip shows");
            waitForRendering(strip.contentItem);
            mouseClick(Kit.button(strip, "edit"));
            compare(app.keyboard.calls.filter(c => c === "aimSnip").length, 1, "App.qml's aim reaches the keyboard");
        }

        function test_bubble_turns_the_keyboard_into_its_bubble_at_once_and_the_end_brings_it_back() {
            const app = make();
            aside(app, "bubble");
            verify(!app.face.visible && !app.kbGlow.visible, "the keyboard hid at once");
            verify(app.bubble.visible, "its bubble shows");
            aside(app, null);
            verify(app.face.visible && !app.bubble.visible, "the keyboard is back as it was");
        }

        function test_a_bubble_click_mid_recording_brings_the_keyboard_back_and_the_record_bar_stays() {
            const app = make();
            aside(app, "bubble");
            records(app);
            app.bubble.clicked();
            verify(app.face.visible, "the keyboard came back");
            aside(app, "bubble");
            verify(app.face.visible, "the aside it already took never hides it again");
            verify(app.snips.item.record.bar.visible, "the record bar still shows");
            aside(app, null);
            verify(app.face.visible);
        }

        function test_hidden_hides_ours_the_overlay_and_the_outline_at_once_but_not_snips_and_the_end_brings_them_back() {
            const app = make();
            notify(app, 1);
            selects(app);
            app.voice.page = JSON.parse(JSON.stringify(Voice.views.en.listening));
            scrolls(app);
            pointing(app);
            tryVerify(() => app.notice.visible && app.strip.chips.visible && app.dictation.bar.visible, 2000, "ours show");
            tryVerify(() => edges(app).length === 4 && edges(app).every(e => e.visible), 2000, "the outline shows");
            tryVerify(() => !!app.pointer.item && app.pointer.item.visible, 2000, "the overlay shows");
            app.ui.trayClicked(300, 500);
            verify(app.menu.visible, "the tray menu is open");
            records(app);
            aside(app, "hidden");
            verify(ours(app).every(w => !w.visible), "nothing of ours shows");
            verify(!app.pointer.item.visible, "nor the pointer overlay");
            verify(app.snips.item.record.bar.visible && app.snips.item.record.visible, "Snip's record bar still shows");
            aside(app, null);
            verify(app.face.visible && app.kbGlow.visible, "the keyboard comes back");
            tryVerify(() => app.notice.visible && app.noticeGlow.visible, 2000, "the notice still waiting");
            tryVerify(() => app.strip.chips.visible && app.strip.visible, 2000, "the strip by the text still selected");
            tryVerify(() => app.dictation.bar.visible && app.dictation.visible, 2000, "the bar still listening");
            tryVerify(() => edges(app).every(e => e.visible) && app.pointer.item.visible, 2000, "the outline and overlay");
            verify(!app.menu.visible && !app.menuGlow.visible, "the tray menu answered a click that has passed");
            verify(!app.aside.hidden);
        }

        function test_while_hidden_lasts_nothing_of_ours_shows_and_the_end_brings_back_only_what_still_shows() {
            const app = make();
            aside(app, "hidden");
            notify(app, 1);
            selects(app);
            app.voice.page = VKit.view("en", "home");
            scrolls(app);
            pointing(app);
            app.ui.trayClicked(300, 500);
            app.ui.showAsked();
            tryVerify(() => !!app.dictation.panel && edges(app).length === 4 && !!app.pointer.item, 2000, "all made");
            app.notice.open();
            const loose = ours(app).filter(w => w.visible || !held(w));
            verify(loose.length === 0, "nothing that asked to show since the hush shows, nor can: " + loose.map(w => w.title));
            verify(!app.pointer.item.visible, "nor the overlay");
            app.notice.answer(false);
            app.ui.selView = null;
            aside(app, "hidden");
            verify(ours(app).every(w => !w.visible), "another ask keeps it so");
            aside(app, null);
            verify(app.face.visible, "the keyboard its Show asked for");
            tryVerify(() => app.dictation.panel.visible && app.dictation.panel.pane.visible, 2000, "the Voice panel");
            tryVerify(() => edges(app).every(e => e.visible) && app.pointer.item.visible, 2000, "the outline and overlay");
            app.notice.open();
            verify(!app.notice.shown, "no notice waits");
            verify(!app.notice.visible && !app.noticeGlow.visible, "a notice closed meanwhile stays gone");
            verify(!app.strip.chips.visible && !app.strip.visible, "a strip whose text went stays gone");
            verify(!app.menu.visible && !app.menuGlow.visible, "the tray menu stays shut");
        }

        function test_hidden_hides_an_open_panel_and_its_tip_and_a_tip_asked_meanwhile_never_shows() {
            const app = make();
            app.keyboard.board = Object.assign({}, Face.board, { lang_key: "list" });
            app.keyboard.state = Face.states.list;
            app.face.redraw(JSON.stringify({ board: true, state: true }));
            tryVerify(() => !!app.face.panel && app.face.panel.list.visible, 2000, "the language list shows");
            const tip = Board.tipOf(app.face);
            app.face.hint("Tip", app.face.panel.list.contentItem);
            tryVerify(() => tip.visible, 2000, "its tip shows");
            aside(app, "hidden");
            verify(!tip.visible && !(app.face.panel && app.face.panel.list.visible), "the list and its tip hide");
            app.face.hint("Tip", app.face.contentItem);
            hovered(tip);
            verify(!tip.visible, "a tip asked meanwhile stays hidden");
            aside(app, null);
            tryVerify(() => !!app.face.panel && app.face.panel.list.visible, 2000, "the list the state holds comes back");
            hovered(tip);
            verify(!tip.visible, "the tip answered a pointer that has moved on");
        }

        function test_hidden_hides_the_snip_panel_and_the_end_brings_it_back() {
            const app = make();
            app.snip.page = Kit.view("panel-capture");
            const panel = app.snips.item.panel;
            tryVerify(() => panel.pane.visible && panel.visible, 2000, "the Snip panel shows");
            verify(app.snips.item.win === app.face, "above the keyboard, at its size");
            aside(app, "hidden");
            verify(!panel.pane.visible && !panel.visible, "it hides with its shadow while Hidden lasts");
            aside(app, null);
            tryVerify(() => panel.pane.visible && panel.visible, 2000, "and comes back");
        }

        function test_the_end_of_hidden_brings_back_a_minimised_keyboard_as_its_bubble() {
            const app = make();
            aside(app, "hidden", { away: true, minimised: true });
            verify(!app.face.visible && !app.bubble.visible);
            aside(app, null, { away: true, minimised: true });
            verify(!app.face.visible && app.bubble.visible, "the bubble the hub kept");
        }
    }
}

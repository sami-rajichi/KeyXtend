// The dictation bar in the app: it pops in over its own shadow, which follows its fade, and the guard puts it in front
// as a pop-up; it is placed by the keyboard's box and placed again as the keyboard moves, it stays at the bottom
// centre while the keyboard hides (edge 15), a press on it closes the tray menu, the tray's Dictation row opens the
// panel, and its words take the Arabic font with an Arabic keyboard.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/voice.js" as Voice
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {}
            voice: FakeVoice {}
        }
    }

    TestCase {
        name: "voicebar_app"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A copy of the hub's view `name` in language `tag` (English by default).
        function view(name, tag) {
            return JSON.parse(JSON.stringify(Voice.views[tag || "en"][name]));
        }
        // The app once its keyboard shows.
        function make() {
            const a = createTemporaryObject(appC, root);
            tryVerify(() => a.face.visible, 2000, "the keyboard shows");
            return a;
        }
        // Shows view `v` in app `a` and returns the bar's window.
        function show(a, v) {
            a.voice.page = v;
            const bar = a.dictation.bar;
            tryVerify(() => bar.visible, 2000, "the bar shows");
            return bar;
        }
        // The newest place ask.
        function lastAsk(a) {
            return a.voice.asks[a.voice.asks.length - 1];
        }
        function stop(bar) {
            return Board.all(bar.contentItem, o => o.Accessible.id === "voice.stop")[0];
        }

        function test_it_pops_in_over_its_shadow_and_the_guard_puts_it_in_front() {
            const a = make();
            const bar = show(a, view("listening"));
            verify(a.dictation.visible, "its shadow shows");
            compare(a.dictation.transientParent, null, "its own window, not the keyboard's");
            verify(bar.body.opacity < 1, "it fades in");
            compare(a.dictation.shade.opacity, bar.body.opacity, "the shadow fades with it");
            tryCompare(bar.body, "opacity", 1, 1000, "fully shown");
            tryVerify(() => a.keyboard.calls.indexOf("guard " + bar.title) >= 0, 2000, "guarded in front");
            a.voice.page = view("hidden");
            verify(!bar.visible && !a.dictation.visible, "both hide");
        }

        function test_it_is_placed_by_the_keyboard_and_again_as_the_keyboard_moves() {
            const a = make();
            show(a, view("listening"));
            compare(JSON.stringify(lastAsk(a).over), a.face.spots.beside(NaN)[0], "by the keyboard's box");
            const asks = a.voice.asks.length;
            a.face.at = Qt.point(a.face.at.x + 30, a.face.at.y);
            verify(a.voice.asks.length > asks, "placed again");
            compare(JSON.stringify(lastAsk(a).over), a.face.spots.beside(NaN)[0], "by its new box");
        }

        // Edge 15: the keyboard hides while it listens; the bar stays, placed with no keyboard, and Stop still acts.
        function test_it_stays_when_the_keyboard_hides() {
            const a = make();
            const v = view("listening");
            const bar = show(a, v);
            a.hide();
            verify(!a.face.visible, "the keyboard hid");
            verify(bar.visible, "the bar stays");
            compare(lastAsk(a).over, null, "placed with no keyboard");
            mouseClick(stop(bar));
            compare(a.voice.calls.filter(c => c.startsWith("act bar")), ["act bar stop 0 " + v.round]);
        }

        function test_a_press_on_it_closes_the_tray_menu() {
            const a = make();
            const bar = show(a, view("listening"));
            a.menu.toggle(300, 500);
            verify(a.menu.visible);
            mousePress(bar.body, 4, 4);
            mouseRelease(bar.body, 4, 4);
            verify(!a.menu.visible, "closed");
        }

        function test_the_trays_dictation_row_opens_the_panel() {
            const a = make();
            a.menu.toggle(300, 500);
            waitForRendering(a.menu.contentItem);
            const row = Board.all(a.menu.contentItem, o => o.Accessible.role === Accessible.MenuItem && o.part === "voice")[0];
            mouseClick(row);
            verify(!a.menu.visible, "the menu closed");
            compare(a.voice.calls, ["open"]);
        }

        function test_an_arabic_keyboard_draws_the_bars_words_in_the_arabic_font() {
            const a = make();
            a.ui.lang = Ar.lang;
            a.ui.texts = Ar.texts;
            const v = view("listening", "ar");
            const bar = show(a, v);
            verify(bar.ar, "the keyboard speaks Arabic");
            const word = Board.all(bar.contentItem, o => o.text === v.bar.word && o.font !== undefined)[0];
            compare(word.font.family, a.face.res.fam.arabic);
            compare(word.font.pixelSize, N.lk.shape.caption.status_px * N.lk.look.arabic_scale * a.face.s);
        }
    }
}

// The selection strip in the app: it pops in with its shadow, a click's tick shows for its time, then the strip and its
// shadow fade and rise and the hub forgets the round; reduced motion skips the pop and the fade, not the tick. It shows
// while the keyboard is minimised, keeps clear of our windows, places itself again as the keyboard moves and reads the
// hub again on new settings (the pill's checks from the spike, P10 Ruling D).
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/sel.js" as Sel
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R

Item {
    id: root
    width: 200
    height: 100

    // The look the next app starts in.
    property var look: N.lk

    Component {
        id: appC
        App {
            ui: FakeUi {
                look: root.look
            }
            keyboard: FakeKeyboard {}
        }
    }

    TestCase {
        name: "selchips_app"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // A copy of the hub's English view `name`, with `more` laid over it.
        function view(name, more) {
            return Object.assign(JSON.parse(JSON.stringify(Sel.views.en[name])), more || {});
        }
        // The app once its keyboard shows, in look `lk` (Native light by default).
        function make(lk) {
            root.look = lk || N.lk;
            const a = createTemporaryObject(appC, root);
            tryVerify(() => a.face.visible, 2000, "the keyboard shows");
            return a;
        }
        // Shows strip `v` in app `a` and returns its window.
        function show(a, v) {
            a.ui.selView = v;
            const strip = a.strip.chips;
            tryVerify(() => strip.visible, 2000, "the strip shows");
            return strip;
        }
        // The Qt box at (`x`, `y`), `w` by `h`, in whole physical pixels on the screen that holds it, as the hub takes it.
        function physical(a, x, y, w, h) {
            const p = a.face.spots.physical(x, y);
            return { x: Math.round(p.x), y: Math.round(p.y), w: Math.round(w * p.dpr), h: Math.round(h * p.dpr) };
        }
        function key(strip, id) {
            return Board.all(strip.contentItem, o => o.chip !== undefined && o.hit !== undefined && o.chip.id === id)[0];
        }
        // The shadow copies the plate's fade, rise and scale, at the look's margin.
        function follows(a) {
            const [body, shade] = [a.strip.chips.body, a.strip.shade];
            compare(shade.opacity, body.opacity, "the shadow fades with the strip");
            compare(shade.y, a.face.lk.margin + body.y, "the shadow rises with the strip");
            compare(shade.scale, body.scale, "and grows with it");
        }

        function test_the_strip_and_its_shadow_pop_in_together() {
            const a = make();
            const strip = show(a, view("both"));
            verify(a.strip.visible, "its shadow shows");
            verify(strip.body.opacity < 1 && strip.body.y > 0, "it fades in and rises as it shows");
            follows(a);
            tryCompare(strip.body, "opacity", 1, 1000, "fully shown");
            follows(a);
            compare(a.strip.shade.scale, 1, "the shadow ends at full size");
            tryVerify(() => a.keyboard.calls.indexOf("guard " + strip.title) >= 0, 2000, "guarded in front");
        }

        // The tick shows for `[shows] chips_done`, then the strip fades and rises with its shadow, and the hub forgets it.
        function test_a_tick_shows_then_the_strip_fades_with_its_shadow() {
            const a = make();
            const strip = show(a, view("both", { round: 2 }));
            tryCompare(strip.body, "opacity", 1, 1000, "popped in");
            waitForRendering(strip.contentItem);
            mouseClick(key(strip, "copy"));
            compare(a.ui.selCalls, ["selAct 2 copy"]);
            a.ui.selView = view("ticked", { round: 2 });
            verify(strip.done.running && strip.visible, "the tick shows");
            compare(strip.done.interval, a.face.lk.motion.shows.chips_done);
            strip.done.triggered();
            tryVerify(() => strip.body.opacity < 1 && strip.body.y < 0, 1000, "it fades and rises");
            follows(a);
            tryVerify(() => !strip.visible, 1000, "then goes");
            verify(!a.strip.visible, "with its shadow");
            compare(a.ui.selCalls, ["selAct 2 copy", "selFaded 2"], "the hub forgets the round");
            a.ui.settings = Object.assign({}, a.ui.settings);
            verify(!strip.visible, "a strip the hub forgot stays gone");
        }

        function test_reduced_motion_skips_the_pop_and_the_fade_but_not_the_tick() {
            const a = make(R.lk);
            const strip = show(a, view("both", { round: 3 }));
            compare([strip.body.opacity, strip.body.y, strip.body.scale], [1, 0, 1], "no pop");
            a.ui.selView = view("ticked", { round: 3 });
            verify(strip.done.running, "the tick still shows");
            compare(strip.done.interval, R.lk.motion.shows.chips_done, "for as long");
            strip.done.triggered();
            verify(!strip.visible && !a.strip.visible, "no fade");
            compare(a.ui.selCalls, ["selFaded 3"]);
        }

        // Not owned by the keyboard's shadow, it shows by the bubble too, and keeps clear of it.
        function test_it_shows_while_the_keyboard_is_minimised() {
            const a = make();
            a.keyboard.bubble = { x: 40, y: 50 };
            verify(a.minimise(), "minimised");
            const strip = show(a, view("both"));
            compare(a.strip.transientParent, null, "its own window");
            const clear = a.ui.selAsks[a.ui.selAsks.length - 1].clear;
            const b = a.bubble;
            compare(JSON.stringify(clear), JSON.stringify([physical(a, b.x, b.y, b.width, b.height)]), "the bubble, not the hidden keyboard");
            verify(strip.visible);
        }

        // A keyboard that moves places the strip again, clear of its new box; new settings read the hub again.
        function test_it_follows_the_keyboard_and_reads_again_on_new_settings() {
            const a = make();
            show(a, view("both"));
            const asks = a.ui.selAsks.length;
            const f = a.face;
            const plate = () => physical(a, f.at.x, f.at.y, f.drawn.width, f.drawn.height);
            compare(JSON.stringify(a.ui.selAsks[asks - 1].clear[0]), JSON.stringify(plate()), "the keyboard");
            f.at = Qt.point(f.at.x + 30, f.at.y);
            verify(a.ui.selAsks.length > asks, "placed again");
            compare(JSON.stringify(a.ui.selAsks[a.ui.selAsks.length - 1].clear[0]), JSON.stringify(plate()), "clear of its new place");
            const reads = a.ui.selReads;
            a.ui.settings = Object.assign({}, a.ui.settings);
            verify(a.ui.selReads > reads, "a new [ui] language wakes the settings, so it reads again");
        }
    }
}

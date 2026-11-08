// The selection strip in the app: it pops in with its shadow, shows a click's tick for its time, then fades and rises
// with its shadow as the hub forgets the round, and reduced motion skips only the pop and the fade. It shows while the
// keyboard is minimised, keeps clear of our windows, places itself again as the keyboard or an open panel moves, reads
// the hub again on new settings (the pill's checks from the spike, P10 Ruling D), and on a large Arabic keyboard draws
// keycaps one character key tall in the theme's Arabic font.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/sel.js" as Sel
import "fixtures/ui-ar.js" as Ar
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
        // The strip's keycaps, and the one for chip `id`.
        function keys(strip) {
            return Board.all(strip.contentItem, o => o.chip !== undefined && o.hit !== undefined);
        }
        function key(strip, id) {
            return keys(strip).find(k => k.chip.id === id);
        }
        // The face's board at `size`, off preset: the module scales every box by the size (kx-mod-keyboard geom.rs).
        function scaled(size) {
            const box = b => Object.assign({}, b, { x: b.x * size, y: b.y * size, w: b.w * size, h: b.h * size });
            const b = Face.board;
            return Object.assign({}, b, { size: size, preset: null, plate: box(b.plate), bar: box(b.bar), dpad: box(b.dpad), keys: b.keys.map(box) });
        }
        // A character key's height on `board`.
        function charOf(board) {
            return board.keys.find(k => !k.side && k.kind === "char").h;
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

        // An open panel that moves places the strip again, clear of the panel's new box.
        function test_it_places_itself_again_as_an_open_panel_moves() {
            const a = make();
            show(a, view("both"));
            a.keyboard.board = Object.assign({}, Face.board, { lang_key: "list" });
            a.keyboard.state = Face.states.list;
            a.face.redraw(JSON.stringify({ board: true, state: true }));
            tryVerify(() => !!a.face.panel && a.face.panel.list.visible, 2000, "the list shows");
            const list = a.face.panel.list;
            const box = () => JSON.stringify(physical(a, list.x, list.y, list.width, list.height));
            const asks = a.ui.selAsks.length;
            list.x = list.x + 30;
            // Windows moves the window, then tells Qt its new place.
            tryVerify(() => a.ui.selAsks.length > asks, 2000, "placed again");
            const clear = a.ui.selAsks[a.ui.selAsks.length - 1].clear;
            verify(clear.some(c => JSON.stringify(c) === box()), "clear of the list's new place: " + JSON.stringify(clear));
        }

        // At size 1.5 in Arabic a keycap is one character key tall, as `SelStrip.rowOf` reads it, and its word takes the
        // theme's Arabic font.
        function test_a_large_arabic_keyboard_draws_keycaps_one_key_row_tall() {
            const a = make();
            a.keyboard.board = scaled(1.5);
            a.face.redraw(JSON.stringify({ board: true, state: false }));
            a.ui.lang = Ar.lang;
            a.ui.texts = Ar.texts;
            compare(a.face.s, 1.5, "the keyboard's size");
            const row = charOf(a.face.board);
            compare(row, charOf(Face.board) * 1.5, "its character keys grew");
            compare(a.strip.rowOf(a.face.board), row, "the strip reads that row");
            const v = JSON.parse(JSON.stringify(Sel.views.ar.both));
            const strip = show(a, v);
            verify(strip.ar, "the keyboard speaks Arabic");
            const caps = keys(strip);
            compare(caps.length, v.chips.length, "every chip");
            for (const k of caps) {
                compare(k.height, row, k.chip.id + ": one key row");
                const w = Board.all(k, o => o.font !== undefined && o.renderType !== undefined)[0];
                compare(w.text, v.chips.find(c => c.id === k.chip.id).word, k.chip.id);
                compare(w.font.family, a.face.res.fam.arabic, k.chip.id + ": the theme's Arabic font");
            }
            compare(a.ui.selAsks[a.ui.selAsks.length - 1].size, 1.5, "the gap grows with it");
        }
    }
}

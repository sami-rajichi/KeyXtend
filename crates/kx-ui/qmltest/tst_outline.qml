// The scroll pad's outline (spec §5.2, mock-up .scroll-target): four thin click-through edge windows inside the target's
// box, or just outside our own list's, in the ring colour, Highlight in high contrast, made only while the hub names a
// box and gone after a stop, cut to the screen holding the target, guarded and laid just under the keyboard; and the
// app's keyboard has its entries watched while the scroll module runs, and its open list asks to be marked as paging.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "pointer.js" as Pointer
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR

Item {
    id: root
    width: 200
    height: 100

    Component { id: uiC; FakeUi {} }
    Component { id: outlineC; Outline {} }
    Component { id: spotsC; Spots {} }
    // The hidden keyboard window the spots belong to; the outline reads only their screens.
    Component {
        id: hostC
        Window {
            flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus
        }
    }
    // A stand-in screen, as tst_overlay's: its corner in physical pixels, its size in Qt units, at its own scale.
    Component {
        id: screenC
        QtObject {
            property real devicePixelRatio: 1
            property int virtualX: 0
            property int virtualY: 0
            property int width: 4000
            property int height: 3000
        }
    }
    // The keyboard window as the outline sees it: its look, texts, spots, title, whether it shows, and its quiet guard.
    Component {
        id: faceC
        QtObject {
            property var lk: N.lk
            property var res: Board.resOf(N.lk, En)
            property var spots: null
            property string title: En.texts["window.keyboard"]
            property bool visible: true
            property int guards: 0
            function guardQuiet() {
                guards++;
            }
        }
    }
    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {}
        }
    }

    TestCase {
        name: "outline"
        when: windowShown

        readonly property real px: N.lk.shape.outline.px

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // An outline over stand-in screens, each `{d, x, y}` (scale and corner), in look `lk`.
        function make(screens, lk) {
            const scs = screens.map(s => createTemporaryObject(screenC, root, { devicePixelRatio: s.d, virtualX: s.x || 0, virtualY: s.y || 0 }));
            const spots = createTemporaryObject(spotsC, root, { win: createTemporaryObject(hostC, root), screens: scs });
            const face = createTemporaryObject(faceC, root, { spots: spots, lk: lk || N.lk });
            const ui = createTemporaryObject(uiC, root);
            const o = createTemporaryObject(outlineC, root, { ui: ui, face: face });
            tryCompare(ui, "scrollReads", 1, 2000, "read once at start");
            return { o: o, ui: ui, face: face };
        }
        // The hub names box `area` (physical pixels, or null), ours or not, while the module is live.
        function aim(t, area, ours) {
            t.ui.scroll = Object.assign({}, En.scroll, { live: !!area, area: area, ours: !!ours });
        }
        // The shown edge windows, the real pointer passing through them.
        function edges(o) {
            const all = Array.from({ length: o.edges.count }, (_, i) => o.edges.objectAt(i));
            Pointer.through(all);
            return all;
        }
        // Edges as whole boxes, top, bottom, left and right, in Qt units.
        function boxes(o) {
            return edges(o).map(e => [e.x, e.y, e.width, e.height]);
        }
        function frame(x, y, w, h) {
            return [[x, y, w, px], [x, y + h - px, w, px], [x, y + px, px, h - 2 * px], [x + w - px, y + px, px, h - 2 * px]];
        }
        function at(o, x, y, w, h) {
            return JSON.stringify(boxes(o)) === JSON.stringify(frame(x, y, w, h));
        }
        // Shown edges on box `a`, faded in; each new place they take notes how much of them showed, and each window
        // change, which Windows reports a frame later, is noted too.
        function settled(t, a) {
            aim(t, a);
            tryCompare(t.o.edges, "count", 4, 2000);
            const all = edges(t.o), seen = [], moved = [];
            for (const e of all) {
                tryCompare(e.paint, "opacity", 1, 2000, "faded in");
                e.spotChanged.connect(() => seen.push(e.paint.opacity));
                for (const s of [e.xChanged, e.yChanged, e.widthChanged, e.heightChanged])
                    s.connect(() => moved.push(e));
            }
            return { all: all, seen: seen, moved: moved, reads: t.ui.scrollReads };
        }

        function test_no_edge_window_until_the_hub_names_a_box_and_none_after_a_stop() {
            const t = make([{ d: 1 }]);
            compare(t.o.edges.count, 0, "none at start");
            aim(t, { x: 100, y: 100, w: 400, h: 300 });
            tryCompare(t.o.edges, "count", 4, 2000, "four edges");
            compare(t.ui.scrollReads, 2, "one read for the change");
            aim(t, null);
            tryCompare(t.o.edges, "count", 0, 2000, "freed after a stop");
        }

        function test_four_click_through_edges_in_the_ring_colour_inside_the_box() {
            for (const d of [1, 1.5]) {
                const t = make([{ d: d }]);
                aim(t, { x: 150 * d, y: 120 * d, w: 600 * d, h: 450 * d });
                tryCompare(t.o.edges, "count", 4, 2000);
                compare(boxes(t.o), frame(150, 120, 600, 450), "inside the box at " + d);
                for (const e of edges(t.o)) {
                    for (const f of [Qt.Tool, Qt.FramelessWindowHint, Qt.WindowStaysOnTopHint, Qt.WindowDoesNotAcceptFocus, Qt.WindowTransparentForInput])
                        verify((e.flags & f) === f, "flag " + f);
                    compare(e.transientParent, null, "owned by none, so it can lie under the keyboard");
                    compare(e.title, En.texts["window.outline"]);
                    verify(Qt.colorEqual(e.paint.color, N.lk.palette.ring), "the ring colour");
                    tryCompare(e.paint, "opacity", 1, 2000, "faded in");
                }
            }
        }

        // Our own open list is opaque and lies over the edges, so its outline goes just outside its box, still
        // click-through, never focused and laid under the keyboard; another app's box is outlined inside again.
        function test_our_own_window_is_outlined_just_outside_its_box() {
            for (const d of [1, 1.5]) {
                const t = make([{ d: d }]);
                const area = { x: 150 * d, y: 120 * d, w: 600 * d, h: 450 * d };
                aim(t, area, true);
                tryCompare(t.o.edges, "count", 4, 2000);
                compare(boxes(t.o), frame(150 - px, 120 - px, 600 + 2 * px, 450 + 2 * px), "just outside at " + d);
                const shut = Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput;
                verify(edges(t.o).every(e => (e.flags & shut) === shut && e.transientParent === null), "click-through, owned by none");
                tryVerify(() => t.ui.calls.indexOf("layUnder " + En.texts["window.outline"] + " " + t.face.title) >= 0, 2000);
                aim(t, area, false);
                tryVerify(() => at(t.o, 150, 120, 600, 450), 2000, "inside another app's box at " + d);
            }
        }

        function test_high_contrast_outlines_in_highlight() {
            const t = make([{ d: 1 }], NC.lk);
            aim(t, { x: 10, y: 10, w: 100, h: 100 });
            tryCompare(t.o.edges, "count", 4, 2000);
            verify(edges(t.o).every(e => Qt.colorEqual(e.paint.color, NC.lk.hover_line)));
        }

        // Partly off-screen, it outlines the part on the screen holding most of it; wholly off, nothing.
        function test_a_target_partly_off_screen_is_cut_to_its_screen() {
            const t = make([{ d: 1 }, { d: 2, x: 4000 }]);
            aim(t, { x: -100, y: 2800, w: 600, h: 400 });
            tryCompare(t.o.edges, "count", 4, 2000);
            compare(boxes(t.o), frame(0, 2800, 500, 200), "on the main screen");
            aim(t, { x: 3800, y: 100, w: 1200, h: 400 });
            tryVerify(() => t.o.edges.count === 4 && edges(t.o)[0].x === 4000, 2000);
            compare(boxes(t.o), frame(4000, 50, 500, 200), "on the scaled screen holding most of it");
            aim(t, { x: -900, y: 100, w: 400, h: 300 });
            tryCompare(t.o.edges, "count", 0, 2000, "off every screen");
        }

        // A new box moves the same windows; each new set joins the guard's quiet run and lies just under the keyboard,
        // and again when the keyboard shows again.
        function test_the_edges_move_join_the_guard_and_lie_under_the_keyboard() {
            const t = make([{ d: 1 }]);
            const laid = () => t.ui.calls.filter(c => c === "layUnder " + En.texts["window.outline"] + " " + t.face.title).length;
            aim(t, { x: 100, y: 100, w: 400, h: 300 });
            tryCompare(t.o.edges, "count", 4, 2000);
            tryVerify(() => laid() === 1 && t.face.guards === 1, 2000, "guarded and laid under once");
            const first = edges(t.o);
            aim(t, { x: 200, y: 150, w: 300, h: 200 });
            tryVerify(() => edges(t.o)[0].x === 200, 2000);
            verify(edges(t.o).every((e, i) => e === first[i]), "moved, not made again");
            t.face.visible = false;
            t.face.visible = true;
            tryCompare(t.o.edges, "count", 4, 2000);
            tryVerify(() => laid() === 2, 2000, "under the keyboard shown again");
        }

        // A new box fades the edges out with the outline's move, moves them while unseen, and fades them back in.
        function test_a_moved_box_fades_out_moves_unseen_and_fades_back_in() {
            const t = make([{ d: 1 }]), s = settled(t, { x: 100, y: 100, w: 400, h: 300 });
            aim(t, { x: 200, y: 150, w: 300, h: 200 });
            tryCompare(t.ui, "scrollReads", s.reads + 1, 2000);
            verify(s.all.every(e => e.moving), "each edge fades out first");
            verify(at(t.o, 100, 100, 400, 300), "still in place while it shows");
            tryVerify(() => at(t.o, 200, 150, 300, 200) && s.all.every(e => e.paint.opacity === 1 && !e.moving), 2000);
            compare(s.seen, [0, 0, 0, 0], "each moved once, while unseen");
            verify(s.moved.length > 0, "the windows moved");
        }

        // Each read makes a new box; one with the same values moves and fades nothing.
        function test_an_unchanged_box_moves_nothing() {
            const t = make([{ d: 1 }]), s = settled(t, { x: 100, y: 100, w: 400, h: 300 });
            aim(t, { x: 100, y: 100, w: 400, h: 300 });
            tryCompare(t.ui, "scrollReads", s.reads + 1, 2000);
            verify(s.all.every(e => !e.moving && e.paint.opacity === 1), "still shown");
            compare([s.seen.length, s.moved.length], [0, 0], "not moved");
        }

        // With reduced motion the edges move at once, and show as they are made.
        function test_reduced_motion_moves_the_edges_at_once() {
            const t = make([{ d: 1 }], NR.lk);
            verify(NR.lk.motion.moves.outline.ms === 0);
            const s = settled(t, { x: 100, y: 100, w: 400, h: 300 });
            aim(t, { x: 200, y: 150, w: 300, h: 200 });
            tryCompare(t.ui, "scrollReads", s.reads + 1, 2000);
            verify(at(t.o, 200, 150, 300, 200), "moved at the read");
            verify(s.all.every(e => !e.moving && e.paint.opacity === 1), "never faded");
        }

        // In the app, the outline follows the hub's scroll state, and the keyboard's entries are watched only while the
        // scroll module runs: asked at each guard try until it takes, and stopped with the module.
        function test_the_app_outlines_and_its_keyboard_watches_entries_only_while_the_module_runs() {
            const a = createTemporaryObject(appC, root);
            a.ui.notYet = 1;
            tryVerify(() => a.face.visible, 2000, "the keyboard shows");
            Pointer.through([a.face, a.bubble]);
            const count = call => a.ui.calls.filter(c => c === call + " " + a.face.title).length;
            const tries = () => a.keyboard.calls.filter(c => c === "guard").length;
            const run = () => {
                const before = tries();
                a.face.guardRun();
                tryVerify(() => tries() > before, 5000, "the guard tried again");
            };
            tryVerify(() => tries() > 0, 5000, "the guard ran");
            compare(count("watchEntries"), 0, "not while the module is off");
            a.ui.scroll = Object.assign({}, En.scroll, { live: true });
            tryVerify(() => count("watchEntries") === 1, 5000, "asked once the module runs");
            verify(!a.face.watched, "not shown yet");
            run();
            tryCompare(a.face, "watched", true, 5000, "asked again at the next guard run");
            run();
            compare(count("watchEntries"), 2, "once it took");
            a.ui.scroll = Object.assign({}, En.scroll, { live: true, area: { x: 20, y: 20, w: 300, h: 200 } });
            tryCompare(a.outline.edges, "count", 4, 2000);
            tryVerify(() => a.ui.calls.indexOf("layUnder " + En.texts["window.outline"] + " " + a.face.title) >= 0, 2000);
            Pointer.through(edges(a.outline));
            a.ui.scroll = En.scroll;
            tryCompare(a.outline.edges, "count", 0, 2000);
            tryCompare(a.face, "watched", false, 2000, "the module stopped");
            compare(count("unwatchEntries"), 1);
            run();
            compare(count("watchEntries"), 2, "never while the module is off");
        }

        // The app's open list asks for its page mark as it opens, then at each guard try until it holds, the module
        // running or not, so a pad tap at once turns its page instead of passing over it.
        function test_the_apps_open_list_asks_for_its_page_mark_until_it_holds() {
            const a = createTemporaryObject(appC, root);
            a.ui.unpaged = 1;
            tryVerify(() => a.face.visible, 2000, "the keyboard shows");
            Pointer.through([a.face, a.bubble]);
            const count = () => a.ui.calls.filter(c => c === "markPages " + En.texts["window.list"]).length;
            const tries = () => a.keyboard.calls.filter(c => c === "guard").length;
            const run = () => {
                const before = tries();
                a.face.guardRun();
                tryVerify(() => tries() > before, 5000, "the guard tried again");
            };
            // Each guard try that finds the list shown joins the calls log, so their order shows which asked first.
            const shown = () => !!a.face.panel && a.face.panel.list.visible;
            a.keyboard.guarding.connect(() => shown() && a.ui.note("guard"));
            a.keyboard.board = Object.assign({}, Face.board, { lang_key: "list" });
            a.keyboard.state = Face.states.list;
            a.face.redraw(JSON.stringify({ board: true, state: true }));
            tryVerify(shown, 2000, "the list shows");
            tryVerify(() => a.ui.calls.indexOf("guard") >= 0, 5000, "the guard tried with the list shown");
            const mark = a.ui.calls.indexOf("markPages " + En.texts["window.list"]);
            verify(mark >= 0 && mark < a.ui.calls.indexOf("guard"), "asked as it opened, before the guard's try");
            tryCompare(a.face.panel.list, "paged", true, 5000, "asked again at the try");
            compare(count(), 2, "refused as it opened, held at the try");
            run();
            compare(count(), 2, "once it held");
        }
    }
}

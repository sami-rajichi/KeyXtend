// The pointer overlay inside the app (spec §5.3): made only while the hub names a mode or the pad scrolls, read once
// soon after each wake with no frame needed and at most once per `poll_ms`, its badge hidden over each of our windows,
// whose guard run keeps a shown tip, and its window grown only for a wider badge word as the keyboard's language changes.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-ar.js" as Ar
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "pointer.js" as Pointer

Item {
    id: root
    width: 200
    height: 100

    Component {
        id: appC
        App {
            ui: FakeUi {}
            keyboard: FakeKeyboard {}
        }
    }

    SignalSpy {
        id: widths
        signalName: "widthChanged"
    }
    SignalSpy {
        id: heights
        signalName: "heightChanged"
    }

    TestCase {
        name: "overlayApp"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The hub's state: Right-click on, the pointer at physical `p`, then `extra`.
        function state(p, extra) {
            return Object.assign({}, En.assist, { live: true, rclick: true, mode: "rclick", point: p }, extra || {});
        }
        // The hub's state with no mode on, numbered `seq`.
        function numbered(seq) {
            return Object.assign({}, En.assist, { seq: seq });
        }
        // The app over fresh stand-ins with its `props`, once its keyboard shows and the state was read at start; the
        // real pointer passes through its keyboard.
        function app(props) {
            const a = createTemporaryObject(appC, root, props || {});
            tryVerify(() => a.face.visible && a.assist !== null, 2000, "the keyboard shows");
            Pointer.through([a.face, a.bubble]);
            return a;
        }
        // Physical points over the middle of window `w`, and as far again to its left, clear of it.
        function over(a, w) {
            return a.face.spots.physical(w.x + w.width / 2, w.y + w.height / 2);
        }
        function beside(a, w) {
            return a.face.spots.physical(w.x - w.width / 2, w.y + w.height / 2);
        }
        // Every bundled font the stand-in loads is in, so no font load widens a word later.
        function fontsIn(a) {
            const l = a.ui.loaders, all = Array.from({ length: l.count }, (_, i) => l.objectAt(i));
            return all.length > 0 && all.every(f => f.status === FontLoader.Ready);
        }
        // Switches the keyboard's language to `t`, a ui-*.js fixture, as the hub does.
        function say(a, t) {
            a.ui.lang = t.lang;
            a.ui.texts = t.texts;
        }
        // The overlay, made in language `t` with a mode on and its size spies set.
        function madeIn(a, t, on) {
            say(a, t);
            a.ui.assist = on;
            tryVerify(() => a.pointer.item !== null, 2000, "made");
            for (const spy of [widths, heights]) {
                spy.clear();
                spy.target = a.pointer.item;
            }
            return a.pointer.item;
        }
        // Frees the overlay with every mode off.
        function free(a, on) {
            a.ui.assist = Object.assign({}, on, { rclick: false, mode: null });
            tryCompare(a.pointer, "item", null, 2000, "freed");
        }
        // Opens the List design's language list, as tst_app does, and answers it through the alias the overlay reads.
        function openList(a) {
            a.keyboard.board = Object.assign({}, Face.board, { lang_key: "list" });
            a.keyboard.state = Face.states.list;
            a.face.redraw(JSON.stringify({ board: true, state: true }));
            tryVerify(() => !!a.face.panel && a.face.panel.list.visible, 2000, "the list shows");
            return a.face.panel.list;
        }

        function test_the_overlay_is_made_only_while_a_mode_is_on_and_freed_after() {
            const a = app();
            compare(a.pointer.item, null, "no mode, no ring");
            const on = state(null);
            a.ui.assist = on;
            tryVerify(() => a.pointer.item !== null, 2000, "made with Right-click on");
            a.ui.assist = Object.assign({}, on, { rclick: false, grab: "armed", mode: "grab" });
            tryVerify(() => a.assist.grab === "armed", 2000);
            verify(a.pointer.item !== null, "kept while Grab is on");
            a.ui.assist = Object.assign({}, on, { live: false, mode: null });
            tryCompare(a.pointer, "item", null, 2000, "freed when the mouse module stops");
            a.ui.assist = on;
            tryVerify(() => a.pointer.item !== null, 2000);
            a.ui.assist = Object.assign({}, on, { rclick: false, mode: null });
            tryCompare(a.pointer, "item", null, 2000, "freed with every mode off");
            a.ui.assist = on;
            tryVerify(() => a.pointer.item !== null, 2000);
            a.ui.assist = Object.assign({}, on, { mode: null });
            tryCompare(a.pointer, "item", null, 2000, "the hub's mode alone decides");
        }

        // The pad scrolling with no mode on makes the overlay too, for the badge's arrow and speed, and frees it after.
        function test_the_overlay_is_made_while_the_pad_scrolls_and_freed_after() {
            const a = app(), p = beside(a, a.face);
            const scrolling = Object.assign({}, En.assist, { live: true, point: p, scroll: { dir: "down", fast: false } });
            a.ui.assist = scrolling;
            tryVerify(() => a.pointer.item !== null && a.pointer.item.badge.shown, 2000, "made with no mode on");
            const b = a.pointer.item.badge;
            compare(b.scroll.dir, "down", "it shows the scroll part");
            compare(b.Accessible.description, En.texts["badge.slow"], "and its speed");
            a.ui.assist = Object.assign({}, scrolling, { scroll: null, point: null });
            tryCompare(a.pointer, "item", null, 2000, "freed once the pad stops");
        }

        // With the keyboard away no window of ours draws, yet a wake reads on the event loop's next pass: wait(0) runs
        // that pass once and never sleeps, too soon for any frame or timer.
        function test_a_wake_reads_on_the_next_pass_with_no_frame() {
            let t = 1000;
            const a = app({ now: () => t });
            a.hide();
            t += a.face.cfg.pollMs;
            const n = a.ui.assistReads;
            a.ui.assist = numbered(6);
            a.ui.assist = numbered(7);
            compare(a.ui.assistReads, n, "never in the wake's handler");
            wait(0);
            compare(a.ui.assistReads, n + 1, "two wakes in one pass read once");
            compare(a.assist.seq, 7, "the newest state");
            verify(!a.assistRead.pace.running, "no timer");
        }

        // While changes stream, a wake within poll_ms of the last read waits out the rest of it, more wakes add
        // nothing, and the one read then takes the newest state; after it nothing runs.
        function test_wakes_within_the_pace_wait_out_its_rest_and_read_once() {
            let t = 1000;
            const a = app({ now: () => t }), ms = a.face.cfg.pollMs;
            const n = a.ui.assistReads;
            t += ms / 4;
            a.ui.assist = numbered(1);
            a.ui.assist = numbered(2);
            wait(0);
            compare(a.ui.assistReads, n, "held within the pace");
            compare(a.assistRead.pace.interval, ms * 3 / 4, "for the rest of it");
            a.ui.assist = numbered(3);
            tryCompare(a.ui, "assistReads", n + 1, 2000, "read once the pace is up");
            compare(a.assist.seq, 3, "the newest state");
            verify(!a.assistRead.pace.running, "then nothing runs");
        }

        // The hub wakes once until QML reads; that read lets the next change wake, so no change is lost.
        function test_a_change_after_a_read_wakes_again() {
            let t = 1000;
            const a = app({ now: () => t }), ms = a.face.cfg.pollMs;
            const n = a.ui.assistReads;
            t += ms;
            a.ui.change(numbered(1));
            a.ui.change(numbered(2));
            wait(0);
            compare([a.ui.assistReads, a.assist.seq], [n + 1, 2], "one wake, one read of the newest");
            t += ms;
            a.ui.change(numbered(3));
            wait(0);
            compare([a.ui.assistReads, a.assist.seq], [n + 2, 3], "the read let it wake again");
            a.ui.change(numbered(4));
            tryCompare(a.ui, "assistReads", n + 3, 2000, "and again within the pace");
            compare(a.assist.seq, 4);
        }

        // Its guard run is a quiet one: a key's tip the hand is reading stays, and no pop-up loses its place in front.
        function test_the_badge_hides_over_the_keyboard_and_its_guard_keeps_a_shown_tip() {
            const a = app(), f = a.face, tip = Board.tipOf(f);
            const guards = () => a.keyboard.calls.filter(c => c.indexOf("guard") === 0).length;
            tryVerify(() => guards() > 0, 2000, "the keyboard's own run");
            f.hint(En.texts["key.rclick"], f.contentItem);
            tryVerify(() => tip.visible, 2000, "a key's tip shows");
            const n = guards();
            a.ui.assist = state(f.spots.physical(f.x + f.width / 2, f.y + f.height / 2));
            tryVerify(() => a.pointer.item !== null && a.pointer.item.visible, 2000);
            const o = a.pointer.item;
            verify(!o.badge.shown, "over the keyboard");
            tryVerify(() => guards() > n, 2000, "guarded once shown");
            verify(tip.visible, "the tip stays");
            compare(a.keyboard.calls.indexOf("guard " + En.texts["window.pointer"]), -1, "never put in front");
            a.ui.assist = state(f.spots.physical(f.x + f.width / 2, f.y - f.height));
            tryVerify(() => o.badge.shown, 2000, "beside it");
        }

        // A resize stalls Qt's frames, so the badge word's room only grows while a mode is on, from each word once set:
        // a narrower word keeps the window's size, a wider one grows it once, and back and forth then never resizes it.
        function test_the_window_grows_only_for_a_wider_badge_word() {
            const a = app(), on = state(beside(a, a.face));
            tryVerify(() => fontsIn(a), 5000, "the bundled fonts load");
            const o = madeIn(a, En, on);
            const words = [En, Fr, Ar].map(t => {
                say(a, t);
                return { t: t, w: o.badge.widest };
            }).sort((p, q) => p.w - q.w);
            const [narrow, mid, wide] = words.map(word => word.t);
            verify(words[2].w > words[0].w + 1, "the words differ, so the test means something");
            free(a, on);
            const first = madeIn(a, wide, on), size = [first.width, first.height];
            say(a, narrow);
            wait(0);
            compare([first.width, first.height], size, "a narrower word keeps the size");
            compare(widths.count + heights.count, 0);
            free(a, on);
            const next = madeIn(a, narrow, on), small = [next.width, next.height];
            say(a, wide);
            wait(0);
            verify(next.width > small[0] && next.height === small[1], "a wider word grows it");
            compare([widths.count, heights.count], [1, 0], "once");
            const grown = [next.width, next.height];
            for (const t of [narrow, mid, wide, mid, narrow, wide]) {
                say(a, t);
                wait(0);
            }
            compare([next.width, next.height], grown, "back and forth keeps it");
            compare([widths.count, heights.count], [1, 0], "never resized again");
        }

        // The minimise bubble and the open language list hide the badge too, each shown in turn.
        function test_the_badge_hides_over_the_bubble_and_the_open_language_list() {
            const a = app();
            a.keyboard.bubble = { x: 40, y: 50 };
            verify(a.minimise() && a.bubble.visible, "minimised to the bubble");
            a.ui.assist = state(beside(a, a.bubble));
            tryVerify(() => a.pointer.item !== null && a.pointer.item.badge.shown, 2000, "beside the bubble");
            const o = a.pointer.item;
            a.ui.assist = state(over(a, a.bubble));
            tryVerify(() => !o.badge.shown, 2000, "over the bubble");
            a.open();
            const list = openList(a);
            a.ui.assist = state(beside(a, list));
            tryVerify(() => o.badge.shown, 2000, "beside the list");
            a.ui.assist = state(over(a, list));
            tryVerify(() => !o.badge.shown, 2000, "over the list");
        }
    }
}

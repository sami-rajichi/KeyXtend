// The pointer overlay (spec §3.2, §5.3): the badge per mode, beside the pointer and clear of its size, flipped inside the
// work area at a 1.0 and a 1.5 screen ratio, hidden over our windows; the ring at the press, once per seq; and one
// click-through window, made only while a mode is on and read once on the frame after each wake.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/ui-en.js" as En
import "fixtures/ui-ar.js" as Ar
import "pointer.js" as Pointer
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC

Item {
    id: root
    width: 200
    height: 100

    Component { id: badgeC; Badge {} }
    Component { id: overC; Overlay {} }
    Component { id: spotsC; Spots {} }
    // The hidden keyboard window the spots belong to; the overlay reads only their screens.
    Component {
        id: hostC
        Window {
            flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus
        }
    }
    // The smallest stand-in screen, as tst_scale's: its corner at (0, 0), 4000 by 3000 Qt units, at the test's scale.
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
    // A stand-in for one of our windows, in Qt units; `boxes` cut it as the keyboard's plate and corner cut it.
    Component {
        id: oursC
        QtObject {
            property real x: 300
            property real y: 200
            property real width: 400
            property real height: 150
            property bool visible: true
            property var boxes: undefined
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
        name: "overlay"
        when: windowShown

        // The badge's tokens, the two screen ratios each placement runs at (ruling 9), and the stand-in taskbar's height.
        readonly property var sh: N.lk.shape.badge
        readonly property var ratios: [1, 1.5]
        readonly property int bar: 48

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function find(item, ok) {
            return Board.all(item, ok)[0] || null;
        }
        // The hub's state: Right-click on, the pointer at physical `p` in work area `area`, then `extra`.
        function state(p, area, extra) {
            return Object.assign({}, En.assist, { live: true, rclick: true, point: p, area: area }, extra || {});
        }
        // Physical pixels for Qt point (x, y) on a stand-in screen at ratio `d`.
        function px(x, y, d) {
            return { x: x * d, y: y * d };
        }
        // Spots over one stand-in screen at ratio `d`, and its work area in pixels: the screen less the taskbar.
        function screen(d) {
            const sc = createTemporaryObject(screenC, root, { devicePixelRatio: d });
            const spots = createTemporaryObject(spotsC, root, { win: createTemporaryObject(hostC, root), screens: [sc] });
            return { spots: spots, area: { x: 0, y: 0, w: sc.width * d, h: (sc.height - bar) * d } };
        }
        function overlay(s, st, ours) {
            const res = Board.resOf(N.lk, En);
            return createTemporaryObject(overC, root, { lk: N.lk, res: res, spots: s.spots, ours: ours || [], st: st });
        }
        function badge(ui, grab) {
            const res = Board.resOf(N.lk, ui);
            return createTemporaryObject(badgeC, root, { lk: N.lk, res: res, rclick: grab === "off", grab: grab, shown: true });
        }
        // The badge's text and its pill.
        function word(b) {
            return find(b, o => o.font !== undefined && o.text !== undefined);
        }
        function pill(b) {
            return find(b, o => o.radius !== undefined && o.color !== undefined && Qt.colorEqual(o.color, N.lk.palette.badge_bg));
        }
        // The app over fresh stand-ins, once its keyboard shows and the state was read at start; the real pointer passes
        // through its keyboard.
        function app() {
            const a = createTemporaryObject(appC, root);
            tryVerify(() => a.face.visible && a.assist !== null, 2000, "the keyboard shows");
            Pointer.through([a.face, a.bubble]);
            return a;
        }

        function test_each_mode_shows_its_icon_and_carrying_adds_the_word() {
            for (const [grab, mode, said] of [["off", "rclick", false], ["armed", "grab", false], ["carrying", "grab", true]]) {
                const b = badge(En, grab);
                const icon = find(b, o => o.icon !== undefined && o.tint !== undefined);
                compare(icon.icon, N.lk.icons.keys[mode], grab);
                verify(Qt.colorEqual(icon.tint, N.lk.palette.badge_ink), grab);
                compare(word(b).visible, said, grab);
                compare(b.Accessible.name, En.texts["label." + mode], "readers hear the mode");
                compare(b.Accessible.description, said ? En.texts["badge.holding"] : "");
                const wide = said ? sh.gap_px + word(b).implicitWidth : 0;
                fuzzyCompare(b.width, 2 * sh.pad_px[1] + sh.icon_px + wide, 0.01, grab);
                compare(b.height, sh.icon_px + 2 * sh.pad_px[0], grab);
            }
        }

        // A pill (mock-up .badge, border-radius 999px) in the badge colours; its word is in the keyboard's language.
        function test_the_badge_is_a_pill_in_its_colours_and_speaks_the_keyboards_language() {
            const b = badge(Ar, "carrying");
            compare(pill(b).radius, b.height / 2);
            compare(word(b).text, Ar.texts["badge.holding"]);
            compare(word(b).font.family, Board.resOf(N.lk, Ar).fam.ui);
            verify(Qt.colorEqual(word(b).color, N.lk.palette.badge_ink));
            b.shown = false;
            tryCompare(b, "opacity", 0, 2000, "it fades out");
        }

        // Its own small shadow (mock-up .badge box-shadow), not the windows' wide one; high contrast drops it.
        function test_the_badge_casts_its_own_small_shadow() {
            const fields = l => ["inset", "x", "y", "blur", "spread", "color"].map(k => String(l[k])).join(" ");
            for (const look of [N.lk, NC.lk]) {
                const b = createTemporaryObject(badgeC, root, { lk: look, res: Board.resOf(look, En), rclick: true, grab: "off", shown: true });
                const shades = Board.all(b, o => o.shadow !== undefined && o.modelData !== undefined).map(o => fields(o.shadow));
                compare(shades, look.palette.badge_shadow.map(fields), look.contrast ? "contrast" : "native");
            }
            verify(N.lk.palette.badge_shadow.length > 0 && NC.lk.palette.badge_shadow.length === 0);
        }

        // Windows draws the pointer at the screen's scale, so the badge clears it in Qt units at every ratio.
        function test_the_badge_sits_beside_the_pointer_and_a_big_pointer_pushes_it_out() {
            for (const d of ratios) {
                const s = screen(d);
                for (const grow of [1, 4]) {
                    const pointer = En.assist.pointerBase * grow;
                    const o = overlay(s, state(px(1000, 800, d), s.area, { pointerPx: pointer }));
                    const tag = "ratio " + d + ", pointer " + pointer;
                    fuzzyCompare(o.badgeAt.x - 1000, sh.offset_px[0] * grow, 0.001, tag);
                    fuzzyCompare(o.badgeAt.y - 800, sh.offset_px[1] * grow, 0.001, tag);
                    fuzzyCompare((o.badgeAt.x - o.at.x) * d / (pointer * d), sh.offset_px[0] / En.assist.pointerBase, 0.001, tag);
                }
            }
        }

        // Flipped, it keeps the base offset on the other side, where the pointer's body is not; over the taskbar it also
        // stays above it.
        function test_near_the_right_and_bottom_edges_the_badge_flips_and_stays_in_the_work_area() {
            const [w, h, e, near] = [4000, 3000 - bar, sh.edge_px, 20];
            const [ox, oy] = sh.offset_px;
            const spots = [[1000, 800, false, false], [w - near, 800, true, false], [1000, h - near, false, true],
                           [w - near, h - near, true, true], [1000, h + bar / 2, false, true]];
            for (const [d, grow] of [[1, 1], [1.5, 1], [1.5, 4]]) {
                const s = screen(d);
                for (const [x, y, left, up] of spots) {
                    const o = overlay(s, state(px(x, y, d), s.area, { pointerPx: En.assist.pointerBase * grow }));
                    const b = Qt.rect(o.badgeAt.x, o.badgeAt.y, o.badge.width, o.badge.height);
                    const tag = "ratio " + d + ", grow " + grow + " at " + x + ", " + y;
                    const [gx, gy, bottom] = [b.x - x - ox * grow, b.y - y - oy * grow, b.y + b.height];
                    verify(left ? Math.abs(b.x + b.width - (x - ox)) < 0.001 : gx >= -0.001, tag + (left ? ": left" : ": right"));
                    verify(up ? (y > h ? bottom <= y - oy : Math.abs(bottom - (y - oy)) < 0.001) : gy >= -0.001, tag + ": up or down");
                    verify(b.x >= e && b.x + b.width <= w - e, tag + ": inside across");
                    verify(b.y >= e && b.y + b.height <= h - e, tag + ": inside down");
                }
            }
        }

        // The ring still shows there: a hold over our window never starts, so one that rings is not over it.
        function test_the_badge_hides_over_our_shown_windows_cut_to_their_boxes() {
            const s = screen(1);
            const w = createTemporaryObject(oursC, root);
            const ring = { x: 350, y: 250, afterMs: 0, waveMs: 100, seq: 1 };
            const o = overlay(s, state(px(350, 250, 1), s.area, { ring: ring }), [w, null]);
            verify(!o.badge.shown && o.ring.busy, "over our window");
            o.st = state(px(100, 100, 1), s.area);
            verify(o.badge.shown, "outside it");
            tryCompare(o.badge, "opacity", 1, 2000, "it fades in");
            o.st = state(px(350, 250, 1), s.area);
            verify(!o.badge.shown);
            w.visible = false;
            verify(o.badge.shown, "a hidden window hides nothing");
            w.visible = true;
            w.boxes = [{ x: 0, y: 0, w: 40, h: 40 }];
            verify(o.badge.shown, "outside its boxes, as beside the keyboard's plate");
            o.st = state(px(310, 210, 1), s.area);
            verify(!o.badge.shown, "inside one");
        }

        function test_the_ring_centres_on_the_press_and_plays_once_per_seq() {
            const s = screen(1.5);
            const old = { x: 15, y: 15, seq: 3 };
            const ring = { x: 1500, y: 1200, afterMs: 0, waveMs: 100, seq: 5 };
            const at = (extra, p) => state(px(p || 1003, 801, 1.5), s.area, Object.assign({ burst: old }, extra));
            const o = overlay(s, at({}));
            verify(!o.ring.busy, "a burst made before the overlay never plays");
            o.st = at({ ring: ring });
            verify(o.ring.waves.running, "a hold plays");
            fuzzyCompare(o.frame.x + o.ring.x + o.ring.width / 2, 1000, 0.001);
            fuzzyCompare(o.frame.y + o.ring.y + o.ring.height / 2, 800, 0.001);
            tryVerify(() => !o.ring.waves.running, 2000, "once");
            o.st = at({ ring: ring }, 1004);
            verify(!o.ring.waves.running, "the same seq never plays again");
            o.st = at({ ring: Object.assign({}, ring, { seq: 6 }) });
            verify(o.ring.waves.running, "a new hold does");
            o.st = at({});
            verify(!o.ring.busy, "a cleared ring hides at once");
            const burst = { x: 1500, y: 1200, seq: 9 };
            o.st = at({ burst: burst });
            verify(o.ring.busy, "a fire bursts");
            tryVerify(() => !o.ring.busy, 2000);
            o.st = at({ burst: burst }, 1005);
            verify(!o.ring.busy, "once, though the burst stays in the state");
        }

        function test_one_click_through_window_on_top_that_never_takes_focus() {
            const o = overlay(screen(1), state(px(100, 100, 1), null));
            compare(o.flags & Qt.WindowType_Mask, Qt.Tool, "a tool window");
            for (const f of [Qt.WindowTransparentForInput, Qt.WindowDoesNotAcceptFocus, Qt.WindowStaysOnTopHint, Qt.FramelessWindowHint])
                verify((o.flags & f) === f, "flag " + f);
            compare(o.title, En.texts["window.pointer"]);
            compare(o.transientParent, null, "unowned, so it shows while the keyboard is away");
            verify(o.visible && Qt.colorEqual(o.color, "transparent"));
            compare([o.x, o.y], [o.frame.x, o.frame.y], "it sits where it means to");
            compare([o.badge.x, o.badge.y], [o.badgeAt.x - o.frame.x, o.badgeAt.y - o.frame.y]);
            const [m, a] = [N.lk.badge_margin, o.badgeAt];
            const [x0, y0] = [Math.floor(a.x - m), Math.floor(a.y - m)];
            const [x1, y1] = [Math.ceil(a.x + o.badge.width + m), Math.ceil(a.y + o.badge.height + m)];
            compare(o.frame, Qt.rect(x0, y0, x1 - x0, y1 - y0), "just the badge and its shadow's room");
        }

        function test_the_overlay_is_made_only_while_a_mode_is_on_and_freed_after() {
            const a = app();
            compare(a.pointer.item, null, "no mode, no ring");
            const on = state(null, null);
            a.ui.assist = on;
            tryVerify(() => a.pointer.item !== null, 2000, "made with Right-click on");
            a.ui.assist = Object.assign({}, on, { rclick: false, grab: "armed" });
            tryVerify(() => a.assist.grab === "armed", 2000);
            verify(a.pointer.item !== null, "kept while Grab is on");
            a.ui.assist = Object.assign({}, on, { live: false });
            tryCompare(a.pointer, "item", null, 2000, "freed when the mouse module stops");
            a.ui.assist = on;
            tryVerify(() => a.pointer.item !== null, 2000);
            a.ui.assist = Object.assign({}, on, { rclick: false });
            tryCompare(a.pointer, "item", null, 2000, "freed with every mode off");
        }

        function test_a_wake_reads_the_state_once_on_the_next_frame_then_rests() {
            const a = app();
            tryVerify(() => !a.assistRead.running, 2000, "the read at start");
            const n = a.ui.assistReads;
            a.ui.assist = state(null, null);
            a.ui.assist = state(null, null, { seq: 7 });
            compare(a.ui.assistReads, n, "never in the wake's handler");
            verify(a.assistRead.running);
            tryVerify(() => !a.assistRead.running, 2000);
            compare(a.ui.assistReads, n + 1, "two wakes before a frame read once");
            compare(a.assist.seq, 7, "the newest state");
        }

        // Its guard run is a quiet one: a key's tip the hand is reading stays, and no pop-up loses its place in front.
        function test_the_badge_hides_over_the_keyboard_and_its_guard_keeps_a_shown_tip() {
            const a = app(), f = a.face, tip = Board.tipOf(f);
            const guards = () => a.keyboard.calls.filter(c => c.indexOf("guard") === 0).length;
            tryVerify(() => guards() > 0, 2000, "the keyboard's own run");
            f.hint(En.texts["key.rclick"], f.contentItem);
            tryVerify(() => tip.visible, 2000, "a key's tip shows");
            const n = guards();
            a.ui.assist = state(f.spots.physical(f.x + f.width / 2, f.y + f.height / 2), null);
            tryVerify(() => a.pointer.item !== null && a.pointer.item.visible, 2000);
            const o = a.pointer.item;
            verify(!o.badge.shown, "over the keyboard");
            tryVerify(() => guards() > n, 2000, "guarded once shown");
            verify(tip.visible, "the tip stays");
            compare(a.keyboard.calls.indexOf("guard " + En.texts["window.pointer"]), -1, "never put in front");
            a.ui.assist = state(f.spots.physical(f.x + f.width / 2, f.y - f.height), null);
            tryVerify(() => o.badge.shown, 2000, "beside it");
        }
    }
}

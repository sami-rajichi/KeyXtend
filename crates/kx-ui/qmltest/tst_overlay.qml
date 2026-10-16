// The pointer overlay (spec §3.2, §5.3): the badge beside the pointer and clear of its size, flipped inside the work
// area at a 1.0 and a 1.5 screen ratio, hidden over our windows; the ring at the press, once per seq; and one
// click-through window of one size that only moves and stays shown from the waves to the burst. tst_badge checks the
// badge alone, tst_overlay_app the overlay inside the app.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

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
    SignalSpy {
        id: shows
        signalName: "visibleChanged"
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
        name: "overlay"
        when: windowShown

        // The badge's tokens, the screen ratios each placement runs at, and the stand-in taskbar's height. `harden` strips
        // QT_SCALE_FACTOR, so both ratios run in this process over stand-in screens, as tst_scale's.
        readonly property var sh: N.lk.shape.badge
        readonly property var ratios: [1, 1.5]
        readonly property int bar: 48

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The hub's state: Right-click on, the pointer at physical `p` in work area `area`, then `extra`.
        function state(p, area, extra) {
            const on = { live: true, rclick: true, mode: "rclick", point: p, area: area };
            return Object.assign({}, En.assist, on, extra || {});
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
        // Whether a box at (x, y), w by h, in the overlay's own units, lies inside its window.
        function inside(o, x, y, w, h) {
            return x >= 0 && y >= 0 && x + w <= o.width && y + h <= o.height;
        }
        function ringIn(o) {
            return inside(o, o.ring.x, o.ring.y, o.ring.width, o.ring.height);
        }
        // The badge with its shadow's room.
        function badgeIn(o) {
            const m = N.lk.badge_margin;
            return inside(o, o.badge.x - m, o.badge.y - m, o.badge.width + 2 * m, o.badge.height + 2 * m);
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
            // As small as the tokens allow: the ring's box, the badge's offset and widest width with its shadow's room,
            // and one unit for rounding.
            const [ring, m, [ox, oy]] = [N.lk.shape.ring.box_px, N.lk.badge_margin, sh.offset_px];
            const room = [Math.ceil(ring + ox + o.badge.widest + m) + 1, Math.ceil(ring + oy + o.badge.height + m) + 1];
            compare([o.width, o.height], room, "the ring's and the widest badge's room");
            compare([o.frame.width, o.frame.height], room);
        }

        // A resize stalls Qt's frames, so from its first read the window keeps one size through moves, a hold's waves
        // and end, a burst with the pointer racing away and Grab's wider badge; it only moves.
        function test_the_window_keeps_one_size_and_only_moves() {
            for (const d of ratios) {
                const s = screen(d);
                const at = (x, y, extra) => state(px(x, y, d), s.area, extra);
                const ring = seq => ({ x: 1000 * d, y: 800 * d, afterMs: 0, waveMs: 1000, seq: seq });
                const o = overlay(s, at(990, 790));
                const [size, start] = [[o.width, o.height], [o.frame.x, o.frame.y]];
                for (const spy of [widths, heights]) {
                    spy.clear();
                    spy.target = o;
                }
                const steps = [at(1000, 800), at(1000, 800, { ring: ring(1) }), at(1004, 803, { ring: ring(1) }),
                               at(1004, 803), at(1000, 800, { ring: ring(2) }), at(1001, 801, { burst: ring(3) }),
                               at(1600, 1300, { burst: ring(3) }), at(1700, 1350, { mode: "grab", grab: "carrying" })];
                for (const [i, st] of steps.entries()) {
                    o.st = st;
                    compare([o.width, o.height], size, "ratio " + d + ", step " + i);
                }
                compare(widths.count + heights.count, 0, "never resized");
                verify(o.frame.x !== start[0] && o.frame.y !== start[1], "it moved");
            }
        }

        // Sized from the tokens, it holds the ring's box and the widest badge with its shadow's room for the pointer
        // anywhere inside the ring's box, as a hold keeps it within its still radius: on either side, near every edge
        // and over the taskbar to its far edge, with a big pointer too.
        function test_a_hold_keeps_the_ring_and_the_badge_inside_it() {
            const [w, h, near, half] = [4000, 3000 - bar, 40, N.lk.shape.ring.box_px / 2];
            const [rs, ra] = [N.lk.shape.ring, N.lk.motion.ring.amount];
            verify(rs.burst_px[0] * ra.burst_scale[1] <= rs.box_px, "the burst's ring stays in the ring's box");
            verify(rs.fly_px[1] + rs.dot_px / 2 <= half, "and so do its dots");
            const spots = [[1000, 800], [w - near, 800], [1000, h - near], [w - near, h - near], [near, near],
                           [1000, h + bar / 2], [1000, h + bar - 1]];
            const moves = [[0, 0], [-half, -half], [half, half], [-half, half], [half, -half]];
            for (const [d, grow] of [[1, 1], [1.5, 1], [1.5, 4]]) {
                const s = screen(d);
                const big = { pointerPx: En.assist.pointerBase * grow, mode: "grab", grab: "carrying" };
                const o = overlay(s, state(px(1000, 800, d), s.area, big));
                let seq = 0;
                for (const [x, y] of spots) {
                    for (const [dx, dy] of moves.filter(m => y + m[1] < 3000)) {
                        const ring = { x: x * d, y: y * d, afterMs: 0, waveMs: 1000, seq: ++seq };
                        o.st = state(px(x + dx, y + dy, d), s.area, Object.assign({ ring: ring }, big));
                        const tag = "ratio " + d + ", grow " + grow + ", press " + x + ", " + y + ", moved " + dx + ", " + dy;
                        fuzzyCompare(o.frame.x + o.ring.x + o.ring.width / 2, x, 0.001, tag + ": the ring at the press");
                        fuzzyCompare(o.frame.y + o.ring.y + o.ring.height / 2, y, 0.001, tag);
                        fuzzyCompare(o.frame.x + o.badge.x, o.badgeAt.x, 0.001, tag + ": the badge at its spot");
                        fuzzyCompare(o.frame.y + o.badge.y, o.badgeAt.y, 0.001, tag);
                        verify(ringIn(o), tag + ": the ring inside");
                        verify(badgeIn(o), tag + ": the badge inside");
                    }
                }
            }
        }

        // Racing away during a burst, the pointer keeps the badge whole and the burst may be cut at the window's edge,
        // as it fades out within its short move anyway.
        function test_a_pointer_racing_away_from_a_burst_keeps_the_badge_whole() {
            const s = screen(1);
            const burst = { x: 1000, y: 800, seq: 1 };
            const o = overlay(s, state(px(1000, 800, 1), s.area));
            o.st = state(px(1000, 800, 1), s.area, { burst: burst });
            verify(o.ring.bursting && ringIn(o) && badgeIn(o), "both at the fire");
            o.st = state(px(1600, 1300, 1), s.area, { burst: burst });
            verify(o.ring.bursting, "the burst still plays");
            verify(badgeIn(o), "the badge whole");
            verify(!ringIn(o), "the burst left behind");
        }

        // With no pointer point yet, as a saved mode before any move, the ring alone shows the window: it stays shown from
        // the waves to the burst, and from one burst to the next hold's waves.
        function test_the_window_stays_shown_from_the_waves_to_the_burst() {
            const s = screen(1);
            const [ring, burst] = [{ x: 100, y: 100, afterMs: 0, waveMs: 1000, seq: 1 }, { x: 100, y: 100, seq: 2 }];
            const o = overlay(s, state(null, s.area, { ring: ring }));
            verify(o.visible && o.ring.waving, "the waves show it");
            shows.clear();
            shows.target = o;
            o.st = state(null, s.area, { burst: burst });
            verify(o.visible && o.ring.busy, "the burst plays");
            o.st = state(null, s.area, { burst: burst, ring: Object.assign({}, ring, { seq: 3 }) });
            verify(o.visible && o.ring.waving, "the next hold's waves");
            compare(shows.count, 0, "it never hid between");
            o.st = state(null, s.area);
            verify(!o.visible && shows.count === 1, "a cleared ring hides it");
        }
    }
}

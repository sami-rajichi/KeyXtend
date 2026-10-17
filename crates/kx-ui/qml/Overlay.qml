// The pointer overlay (spec §5.3): one click-through window at the pointer that draws the hold ring where a press began
// and the badge beside the pointer, with its mode and the scroll pad's part. It never takes a click or the focus
// (ADR-0013), and it keeps one size, as a resize stalls Qt's frames, so a read only moves it.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: over

    // The look and texts, the hub's state as the app last read it (`Ui.assistJson`, in physical pixels), Qt units for
    // physical points (Spots), and our windows, over which the badge hides.
    required property var lk
    required property var res
    required property var st
    required property Spots spots
    required property var ours

    readonly property alias ring: ring
    readonly property alias badge: badge
    readonly property var sh: lk.shape.badge
    // How much bigger than Windows' default the pointer is, which pushes the badge out.
    readonly property real grow: st.pointerPx / st.pointerBase
    // The pointer in Qt units, null until it moves with a mode on, and the badge's top-left beside it. The spots may go
    // first as the windows are torn down.
    readonly property var at: st.point && spots ? spots.logical(st.point.x, st.point.y) : null
    readonly property point badgeAt: at ? spot(at, badge.width, badge.height) : Qt.point(0, 0)
    // The pointer is over one of our windows, so the badge hides.
    readonly property bool covered: !!at && overOurs(at)
    // Where the ring centres, in Qt units: where its hold began or its burst fired.
    property point ringAt: Qt.point(0, 0)
    // The seqs of the last ring and burst played, null for none, once made: `st` may change before the other properties
    // are set.
    property var ringSeq: null
    property var burstSeq: null
    property bool made: false
    // The badge's widest settled width once made: its word follows the keyboard's language, and its room only grows,
    // on the next pass, so a narrower word or a half-set one (new text, old font) never resizes the window.
    property real widest: 0
    // The window's one size in whole Qt units: the ring's box and the widest badge with its shadow's room on either
    // side of the pointer, for the pointer anywhere inside the ring's box as a hold keeps it, and a unit for rounding.
    readonly property size room: {
        const [bx, by] = sh.offset_px, m = lk.badge_margin, wide = made ? widest : badge.widest;
        const side = (o, len) => Math.max(span(o * grow - m, o * grow + len + m), span(-o - len - m, -o + m));
        return Qt.size(Math.ceil(side(bx, wide)) + 1, Math.ceil(side(by, badge.height)) + 1);
    }
    // Where the window sits, in whole Qt units, with its one size: around the ring's box while it plays and the badge
    // with its shadow's room.
    readonly property rect frame: place(ring.busy ? ringBox() : null, at ? badgeBox() : null)

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    transientParent: null
    title: res.tr("window.pointer")
    color: "transparent"
    visible: ring.busy || !!at
    x: rescale.keep(frame.x)
    y: rescale.keep(frame.y)
    width: rescale.keep(room.width)
    height: rescale.keep(room.height)

    onStChanged: if (made) follow()
    // A burst already in the state when the overlay is made played before it, so it never plays.
    Component.onCompleted: {
        burstSeq = st.burst ? st.burst.seq : null;
        widen();
        made = true;
        follow();
    }

    // Plays a new burst, then a new hold's ring, each once by its seq; a ring that went without firing hides at once.
    function follow() {
        const b = st.burst, r = st.ring;
        if (b && b.seq !== burstSeq) {
            burstSeq = b.seq;
            ringAt = pointOf(b);
            ring.fire();
        }
        if (r && r.seq !== ringSeq) {
            ringSeq = r.seq;
            ringAt = pointOf(r);
            ring.wave(r.afterMs, r.waveMs);
        } else if (!r && ring.waving) {
            ring.clear();
        }
    }
    // Keeps the badge's width when it is the widest yet.
    function widen() {
        widest = Math.max(widest, badge.widest);
    }
    // Physical point `p` in Qt units.
    function pointOf(p) {
        const l = spots.logical(p.x, p.y);
        return Qt.point(l.x, l.y);
    }
    // The badge's top-left for pointer `p`: below and right, its offset grown with a pointer bigger than Windows'
    // default. Near the work area's right or bottom edge it flips to the side the pointer's body is not on, at the base
    // offset, and it stays inside.
    function spot(p, w, h) {
        const [bx, by] = sh.offset_px;
        const [x, y] = [p.x + bx * grow, p.y + by * grow];
        if (!st.area)
            return Qt.point(x, y);
        const a = area(st.area), e = sh.edge_px;
        const fit = (v, lo, hi) => Math.max(lo, Math.min(v, hi));
        const fx = x + w > a.x + a.width - e ? p.x - bx - w : x;
        const fy = y + h > a.y + a.height - e ? p.y - by - h : y;
        return Qt.point(fit(fx, a.x + e, a.x + a.width - e - w), fit(fy, a.y + e, a.y + a.height - e - h));
    }
    // Physical rect `r` in Qt units, at the scale of the screen holding its corner.
    function area(r) {
        const c = spots.logical(r.x, r.y);
        const sc = spots.screenOf(r.x, r.y, true);
        const d = sc ? sc.devicePixelRatio : 1;
        return Qt.rect(c.x, c.y, r.w / d, r.h / d);
    }
    // Whether Qt point `p` is over one of our shown windows, each cut to its `boxes` where it has them, as the keyboard.
    function overOurs(p) {
        const holds = (w, b) => p.x >= w.x + b.x && p.x < w.x + b.x + b.w && p.y >= w.y + b.y && p.y < w.y + b.y + b.h;
        const whole = w => [{ x: 0, y: 0, w: w.width, h: w.height }];
        return ours.some(w => !!w && w.visible && (w.boxes || whole(w)).some(b => holds(w, b)));
    }
    function ringBox() {
        return Qt.rect(ringAt.x - ring.width / 2, ringAt.y - ring.height / 2, ring.width, ring.height);
    }
    function badgeBox() {
        const m = lk.badge_margin;
        return Qt.rect(badgeAt.x - m, badgeAt.y - m, badge.width + 2 * m, badge.height + 2 * m);
    }
    // How far the ring's box and a box from `lo` to `hi` beside the pointer reach along one side together, with the
    // pointer anywhere inside the ring's box.
    function span(lo, hi) {
        const half = ring.width / 2;
        return Math.max(...[-half, half].map(d => Math.max(half, d + hi) - Math.min(-half, d + lo)));
    }
    // The window centred on the ring's box `rb` and the badge's `bb` that show; when both do not fit, as the pointer
    // races away from a burst, it keeps the badge whole and the burst is cut at its edge. The ring's box while none
    // shows, as nothing does then.
    function place(rb, bb) {
        const b = [rb, bb].filter(r => r !== null);
        if (b.length === 0)
            b.push(ringBox());
        const lo = k => Math.min(...b.map(r => r[k]));
        const hi = (k, len) => Math.max(...b.map(r => r[k] + r[len]));
        const x = start(lo("x"), hi("x", "width"), room.width, bb ? [bb.x, bb.x + bb.width] : null);
        const y = start(lo("y"), hi("y", "height"), room.height, bb ? [bb.y, bb.y + bb.height] : null);
        return Qt.rect(x, y, room.width, room.height);
    }
    // Where a window side `size` long starts, in whole Qt units: centred on `lo` to `hi`, holding all of it when it
    // fits, and always the span `keep` when given.
    function start(lo, hi, size, keep) {
        const from = Math.min(Math.max(Math.floor((lo + hi - size) / 2), Math.ceil(hi) - size), Math.floor(lo));
        return keep ? Math.min(Math.max(from, Math.ceil(keep[1]) - size), Math.floor(keep[0])) : from;
    }

    // Keeps the size and place above across display scale changes.
    Rescale {
        id: rescale
    }
    HoldRing {
        id: ring
        lk: over.lk
        x: over.ringAt.x - over.frame.x - width / 2
        y: over.ringAt.y - over.frame.y - height / 2
    }
    Badge {
        id: badge
        lk: over.lk
        res: over.res
        mode: over.st.mode || ""
        grab: over.st.grab
        scroll: over.st.scroll || null
        shown: !!over.at && !over.covered
        x: over.badgeAt.x - over.frame.x
        y: over.badgeAt.y - over.frame.y
        onWidestChanged: Qt.callLater(over.widen)
    }
}

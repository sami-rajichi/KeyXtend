// The pointer overlay (spec §5.3): one click-through window at the pointer that draws the hold ring where a press began
// and the mode badge beside the pointer. It never takes a click or the focus (ADR-0013), and it moves only on a read.
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
    // What the window covers, in whole Qt units: the ring while it plays, and the badge with its shadow's room.
    readonly property rect frame: cover([ring.busy ? ringBox() : null, at ? badgeBox() : null])

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    transientParent: null
    title: res.tr("window.pointer")
    color: "transparent"
    visible: ring.busy || !!at
    x: rescale.keep(frame.x)
    y: rescale.keep(frame.y)
    width: rescale.keep(frame.width)
    height: rescale.keep(frame.height)

    onStChanged: if (made) follow()
    // A burst already in the state when the overlay is made played before it, so it never plays.
    Component.onCompleted: {
        burstSeq = st.burst ? st.burst.seq : null;
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
    // Physical point `p` in Qt units.
    function pointOf(p) {
        const l = spots.logical(p.x, p.y);
        return Qt.point(l.x, l.y);
    }
    // The badge's top-left for pointer `p`: below and right, its offset grown with a pointer bigger than Windows'
    // default. Near the work area's right or bottom edge it flips to the side the pointer's body is not on, at the base
    // offset, and it stays inside.
    function spot(p, w, h) {
        const grow = st.pointerPx / st.pointerBase;
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
    // The smallest box in whole Qt units around the `boxes` given; the ring's while none is, as nothing shows then.
    function cover(boxes) {
        const b = boxes.filter(r => r !== null);
        if (b.length === 0)
            b.push(ringBox());
        const left = Math.floor(Math.min(...b.map(r => r.x)));
        const top = Math.floor(Math.min(...b.map(r => r.y)));
        const right = Math.ceil(Math.max(...b.map(r => r.x + r.width)));
        const bottom = Math.ceil(Math.max(...b.map(r => r.y + r.height)));
        return Qt.rect(left, top, right - left, bottom - top);
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
        rclick: over.st.rclick
        grab: over.st.grab
        shown: !!over.at && !over.covered
        x: over.badgeAt.x - over.frame.x
        y: over.badgeAt.y - over.frame.y
    }
}

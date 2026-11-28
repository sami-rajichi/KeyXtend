// One frozen screen (design/snip-parts.css .dim, .hole, .tag, .hover): its picture, dimmed but for the part being
// picked or taken, with the region's edge or frame and size tag, or a ring round the window or screen a click would
// take. A press is a spot on it; it never takes focus.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: ov

    // Snip's windows, and this screen's place in the view's frozen list.
    required property var snips
    required property int at

    readonly property var lk: snips.lk
    readonly property var v: snips.view
    readonly property var n: snips.names
    // This screen as the view gives it (`{rect, dpi, src}`), and its box in Qt units with its scale.
    readonly property var screenInfo: v && v.frozen[at] ? v.frozen[at] : null
    readonly property rect area: screenInfo ? snips.box(screenInfo.rect) : Qt.rect(0, 0, 0, 0)
    readonly property real dpr: screenInfo && area.width > 0 ? screenInfo.rect.w / area.width : 1
    // The part to keep clear in physical pixels (`{box, ring}`, with the frame's handle to light as `lit`), or null:
    // the strip's part, moved by a held handle, the region from the first corner to the pointer, or the window or
    // screen.
    readonly property var part: clear()
    readonly property string way: v && v.stage === n.stages.pick ? wayOf(v.pick.keys) : ""
    // A region's frame on the strip (`{within, min, grip}`), whose handles take clicks, or null.
    readonly property var frame: v && v.stage === n.stages.strip ? v.strip.frame : null
    // The pointer as a handle was picked up; the frame follows only once it moves, so a reader's pick-up waits for it.
    property point still: Qt.point(NaN, NaN)
    readonly property string grip: frame && frame.grip ? JSON.stringify(frame.grip) : ""
    onGripChanged: still = snips.pointer

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | (snips.low ? 0 : Qt.WindowStaysOnTopHint)
    title: snips.screenTitle
    color: "transparent"
    visible: !!screenInfo
    x: rescale.keep(area.x)
    y: rescale.keep(area.y)
    width: rescale.keep(area.width)
    height: rescale.keep(area.height)

    // Moved or on a new scale, the frozen screens stay under our other windows.
    onXChanged: snips.reorder()
    onYChanged: snips.reorder()
    onWidthChanged: snips.reorder()
    onHeightChanged: snips.reorder()

    // Keeps the size and place above across display scale changes.
    Rescale {
        id: rescale
    }

    // The picked way, by the lit bar key.
    function wayOf(keys) {
        const lit = keys.find(k => k.on);
        return lit ? lit.key : "";
    }
    // Physical box `r` in this window's Qt units.
    function local(r) {
        const o = screenInfo.rect;
        return Qt.rect((r.x - o.x) / dpr, (r.y - o.y) / dpr, r.w / dpr, r.h / dpr);
    }
    // The box from the first corner to the pointer, both corners in, as the module takes a region.
    function span(a, b) {
        const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y);
        return { x: x, y: y, w: Math.abs(a.x - b.x) + 1, h: Math.abs(a.y - b.y) + 1 };
    }
    // Point `p` kept on box `r`, the screen a video region keeps to, or as it is when there is none.
    function kept(p, r) {
        if (!r)
            return p;
        return { x: Math.min(Math.max(p.x, r.x), r.x + r.w - 1), y: Math.min(Math.max(p.y, r.y), r.y + r.h - 1) };
    }
    function clear() {
        if (!v || !screenInfo)
            return null;
        if (v.stage === n.stages.strip)
            return framed();
        if (v.stage !== n.stages.pick)
            return null;
        const p = snips.pointer;
        if (isNaN(p.x))
            return null;
        if (v.pick.corner)
            return { box: span(v.pick.corner, kept(p, v.pick.within)), ring: false };
        const under = r => p.x >= r.x && p.x < r.x + r.w && p.y >= r.y && p.y < r.y + r.h;
        if (way === n.window) {
            const w = v.pick.windows.find(under);
            return w ? { box: w, ring: true } : null;
        }
        const whole = n.whole.indexOf(way) >= 0;
        return whole && under(screenInfo.rect) ? { box: screenInfo.rect, ring: true } : null;
    }
    // The strip's part, moved by the held handle once the pointer moves; the pointer is read only while one is held.
    function framed() {
        if (!grip)
            return { box: v.strip.part, ring: false, lit: "" };
        const p = snips.pointer;
        if (isNaN(p.x) || (p.x === still.x && p.y === still.y))
            return { box: v.strip.part, ring: false, lit: frame.grip.handle };
        const m = handles.follow(v.strip.part, frame, p);
        return { box: m.box, ring: false, lit: m.lit };
    }

    // The frozen picture, read once and never cached, so its pixels go with the view (G8).
    Image {
        anchors.fill: parent
        source: ov.screenInfo ? ov.screenInfo.src : ""
        cache: false
        smooth: false
        fillMode: Image.Stretch
    }
    SnipShade {
        objectName: "shade"
        anchors.fill: parent
        lk: ov.lk
        res: ov.snips.res
        tag: ov.snips.tag
        hole: ov.part && !ov.part.ring ? ov.local(ov.part.box) : null
        size: ov.part && !ov.part.ring ? ov.part.box : null
        ring: ov.part && ov.part.ring ? ov.local(ov.part.box) : null
        framed: !!ov.frame
    }
    // A press is a spot in this window's Qt units, while picking or on a region's frame; the pointer is followed across
    // every frozen screen.
    MouseArea {
        objectName: "spots"
        anchors.fill: parent
        hoverEnabled: true
        enabled: !!ov.v && (ov.v.stage === ov.n.stages.pick || !!ov.frame)
        cursorShape: handles.cursorAt(mouseX, mouseY, Qt.CrossCursor)
        Accessible.role: Accessible.Canvas
        Accessible.name: ov.title
        onPositionChanged: mouse => ov.snips.pointer = Qt.point(ov.screenInfo.rect.x + mouse.x * ov.dpr, ov.screenInfo.rect.y + mouse.y * ov.dpr)
        onPressed: mouse => ov.snips.snip.spot(ov.at, mouse.x, mouse.y, ov.snips.round)
    }
    // The region's frame over the hole, its held side or corner following the pointer; a press falls through to the
    // spot under it, and a reader's press on a handle picks it up by name.
    SnipFrame {
        id: handles
        objectName: "frame"
        anchors.fill: parent
        lk: ov.lk
        res: ov.snips.res
        n: ov.n
        box: ov.frame && ov.part ? ov.local(ov.part.box) : null
        held: ov.frame && ov.part ? ov.part.lit || "" : ""
        onPick: id => ov.snips.act(ov.n.acts.handle, id)
    }
}

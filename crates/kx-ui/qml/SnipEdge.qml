// The part's edge during a countdown and a recording (design/snip-parts.css .soon, .edge): four thin click-through
// windows just outside the part, or just inside where its screen ends, as one window over the part would sit in the
// video. All four share one title, so one shield keeps them out of every capture.
pragma ComponentBehavior: Bound
import QtQuick

QtObject {
    id: edge

    // Snip's windows.
    required property var snips

    readonly property var lk: snips.lk
    readonly property var sh: lk.shape.snip
    readonly property var n: snips.names
    readonly property var sides: n.sides
    readonly property var v: snips.view
    // The part counting down or recording, its stage, and whether the edges show.
    readonly property var live: v && v.stage === n.stages.count ? v.count : v && v.stage === n.stages.record ? v.record : null
    readonly property bool rec: !!v && v.stage === n.stages.record
    readonly property bool shows: !!live && snips.up(v.stage, n.capture.edge)
    readonly property rect part: live ? snips.box(live.part) : Qt.rect(0, 0, 0, 0)
    // The screen holding the part's centre, in Qt units.
    readonly property rect area: live ? screenOf() : Qt.rect(0, 0, 0, 0)
    // How thick each side is: the red line and its glow, or the dashed line.
    readonly property real px: rec ? sh.edge_px + sh.glow_px : sh.soon_px
    readonly property string title: snips.res.tr("window.snip-edge")

    // Side `name`'s window box: just outside the part, or just inside it where it would leave the part's screen. Where
    // two sides meet outside, the top or bottom reaches over the corner (`lead` at its start, `tail` at its end), so the
    // ring closes.
    function at(name) {
        const [b, a, t, s] = [part, area, px, sides];
        const out = {
            [s.top]: b.y - t >= a.y,
            [s.bottom]: b.y + b.height + t <= a.y + a.height,
            [s.left]: b.x - t >= a.x,
            [s.right]: b.x + b.width + t <= a.x + a.width
        };
        const wide = across(name) && out[name];
        const [lead, tail] = [wide && out[s.left] ? t : 0, wide && out[s.right] ? t : 0];
        const row = (y) => Qt.rect(b.x - lead, y, b.width + lead + tail, t);
        const boxes = {
            [s.top]: row(out[s.top] ? b.y - t : b.y),
            [s.bottom]: row(out[s.bottom] ? b.y + b.height : b.y + b.height - t),
            [s.left]: Qt.rect(out[s.left] ? b.x - t : b.x, b.y, t, b.height),
            [s.right]: Qt.rect(out[s.right] ? b.x + b.width : b.x + b.width - t, b.y, t, b.height)
        };
        return { box: boxes[name], out: out[name], lead: lead, tail: tail };
    }
    // True for the top or bottom side, which lies across.
    function across(name) {
        return name === sides.top || name === sides.bottom;
    }
    // The side of its window the line lies on: by the part, so inside it faces out and outside it faces in.
    function near(name) {
        const s = sides;
        const flip = { [s.top]: s.bottom, [s.bottom]: s.top, [s.left]: s.right, [s.right]: s.left };
        return at(name).out ? flip[name] : name;
    }
    function screenOf() {
        const r = live.part;
        const sc = snips.spots.screenOf(r.x + r.w / 2, r.y + r.h / 2, true);
        if (!sc)
            return part;
        return Qt.rect(sc.virtualX, sc.virtualY, sc.width, sc.height);
    }

    property SnipSide top: SnipSide {
        edge: edge
        name: edge.sides.top
    }
    property SnipSide bottom: SnipSide {
        edge: edge
        name: edge.sides.bottom
    }
    property SnipSide left: SnipSide {
        edge: edge
        name: edge.sides.left
    }
    property SnipSide right: SnipSide {
        edge: edge
        name: edge.sides.right
    }

    // All four are made and kept out of every capture before they ever show.
    Component.onCompleted: snips.snip.shield(n.capture.edge, title)
}

// A frame with eight handles (plan 2026-11-26 "The frame"; Lightshot's dashed region with its eight squares): a dashed
// box drawn as the countdown's outline, and a square at each corner and side, the lit one in the ring colour. A click
// lands on what lies under it; a reader presses a handle by name.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: fr

    // The look, the texts and Snip's names; the box in this item's units, or null for none; the lit handle's id, or "".
    required property var lk
    required property var res
    required property var n
    property var box: null
    property string held: ""

    // A reader pressed handle `id`.
    signal pick(string id)

    readonly property var sh: lk.shape.snip
    readonly property var c: lk.common
    // The box, or an empty one at the top-left while there is none, so nothing reads a missing box.
    readonly property rect b: box ? box : Qt.rect(0, 0, 0, 0)
    // Each handle with its place on the box as a share across and down: 0 the near edge, 1 the far one, a half the
    // middle, which leaves that axis alone.
    readonly property var spots: [[n.handle.topLeft, 0, 0], [n.handle.top, 0.5, 0], [n.handle.topRight, 1, 0], [n.handle.right, 1, 0.5], [n.handle.bottomRight, 1, 1], [n.handle.bottom, 0.5, 1], [n.handle.bottomLeft, 0, 1], [n.handle.left, 0, 0.5]]

    // Box `r` (`{x, y, w, h}`) with frame `f`'s held handle moved by the pointer's travel to `p`, by the rules the
    // module sets it by (`kx_mod_snip::frame::moved`): past the opposite side it swaps, and it stays in `f.within`, at
    // least `f.min` on each side it moves. Gives `{box, lit}`, lit being the handle whose spot the moving edges hold.
    function follow(r, f, p) {
        const g = f.grip;
        const s = spots.find(e => e[0] === g.handle);
        const [dx, dy] = [Math.round(p.x) - g.at.x, Math.round(p.y) - g.at.y];
        if (!s || (dx === 0 && dy === 0))
            return { box: r, lit: g.handle };
        const w = f.within;
        const [x0, x1, tx] = axis(r.x, r.x + r.w, s[1], dx, w.x, w.x + w.w, f.min);
        const [y0, y1, ty] = axis(r.y, r.y + r.h, s[2], dy, w.y, w.y + w.h, f.min);
        const [ax, ay] = [tx ? 1 - s[1] : s[1], ty ? 1 - s[2] : s[2]];
        const lit = spots.find(e => e[1] === ax && e[2] === ay)[0];
        return { box: { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }, lit: lit };
    }
    // One axis from `near` to `far`, its edge at share `at` moved by `by`, kept on `low` to `high` and `min` long; the
    // third value says the moving edge crossed to the other side.
    function axis(near, far, at, by, low, high, min) {
        if (by === 0 || (at > 0 && at < 1))
            return [near, far, false];
        const [fixed, moving] = at === 0 ? [far, near] : [near, far];
        const to = Math.min(Math.max(moving + by, low), high);
        const after = to === fixed ? moving > fixed : to > fixed;
        const least = Math.min(min, high - low);
        const [lo, hi] = after ? [fixed, Math.max(to, fixed + least)] : [Math.min(to, fixed - least), fixed];
        const len = Math.min(hi - lo, high - low);
        const from = Math.min(Math.max(lo, low), high - len);
        return [from, from + len, after !== (at === 1)];
    }
    // The pointer's shape over a handle at shares `ax` and `ay`: up and down on the top and bottom, across on the
    // sides, and along its diagonal on a corner.
    function cursorOf(ax, ay) {
        if (ax > 0 && ax < 1)
            return Qt.SizeVerCursor;
        if (ay > 0 && ay < 1)
            return Qt.SizeHorCursor;
        return ax === ay ? Qt.SizeFDiagCursor : Qt.SizeBDiagCursor;
    }
    // The pointer's shape at (`x`, `y`) in this item's units: the handle's under it, or `other` off every handle. The
    // press area asks, since a hover handler here would keep the pointer from it.
    function cursorAt(x, y, other) {
        const g = childAt(x, y);
        return g && g.cursor !== undefined ? g.cursor : other;
    }

    // White dashes over a dim line, so the box shows on any picture (snip-parts.css .soon).
    Shape {
        id: outline
        objectName: "outline"
        visible: !!fr.box
        readonly property rect b: fr.b
        readonly property var corners: [Qt.point(b.x, b.y), Qt.point(b.x + b.width, b.y), Qt.point(b.x + b.width, b.y + b.height), Qt.point(b.x, b.y + b.height), Qt.point(b.x, b.y)]
        ShapePath {
            strokeColor: fr.c.snip_dim
            strokeWidth: fr.sh.soon_px
            fillColor: "transparent"
            PathPolyline {
                path: outline.corners
            }
        }
        ShapePath {
            strokeColor: fr.c.snip_edge
            strokeWidth: fr.sh.soon_px
            fillColor: "transparent"
            strokeStyle: ShapePath.DashLine
            dashPattern: fr.sh.dash_px.map(d => d / fr.sh.soon_px)
            capStyle: ShapePath.FlatCap
            PathPolyline {
                path: outline.corners
            }
        }
    }
    // Each handle's click area round its square; only the handles on this item show, so a reader hears each once.
    Repeater {
        model: fr.box ? fr.spots : []
        delegate: Item {
            id: grip
            required property var modelData
            readonly property string key: modelData[0]
            readonly property real cx: fr.b.x + modelData[1] * fr.b.width
            readonly property real cy: fr.b.y + modelData[2] * fr.b.height
            readonly property bool lit: fr.held === key
            readonly property int cursor: fr.cursorOf(modelData[1], modelData[2])
            objectName: "handle"
            visible: cx >= 0 && cx <= fr.width && cy >= 0 && cy <= fr.height
            width: fr.sh.handle_hit_px
            height: width
            x: cx - width / 2
            y: cy - height / 2
            Accessible.role: Accessible.Button
            Accessible.name: fr.res.tr("snip.handle-" + key)
            Accessible.id: fr.res.uid("snip.handle", key)
            Accessible.onPressAction: fr.pick(grip.key)

            Rectangle {
                objectName: "square"
                anchors.centerIn: parent
                width: fr.sh.handle_px
                height: width
                color: grip.lit ? fr.lk.palette.ring : fr.c.snip_edge
                border.width: fr.sh.hole_px
                border.color: fr.c.snip_dim
            }
        }
    }
}

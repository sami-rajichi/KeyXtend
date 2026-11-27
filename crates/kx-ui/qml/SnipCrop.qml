// Crop on the editor's picture (plan 2026-11-26 "Crop"): from its first click to the pointer, the part it would frame;
// after its second click, the frame and its eight handles (SnipFrame), the held side or corner following the pointer
// until a click sets it. Drawn in the canvas's units over the picture, so a handle on the picture's edge shows whole.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: crop

    // The picture it lies on.
    required property var canvas

    readonly property var sh: canvas.sh
    readonly property var c: canvas.lk.common
    readonly property var pic: canvas.pic
    readonly property bool on: !!pic && pic.tool === canvas.n.crop
    // Crop's frame waiting (`{part, within, min, grip}` in picture pixels), or null.
    readonly property var waiting: on && pic.frame ? pic.frame : null
    // The held handle, by value, so a new view with the same one keeps the pointer it was picked up at.
    readonly property string grip: waiting && waiting.grip ? JSON.stringify(waiting.grip) : ""
    // The pointer as a handle was picked up; the frame follows only once it moves, so a reader's pick-up waits for it.
    property var still: null
    // The frame as it shows, in picture pixels, with the handle to light (`{box, lit}`), or null.
    readonly property var framed: shows()
    // The pointer's box from the first click, in picture pixels, or null.
    readonly property var next: {
        const a = on && pic.pending ? pic.pending : null;
        const b = a ? canvas.pointer : null;
        return a && b ? { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(b.x - a.x), h: Math.abs(b.y - a.y) } : null;
    }

    onGripChanged: still = canvas.pointer

    // The frame waiting, its held handle moved by the pointer's travel by the module's rules once the pointer moves.
    // The pointer is read only while a handle is held, so a move with none held rebuilds nothing.
    function shows() {
        if (!waiting)
            return null;
        if (!grip)
            return { box: waiting.part, lit: "" };
        const p = canvas.pointer;
        const moved = !!p && (!still || p.x !== still.x || p.y !== still.y);
        return moved ? frame.follow(waiting.part, waiting, p) : { box: waiting.part, lit: waiting.grip.handle };
    }
    // Picture box `r` (`{x, y, w, h}`) in this item's units.
    function local(r) {
        const [v, s, k] = [canvas.view, canvas.shown, canvas.k];
        return Qt.rect(v.x + (r.x - s.x) * k, v.y + (r.y - s.y) * k, r.w * k, r.h * k);
    }
    // The pointer's shape at (`x`, `y`) in this item's units: a handle's, or `other` off every handle.
    function cursorAt(x, y, other) {
        return frame.cursorAt(x, y, other);
    }

    // The next part in white dashes over a dim line, so it shows on any picture (snip-parts.css .soon).
    Shape {
        id: soon
        objectName: "soon"
        visible: !!crop.next
        readonly property rect b: crop.next ? crop.local(crop.next) : Qt.rect(0, 0, 0, 0)
        readonly property var corners: [Qt.point(b.x, b.y), Qt.point(b.x + b.width, b.y), Qt.point(b.x + b.width, b.y + b.height), Qt.point(b.x, b.y + b.height), Qt.point(b.x, b.y)]
        ShapePath {
            strokeColor: crop.c.snip_dim
            strokeWidth: crop.sh.soon_px
            fillColor: "transparent"
            PathPolyline {
                path: soon.corners
            }
        }
        ShapePath {
            strokeColor: crop.c.snip_edge
            strokeWidth: crop.sh.soon_px
            fillColor: "transparent"
            strokeStyle: ShapePath.DashLine
            dashPattern: crop.sh.dash_px.map(d => d / crop.sh.soon_px)
            capStyle: ShapePath.FlatCap
            PathPolyline {
                path: soon.corners
            }
        }
    }
    // The frame waiting; a click lands on the picture's press area under it, and a reader presses a handle by name.
    SnipFrame {
        id: frame
        anchors.fill: parent
        lk: crop.canvas.lk
        res: crop.canvas.res
        n: crop.canvas.n
        box: crop.framed ? crop.local(crop.framed.box) : null
        held: crop.framed ? crop.framed.lit : ""
        onPick: id => crop.canvas.snips.act(crop.canvas.n.acts.handle, id)
    }
}

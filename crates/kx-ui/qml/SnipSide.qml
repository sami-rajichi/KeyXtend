// One side of the part's edge (design/snip-parts.css .soon, .edge): a white dashed line on a dim one during the
// countdown, or the red line with its fading glow while it records. Clicks pass through it and it never takes focus.
import QtQuick
import QtQuick.Shapes

Window {
    id: side

    // The part's edge it belongs to, and which of its `sides` it is.
    required property var edge
    required property string name

    readonly property var lk: edge.lk
    readonly property var sh: lk.shape.snip
    readonly property var c: lk.common
    readonly property var s: edge.sides
    readonly property color red: c.snip_rec
    readonly property bool rec: edge.rec
    // Its box in Qt units, with how far it reaches over a corner at its start and end, and the side the line lies on.
    readonly property var at: edge.at(name)
    readonly property string near: edge.near(name)
    readonly property bool across: edge.across(near)
    // The line lies on its window's top or left, where the glow fades from.
    readonly property bool first: near === s.top || near === s.left
    // The line's thickness, and how far it lies from the window's own edges on the far side.
    readonly property real px: rec ? sh.edge_px : sh.soon_px
    readonly property real far: (across ? height : width) - px
    // Over a corner the line reaches only to the other side's line, and the glow fills the rest.
    readonly property real start: at.lead > 0 ? at.lead - px : 0
    readonly property real end: at.tail > 0 ? at.tail - px : 0
    // The line's box inside the window.
    readonly property rect line: across ? Qt.rect(start, first ? 0 : far, width - start - end, px) : Qt.rect(first ? 0 : far, 0, px, height)

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput | (edge.snips.low ? 0 : Qt.WindowStaysOnTopHint)
    title: edge.title
    color: "transparent"
    visible: edge.shows && at.box.width > 0 && at.box.height > 0
    x: Math.round(at.box.x)
    y: Math.round(at.box.y)
    width: Math.max(1, Math.round(at.box.width))
    height: Math.max(1, Math.round(at.box.height))

    // The glow, fading out from the red line to the window's far side.
    Rectangle {
        visible: side.rec
        x: side.first && !side.across ? side.px : 0
        y: side.first && side.across ? side.px : 0
        width: side.across ? side.width : side.width - side.px
        height: side.across ? side.height - side.px : side.height
        gradient: Gradient {
            orientation: side.across ? Gradient.Vertical : Gradient.Horizontal
            GradientStop {
                position: side.first ? 0 : 1
                color: Qt.rgba(side.red.r, side.red.g, side.red.b, side.sh.glow_share)
            }
            GradientStop {
                position: side.first ? 1 : 0
                color: "transparent"
            }
        }
    }
    // The line: red while it records, else dim under the dashes.
    Rectangle {
        x: side.line.x
        y: side.line.y
        width: side.line.width
        height: side.line.height
        color: side.rec ? side.red : side.c.snip_dim
    }
    Shape {
        visible: !side.rec
        anchors.fill: parent
        ShapePath {
            strokeColor: side.c.snip_edge
            strokeWidth: side.px
            fillColor: "transparent"
            strokeStyle: ShapePath.DashLine
            dashPattern: side.sh.dash_px.map(d => d / side.px)
            capStyle: ShapePath.FlatCap
            startX: side.across ? side.line.x : side.line.x + side.px / 2
            startY: side.across ? side.line.y + side.px / 2 : 0
            PathLine {
                x: side.across ? side.line.x + side.line.width : side.line.x + side.px / 2
                y: side.across ? side.line.y + side.px / 2 : side.height
            }
        }
    }
}

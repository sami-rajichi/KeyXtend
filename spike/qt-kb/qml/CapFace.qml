// A key cap's fill: flat, a convex gradient that flips when pressed, or a skirt with a lighter top inset into it.
import QtQuick

Item {
    id: face

    // How caps are built (themes.toml [theme.cap]); the key's colour and skirt colour; radius, size, press and sink (0 to 1).
    required property var cap
    required property color topColour
    required property color skirt
    required property real radius
    required property real s
    required property bool pressed
    required property real sink

    readonly property bool convex: cap.kind === "convex"
    readonly property bool sculpted: cap.kind === "skirt"
    // The gradient in use, if any.
    readonly property var grad: convex ? (pressed ? cap.pressed : cap.rest) : (sculpted ? cap.top : null)
    // The top's inset while sculpted, top, right, bottom, left; it moves with the sink (mock-up .cap::before transition).
    readonly property var inset: sculpted ? cap.top_inset_px.map((v, i) => v + (cap.pressed_inset_px[i] - v) * sink) : [0, 0, 0, 0]

    // A rounded box in `base`, shaded by gradient `g` when there is one; the gradient is made once.
    component Fill: Rectangle {
        id: fill
        required property color base
        required property var g
        readonly property bool up: g ? g.upward : false
        color: base
        gradient: g ? shades : null

        // Top to bottom; an upward gradient runs bottom to top.
        Gradient {
            id: shades
            GradientStop {
                position: fill.up ? 1 : 0
                color: Qt.tint(fill.base, Qt.rgba(1, 1, 1, fill.g ? fill.g.lighten : 0))
            }
            GradientStop {
                position: fill.g ? (fill.up ? 1 - fill.g.mid_stop : fill.g.mid_stop) : 0
                color: fill.base
            }
            GradientStop {
                position: fill.up ? 0 : 1
                color: Qt.tint(fill.base, Qt.rgba(0, 0, 0, fill.g ? fill.g.darken : 0))
            }
        }
    }

    Fill {
        anchors.fill: parent
        radius: face.radius
        base: face.sculpted ? face.skirt : face.topColour
        g: face.convex ? face.grad : null
    }
    Fill {
        visible: face.sculpted
        x: face.inset[3] * face.s
        y: face.inset[0] * face.s
        width: face.width - (face.inset[1] + face.inset[3]) * face.s
        height: face.height - (face.inset[0] + face.inset[2]) * face.s
        radius: face.sculpted ? face.cap.top_radius_px * face.s : 0
        base: face.topColour
        g: face.sculpted ? face.grad : null
    }
}

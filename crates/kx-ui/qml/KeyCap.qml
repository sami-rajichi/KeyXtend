// A key's cap as the theme builds it: its outer shadows, fill, inner shading and pointer tint, sunk and shrunk by the
// press. What the cap shows goes inside it, over all of these; the keys and the selection strip's keycaps share it.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: cap

    // The look and size; the top and skirt colours; the pointer presses it or is over it.
    required property var lk
    required property real s
    required property color topColour
    required property color skirt
    required property bool pressed
    required property bool hovered

    readonly property var p: lk.palette
    readonly property var look: lk.look
    readonly property var press: lk.shape.press
    readonly property real radius: look.key_radius_px
    // How far it has sunk, 0 to 1.
    property real sink: pressed ? 1 : 0

    // A press sinks the cap at once, so the next frame shows it (spec §10); only the rise back eases. It reads the
    // target, since `pressed` and `sink` change in no set order.
    Behavior on sink {
        enabled: targetValue < cap.sink
        Tween {
            move: cap.lk.motion.moves.press
        }
    }

    transform: [
        Scale {
            id: shrink
            origin.x: cap.width / 2
            origin.y: cap.height / 2
            xScale: 1 - (1 - cap.press.scale) * cap.sink
            yScale: shrink.xScale
        },
        Translate {
            y: cap.press.sink_px * cap.s * cap.sink
        }
    ]

    Repeater {
        model: cap.p.key_shadow.filter(l => !l.inset)
        delegate: Shade {
            required property var modelData
            shadow: modelData
            radius: cap.radius
        }
    }
    CapFace {
        anchors.fill: parent
        cap: cap.look.cap
        topColour: cap.topColour
        skirt: cap.skirt
        radius: cap.radius
        s: cap.s
        pressed: cap.pressed
        sink: cap.sink
    }
    Inset {
        anchors.fill: parent
        layers: cap.p.key_shadow.filter(l => l.inset)
        cap: cap.look.cap
        topColour: cap.topColour
        skirt: cap.skirt
        radius: cap.radius
        s: cap.s
        pressed: cap.pressed
        sink: cap.sink
    }
    Hover {
        lk: cap.lk
        radius: cap.radius
        visible: cap.hovered || cap.pressed
        tint: cap.pressed ? cap.p.press : cap.p.hover
    }
}

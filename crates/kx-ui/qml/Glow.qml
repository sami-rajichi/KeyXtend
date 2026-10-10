// A click-through window that draws another window's outer shadow; the window it owns stays just above it.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: glow

    // The look (null until one reads), the shadowed box in screen Qt units, and its corner radius.
    required property var lk
    required property rect box
    required property real radius

    // An item whose fade, rise and scale the shadow copies, so they pop in together; null keeps it still.
    property Item follow: null

    readonly property real m: lk ? lk.margin : 0

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    color: "transparent"
    visible: false
    x: rescale.keep(box.x - m)
    y: rescale.keep(box.y - m)
    width: rescale.keep(box.width + 2 * m)
    height: rescale.keep(box.height + 2 * m)

    // Keeps the size and place above across display scale changes.
    Rescale {
        id: rescale
    }

    Item {
        x: glow.m
        y: glow.m + (glow.follow ? glow.follow.y : 0)
        opacity: glow.follow ? glow.follow.opacity : 1
        scale: glow.follow ? glow.follow.scale : 1
        width: glow.box.width
        height: glow.box.height
        Repeater {
            model: glow.lk ? glow.lk.palette.window_shadow.filter(l => !l.inset) : []
            delegate: Shade {
                required property var modelData
                anchors.fill: parent
                shadow: modelData
                radius: glow.radius
            }
        }
    }
}

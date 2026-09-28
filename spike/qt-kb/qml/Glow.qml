// A click-through window that draws another window's outer shadow; the window it owns stays just above it.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: glow

    // The look, the shadowed box in screen Qt units, and its corner radius.
    required property var lk
    required property rect box
    required property real radius

    readonly property real m: lk.margin

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    color: "transparent"
    visible: false
    x: box.x - m
    y: box.y - m
    width: box.width + 2 * m
    height: box.height + 2 * m

    Item {
        x: glow.m
        y: glow.m
        width: glow.box.width
        height: glow.box.height
        Repeater {
            model: glow.lk.palette.window_shadow.filter(l => !l.inset)
            delegate: Shade {
                required property var modelData
                anchors.fill: parent
                shadow: modelData
                radius: glow.radius
            }
        }
    }
}

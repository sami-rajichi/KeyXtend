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
    // Draws nothing while true, as while its window resizes live: each window draws on its own, so it would trail.
    property bool hidden: false
    // The shadowed box inside this window, for the tests.
    readonly property alias shade: shade

    readonly property real m: lk ? lk.margin : 0
    // The window's top-left in whole Qt units, as a window's place is whole; the box sits inside by the margin or more.
    readonly property int left: Math.floor(box.x - m)
    readonly property int top: Math.floor(box.y - m)

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    color: "transparent"
    visible: false
    x: rescale.keep(left)
    y: rescale.keep(top)
    width: rescale.keep(Math.ceil(box.x + box.width + m) - left)
    height: rescale.keep(Math.ceil(box.y + box.height + m) - top)

    // Keeps the size and place above across display scale changes.
    Rescale {
        id: rescale
    }

    Item {
        id: shade
        visible: !glow.hidden
        // The box exactly where it is on screen, so its edges meet the shadowed window's.
        x: glow.box.x - glow.left
        y: glow.box.y - glow.top + (glow.follow ? glow.follow.y : 0)
        opacity: glow.follow ? glow.follow.opacity : 1
        scale: glow.follow ? glow.follow.scale : 1
        width: glow.box.width
        height: glow.box.height
        Repeater {
            model: glow.lk ? glow.lk.palette.window_shadow.filter(l => !l.inset) : []
            delegate: Shade {
                required property var modelData
                shadow: modelData
                radius: glow.radius
            }
        }
    }
}

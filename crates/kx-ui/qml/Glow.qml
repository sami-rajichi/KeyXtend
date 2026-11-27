// A click-through window that draws another window's outer shadow; the window it owns stays just above it.
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: glow

    // The look (null until one reads), the texts, the shadowed box in screen Qt units, and its corner radius.
    required property var lk
    required property var res
    required property rect box
    required property real radius

    // An item whose fade, rise and scale the shadow copies, so they pop in together; null keeps it still.
    property Item follow: null
    // Draws nothing while true, as while its window resizes live: each window draws on its own, so it would trail.
    property bool hidden: false
    // While the keyboard is aside and hidden, nothing shows it, so nor the window it owns (Aside.qml).
    property bool hush: false
    // The window it owns and shadows, which shows and hides with it, and draws a frame before the shade draws; null
    // draws at once.
    property Window shadowed: null
    // The shadowed window has drawn a frame since it last showed.
    property bool drawn: false
    // The shadowed box inside this window, for the tests.
    readonly property alias shade: shade
    // Flags for a window it owns: the shadow's own, its no-focus flag kept, with only click-through dropped. They
    // change just after the shadow's, and Windows lifts each window alone on new flags (Qt 6.11), so it stays above.
    readonly property int owns: flags & ~Qt.WindowTransparentForInput

    readonly property real m: lk ? lk.margin : 0
    // The window's top-left in whole Qt units, as a window's place is whole; the box sits inside by the margin or more.
    readonly property int left: Math.floor(box.x - m)
    readonly property int top: Math.floor(box.y - m)

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    // Every shadow's one title, by which a capture's shields keep it out with the window it shadows.
    title: res ? res.tr("window.shadow") : ""
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
    // While hushed every show turns into a hide; Qt shows an owned window only once its owner shows.
    Behavior on visible {
        enabled: glow.hush
        PropertyAction {
            value: false
        }
    }
    // The shadowed window shows in the turn its owner does, so Qt never puts its show off to a later turn.
    Binding {
        target: glow.shadowed
        property: "visible"
        value: glow.visible
        when: glow.shadowed !== null
    }
    // Each show waits for a new frame; a stale frame of a hidden window never counts.
    Connections {
        target: glow.shadowed
        function onVisibleChanged() {
            glow.drawn = false;
        }
    }
    // Only the first frame after a show is heard, so no handler runs on every frame.
    Connections {
        target: glow.shadowed
        enabled: !glow.drawn
        function onFrameSwapped() {
            glow.drawn = glow.shadowed.visible;
        }
    }

    Item {
        id: shade
        visible: !glow.hidden && (!glow.shadowed || glow.drawn)
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

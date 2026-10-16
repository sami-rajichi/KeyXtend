// One edge of the scroll pad's outline: a thin click-through window in the outline's colour, which fades in as it is
// made. It never takes a click or the focus (ADR-0013), and no window owns it, so it can lie under the keyboard.
import QtQuick

Window {
    id: edge

    // The look, the colour, and where it sits in Qt units.
    required property var lk
    required property color colour
    required property rect at

    // Its paint, for the tests.
    readonly property alias paint: paint

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    transientParent: null
    color: "transparent"
    visible: true
    // In whole Qt units around its part, at least one each way, as a window's place and size are whole.
    x: rescale.keep(Math.floor(at.x))
    y: rescale.keep(Math.floor(at.y))
    width: rescale.keep(Math.max(1, Math.ceil(at.x + at.width) - Math.floor(at.x)))
    height: rescale.keep(Math.max(1, Math.ceil(at.y + at.height) - Math.floor(at.y)))

    // Keeps the size and place above across display scale changes.
    Rescale {
        id: rescale
    }
    Rectangle {
        id: paint
        anchors.fill: parent
        color: edge.colour
        Tween on opacity {
            from: 0
            to: 1
            move: edge.lk.motion.moves.outline
        }
    }
}

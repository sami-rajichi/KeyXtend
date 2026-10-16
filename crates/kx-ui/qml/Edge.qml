// One edge of the scroll pad's outline: a thin click-through window in the outline's colour, which fades in as it is
// made and, to move, fades out, moves while unseen and fades back in, as a resize stalls Qt's frames. It never takes a
// click or the focus (ADR-0013), and no window owns it, so it can lie under the keyboard.
import QtQuick

Window {
    id: edge

    // The look, the colour, and where it goes in Qt units.
    required property var lk
    required property color colour
    required property rect at
    // Where it stands, which follows `at` while unseen.
    property rect spot: at

    // Its paint and whether it is on its way, for the tests.
    readonly property alias paint: paint
    readonly property alias moving: swap.running
    readonly property var move: lk.motion.moves.outline

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus | Qt.WindowTransparentForInput
    transientParent: null
    color: "transparent"
    visible: true
    // In whole Qt units around its part, at least one each way, as a window's place and size are whole.
    x: rescale.keep(Math.floor(spot.x))
    y: rescale.keep(Math.floor(spot.y))
    width: rescale.keep(Math.max(1, Math.ceil(spot.x + spot.width) - Math.floor(spot.x)))
    height: rescale.keep(Math.max(1, Math.ceil(spot.y + spot.height) - Math.floor(spot.y)))

    // Its place is set once made, so a new `at` waits for the fade; with reduced motion it shows and moves at once.
    Component.onCompleted: {
        spot = at;
        if (move.ms === 0)
            paint.opacity = 1;
        else
            appear.start();
    }
    onAtChanged: go()

    // Each read makes a new box, so only other values move it.
    function go() {
        if (at.x === spot.x && at.y === spot.y && at.width === spot.width && at.height === spot.height)
            return;
        if (move.ms === 0) {
            spot = at;
            return;
        }
        appear.stop();
        swap.restart();
    }

    // Keeps the size and place above across display scale changes.
    Rescale {
        id: rescale
    }
    Rectangle {
        id: paint
        anchors.fill: parent
        color: edge.colour
        opacity: 0
    }
    Tween {
        id: appear
        target: paint
        property: "opacity"
        to: 1
        move: edge.move
    }
    SequentialAnimation {
        id: swap
        Tween {
            target: paint
            property: "opacity"
            to: 0
            move: edge.move
        }
        ScriptAction {
            script: edge.spot = edge.at
        }
        Tween {
            target: paint
            property: "opacity"
            to: 1
            move: edge.move
        }
    }
}

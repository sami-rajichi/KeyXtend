// A number animation timed by one move from motion.toml; `share` plays part of it, such as half a pulse.
import QtQuick

NumberAnimation {
    // The move (its length and curve) and the share of its length this part takes.
    required property var move
    property real share: 1

    duration: move.ms * share
    easing.type: Easing.BezierSpline
    easing.bezierCurve: move.bez
}

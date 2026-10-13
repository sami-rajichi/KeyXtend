// A colour animation timed by one move from motion.toml.
import QtQuick

ColorAnimation {
    // The move: its length and curve.
    required property var move

    duration: move.ms
    easing.type: Easing.BezierSpline
    easing.bezierCurve: move.bez
}

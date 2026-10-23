// The side block's swap to or from the shortcuts layer (motion.toml moves.layer): a cell new to it fades in and rises a
// little into place, as a panel pops in (mock-up popIn). With Windows' animations off or reduced motion it never runs.
import QtQuick

ParallelAnimation {
    id: swap

    // What comes in and the shift it rises by, the look, and the keyboard's size.
    required property Item target
    required property Translate shift
    required property var lk
    required property real s

    readonly property var move: lk.motion.moves.layer

    // Brings the target in, unless motion is off, which leaves it in place.
    function play() {
        if (move.ms > 0)
            restart();
    }

    Tween {
        target: swap.shift
        property: "y"
        from: swap.lk.motion.amount.pop_rise_px * swap.s
        to: 0
        move: swap.move
    }
    Tween {
        target: swap.target
        property: "opacity"
        from: 0
        to: 1
        move: swap.move
    }
}

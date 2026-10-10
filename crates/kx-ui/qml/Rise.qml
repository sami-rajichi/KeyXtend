// What rises in after a language change (mock-up .swap .lg, .sp): it slides up from `from` below while it fades in.
import QtQuick

Item {
    id: rise

    // The move (motion.toml), and how far below its place it starts.
    required property var move
    required property real from
    // It is rising.
    readonly property bool running: both.running

    // Rises in from the start.
    function play() {
        both.restart();
    }

    transform: Translate {
        id: shift
    }

    ParallelAnimation {
        id: both
        Tween {
            target: shift
            property: "y"
            from: rise.from
            to: 0
            move: rise.move
        }
        Tween {
            target: rise
            property: "opacity"
            from: 0
            to: 1
            move: rise.move
        }
    }
}

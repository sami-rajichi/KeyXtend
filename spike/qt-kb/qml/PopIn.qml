// A pop-in (mock-up @keyframes popIn): `target` fades in, rises a little and grows to full size.
// The target rests at y 0, scale 1 and opacity 1, with no bindings on them.
import QtQuick

ParallelAnimation {
    id: pop

    // What pops in, the move that times it, and how far it goes (motion.toml [amount]).
    required property Item target
    required property var move
    required property var amount

    Tween {
        target: pop.target
        property: "opacity"
        from: 0
        to: 1
        move: pop.move
    }
    Tween {
        target: pop.target
        property: "y"
        from: pop.amount.pop_rise_px
        to: 0
        move: pop.move
    }
    Tween {
        target: pop.target
        property: "scale"
        from: pop.amount.pop_scale
        to: 1
        move: pop.move
    }
}

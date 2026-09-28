// The hold ring and its burst (mock-up .ring, .burst): two waves shrink into the centre over the hold, then a ring and dots
// burst out; `loop` repeats this until stopped.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: face

    // The look, and the loop's timing: the wait before the ring shows, the waves' length and the pause after a burst, in ms.
    required property var lk
    required property int afterMs
    required property int waveMs
    required property int gapMs

    readonly property alias loop: cycle
    readonly property var sh: lk.shape.ring
    readonly property var mo: lk.motion.ring
    readonly property var amount: mo.amount
    readonly property var burstMove: lk.motion.moves.burst
    readonly property color ink: lk.palette.ring
    // How far the burst has gone, 0 to 1; at 1 it has faded out.
    property real burst: 1

    width: sh.box_px
    height: sh.box_px

    // Hides the waves and the core.
    function hide() {
        w1.opacity = 0;
        w2.opacity = 0;
        core.opacity = 0;
    }

    // Hides everything, the burst too.
    function rest() {
        hide();
        burst = 1;
    }

    // A circle centred in the ring.
    component Disc: Rectangle {
        anchors.centerIn: parent
        height: width
        radius: width / 2
        opacity: 0
    }
    // One wave shrinking into the centre over `len` ms; its opacity rises to the turn, then on to the end (@keyframes shrink).
    component Shrink: ParallelAnimation {
        id: shr
        property Item wave
        property real len
        NumberAnimation {
            target: shr.wave
            property: "scale"
            from: 1
            to: face.amount.wave_end
            duration: shr.len
            easing.type: Easing.BezierSpline
            easing.bezierCurve: face.mo.wave
        }
        SequentialAnimation {
            NumberAnimation {
                target: shr.wave
                property: "opacity"
                from: face.amount.wave_fade[0]
                to: face.amount.wave_fade[1]
                duration: shr.len * face.amount.wave_turn
                easing.type: Easing.BezierSpline
                easing.bezierCurve: face.mo.wave
            }
            NumberAnimation {
                target: shr.wave
                property: "opacity"
                to: face.amount.wave_fade[2]
                duration: shr.len * (1 - face.amount.wave_turn)
                easing.type: Easing.BezierSpline
                easing.bezierCurve: face.mo.wave
            }
        }
    }

    Disc {
        id: w1
        width: face.sh.box_px
        color: "transparent"
        border.width: face.sh.wave_px[0]
        border.color: face.ink
    }
    Disc {
        id: w2
        width: face.sh.box_px
        color: "transparent"
        border.width: face.sh.wave_px[1]
        border.color: face.ink
    }
    Disc {
        id: core
        width: face.sh.box_px * face.sh.core_share
        color: face.ink
    }
    Disc {
        id: burstRing
        width: face.sh.burst_px[0]
        color: "transparent"
        border.width: face.sh.burst_px[1]
        border.color: face.ink
        scale: face.amount.burst_scale[0] + (face.amount.burst_scale[1] - face.amount.burst_scale[0]) * face.burst
        opacity: 1 - face.burst
    }
    Repeater {
        model: face.sh.dots
        delegate: Rectangle {
            required property int index
            readonly property real turn: 2 * Math.PI * index / face.sh.dots
            readonly property real far: face.sh.fly_px[0] + (face.sh.fly_px[1] - face.sh.fly_px[0]) * face.burst
            x: face.width / 2 + far * Math.cos(turn) - width / 2
            y: face.height / 2 + far * Math.sin(turn) - height / 2
            width: face.sh.dot_px
            height: width
            radius: width / 2
            color: face.ink
            scale: 1 + (face.amount.dot_end - 1) * face.burst
            opacity: 1 - face.burst
        }
    }

    SequentialAnimation {
        id: cycle
        loops: Animation.Infinite
        ScriptAction {
            script: face.rest()
        }
        PauseAnimation {
            duration: face.afterMs
        }
        ParallelAnimation {
            Shrink {
                wave: w1
                len: face.waveMs
            }
            SequentialAnimation {
                PauseAnimation {
                    duration: face.waveMs * (1 - face.amount.wave2_share)
                }
                Shrink {
                    wave: w2
                    len: face.waveMs * face.amount.wave2_share
                }
            }
            NumberAnimation {
                target: core
                property: "scale"
                from: face.amount.core_from
                to: 1
                duration: face.waveMs
                easing.type: Easing.BezierSpline
                easing.bezierCurve: face.mo.core
            }
            NumberAnimation {
                target: core
                property: "opacity"
                from: 0
                to: face.amount.core_to
                duration: face.waveMs
                easing.type: Easing.BezierSpline
                easing.bezierCurve: face.mo.core
            }
        }
        ScriptAction {
            script: face.hide()
        }
        Tween {
            target: face
            property: "burst"
            from: 0
            to: 1
            move: face.burstMove
        }
        PauseAnimation {
            duration: face.gapMs
        }
    }
}

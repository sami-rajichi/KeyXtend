// The hold ring and its burst (mock-up .ring, .burst), from the P1 test build's: after a wait, two waves shrink into the
// press point over the rest of the hold, then a ring and dots burst out when it fires. Each plays once when asked.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: face

    // The look, and the run asked last: the wait before the waves show and their length, in ms.
    required property var lk
    property int afterMs: 0
    property int waveMs: 0

    readonly property alias waves: waves
    // The waves show, running or holding their last frame until the hold ends or fires.
    property bool waving: false
    // The burst plays.
    property bool bursting: false
    // Something shows: the waves or the burst. Each call turns the next part on before the last one off, so `busy` never
    // drops in the middle of a call and the overlay window never hides and shows again.
    readonly property bool busy: waving || bursting
    readonly property var sh: lk.shape.ring
    readonly property var mo: lk.motion.ring
    readonly property var amount: mo.amount
    readonly property var burstMove: lk.motion.moves.burst
    // Always the theme's ring colour (the owner's rule), so the accent and every theme follow.
    readonly property color ink: lk.palette.ring
    // How far the burst has gone, 0 to 1; at 1 it has faded out.
    property real burst: 1

    width: sh.box_px
    height: sh.box_px

    // Plays the waves once, ending a burst still playing: after `after` ms they shrink in over `len` ms, then keep their
    // last frame.
    function wave(after, len) {
        waving = true;
        unburst();
        hide();
        afterMs = after;
        waveMs = len;
        waves.restart();
    }
    // The hold fired: the waves go and the burst plays once; reduced motion makes its move 0 ms, so it is skipped.
    function fire() {
        waves.stop();
        hide();
        burst = 1;
        bursting = burstMove.ms > 0;
        if (bursting)
            pop.restart();
        waving = false;
    }
    // The hold ended without firing: everything hides at once.
    function clear() {
        waves.stop();
        hide();
        unburst();
        waving = false;
    }
    // Hides the waves and the core.
    function hide() {
        w1.opacity = 0;
        w2.opacity = 0;
        core.opacity = 0;
    }
    // Ends and hides the burst.
    function unburst() {
        pop.stop();
        bursting = false;
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

    // The wait, then both waves and the core over the hold; the last frame stays.
    SequentialAnimation {
        id: waves
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
    }
    Tween {
        id: pop
        target: face
        property: "burst"
        from: 0
        to: 1
        move: face.burstMove
        onFinished: face.bursting = false
    }
}

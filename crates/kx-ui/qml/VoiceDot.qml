// The red dot while the microphone listens (design/voice-bar.css .dot): it pulses each `[moves] dot` down to
// `pulse_low`; high contrast draws it in Highlight, a system colour (drawn/voice.rs).
import QtQuick

Rectangle {
    id: dot

    // The look and size.
    required property var lk
    required property real s
    // Its size and the move of one pulse; Snip's record dot gives its own.
    property real px: lk.shape.caption.dot_px * s
    property var move: lk.motion.moves.dot

    // The pulse runs, for the tests.
    readonly property bool pulsing: pulse.running

    width: px
    height: width
    radius: width / 2
    color: lk.contrast ? lk.palette.ring : lk.common.rec_dot

    SequentialAnimation on opacity {
        id: pulse
        running: dot.visible && dot.move.ms > 0
        loops: Animation.Infinite
        onStopped: dot.opacity = 1

        Tween {
            move: dot.move
            share: 0.5
            to: dot.lk.motion.amount.pulse_low
        }
        Tween {
            move: dot.move
            share: 0.5
            to: 1
        }
    }
}

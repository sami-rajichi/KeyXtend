// The part of a line that fills its parent: while it runs (voice-bar.css .prog.run) it slides end to end and back,
// and reduced motion fills it whole, dimmed and still. The voice panel's progress and Snip's waiting key share it.
import QtQuick

Rectangle {
    id: part

    // The look and colour; whether it runs, and the share it fills while it does not.
    required property var lk
    required property color ink
    property bool running: true
    property real share: 1

    // The run's share and length keep the voice panel's token names, where the line began; Snip shares them.
    readonly property var run: lk.shape.voice.run_share
    readonly property bool still: lk.motion.moves.run.ms <= 0
    readonly property real lane: parent ? parent.width : 0

    width: lane * (!running ? Math.max(0, Math.min(1, share)) : still ? 1 : run[0])
    height: parent ? parent.height : 0
    radius: height / 2
    opacity: running && still ? run[1] : 1
    color: ink

    SequentialAnimation {
        running: part.running && !part.still && part.visible
        loops: Animation.Infinite
        onStopped: part.x = 0
        Tween {
            target: part
            property: "x"
            from: 0
            to: part.lane - part.width
            move: part.lk.motion.moves.run
        }
        Tween {
            target: part
            property: "x"
            from: part.lane - part.width
            to: 0
            move: part.lk.motion.moves.run
        }
    }
}

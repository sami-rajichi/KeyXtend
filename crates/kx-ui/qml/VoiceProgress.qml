// A download's or a check's progress (mock-up .prog): its share filled in the ring colour on a pop_hover track, or in
// sel_ink on a lit row, as the ring fades on ET66's ticked row; high contrast draws the track in Window
// (drawn/voicepanel.rs). While a model writes (.prog.run) a part of it slides end to end and back; reduced motion
// fills it dimmed and still. Readers hear the line above it, so it says nothing itself.
import QtQuick

Rectangle {
    id: bar

    // The share done, 0 to 1; whether its row is under the pointer or ticked; the look and size.
    required property real share
    required property bool lit
    required property var lk
    required property real s
    // It runs with no share known, as while a model writes.
    property bool running: false

    readonly property color track: lk.contrast ? lk.palette.pop_bg : lk.palette.pop_hover
    readonly property color fill: lit && !lk.contrast ? lk.sel_ink : lk.palette.ring
    readonly property var run: lk.shape.voice.run_share
    readonly property bool still: lk.motion.moves.run.ms <= 0
    readonly property alias part: part

    height: lk.shape.voice.progress_px * s
    radius: height / 2
    color: track
    border.width: lk.shape.line.px
    border.color: lk.palette.pop_line
    Accessible.ignored: true

    Rectangle {
        id: part
        width: parent.width * (!bar.running ? Math.max(0, Math.min(1, bar.share)) : bar.still ? 1 : bar.run[0])
        height: parent.height
        radius: parent.radius
        opacity: bar.running && bar.still ? bar.run[1] : 1
        color: bar.fill
    }
    SequentialAnimation {
        running: bar.running && !bar.still && bar.visible
        loops: Animation.Infinite
        onStopped: part.x = 0
        Tween {
            target: part
            property: "x"
            from: 0
            to: bar.width - part.width
            move: bar.lk.motion.moves.run
        }
        Tween {
            target: part
            property: "x"
            from: bar.width - part.width
            to: 0
            move: bar.lk.motion.moves.run
        }
    }
}

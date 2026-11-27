// A download's or a check's progress (mock-up .prog): its share filled in the ring colour on a pop_hover track, or in
// sel_ink on a lit row, as the ring fades on ET66's ticked row; high contrast draws the track in Window
// (drawn/voicepanel.rs). While a model writes, its `RunLine` part slides. Readers hear the line above it, so it says
// nothing itself.
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
    readonly property alias part: part

    height: lk.shape.voice.progress_px * s
    radius: height / 2
    color: track
    border.width: lk.shape.line.px
    border.color: lk.palette.pop_line
    Accessible.ignored: true

    RunLine {
        id: part
        lk: bar.lk
        ink: bar.fill
        running: bar.running
        share: bar.share
    }
}

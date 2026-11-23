// The live wave while the bar listens (design/voice-bar.css .wave, voice-bar-scene.js frame()): bars in the ring
// colour as many as fit, each rising from rest with the level smoothed over `waveMs`, a ripple running across them;
// reduced motion draws a plain meter filled to the level instead (.meter).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: w

    // The look and size, the smoothing in ms (0 takes each level as it comes), whether it listens, and the clock in ms.
    required property var lk
    required property real s
    required property int waveMs
    required property bool live
    property var now: () => Date.now()

    // The smoothed level, 0 to 1, when the last level came, and the ripple's turn in radians.
    property real level: 0
    property real heardAt: 0
    property real phase: 0

    // The wave and the meter, for the tests.
    readonly property alias wave: wave
    readonly property alias meter: meter
    readonly property alias fill: fill
    readonly property bool rippling: ripple.running
    readonly property var px: lk.shape.voice.wave_px
    readonly property var amount: lk.motion.amount
    readonly property bool still: lk.motion.moves.wave.ms === 0
    // As many bars as fit, at least one.
    readonly property int count: Math.max(1, Math.floor((width + px[1] * s) / ((px[0] + px[1]) * s)))

    implicitHeight: px[3] * s

    // Starts again from silence, as a new listening starts.
    function reset() {
        level = 0;
        heardAt = now();
    }
    // A new level `v` came: the smoothed level moves towards it by how long since the last one.
    function heard(v) {
        const t = now();
        const k = waveMs > 0 ? 1 - Math.exp(-Math.max(0, t - heardAt) / waveMs) : 1;
        heardAt = t;
        level += (v - level) * k;
    }
    // Bar `n`'s height: rest, plus its share of the level as the ripple passes it (frame()'s .4 + .6 * |sin|).
    function heightOf(n) {
        const share = amount.wave_floor + (1 - amount.wave_floor) * Math.abs(Math.sin(n * amount.wave_step + phase));
        return (px[2] + (px[3] - px[2]) * level * share) * s;
    }

    Row {
        id: wave
        anchors.centerIn: parent
        visible: !w.still
        spacing: w.px[1] * w.s
        Repeater {
            model: w.count
            delegate: Rectangle {
                required property int index
                readonly property bool bar: true
                anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                width: w.px[0] * w.s
                height: w.heightOf(index)
                radius: width / 2
                color: w.lk.palette.ring
            }
        }
    }
    // High contrast draws the track in Window, so the Highlight fill is a system pair (drawn/voice.rs).
    Rectangle {
        id: meter
        anchors.verticalCenter: parent.verticalCenter
        visible: w.still
        width: w.width
        height: w.lk.shape.caption.shimmer_px[0] * w.s
        radius: w.lk.shape.caption.shimmer_px[1] * w.s
        color: w.lk.contrast ? w.lk.palette.pop_bg : w.lk.palette.pop_hover
        border.width: w.lk.shape.line.px
        border.color: w.lk.palette.pop_line
        Rectangle {
            id: fill
            width: meter.width * w.level
            height: meter.height
            radius: meter.radius
            color: w.lk.palette.ring
        }
    }

    NumberAnimation on phase {
        id: ripple
        running: w.live && w.visible && !w.still
        loops: Animation.Infinite
        from: 0
        to: 2 * Math.PI
        duration: w.lk.motion.moves.wave.ms
    }
}

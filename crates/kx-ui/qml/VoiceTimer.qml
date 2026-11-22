// How long the bar has listened (design/voice-bar.css .tm): minutes and two-digit seconds in the language's order, as
// it counts on from the module's time, ticking at each whole second, in the panel's count style at one width to 99:59.
import QtQuick

Label {
    id: t

    // The look, fonts and size, the timer's words with Qt's marks (`%1` minutes, `%2` seconds), the module's time when
    // the view came (ms), when it came on the clock, and the clock.
    required property var lk
    required property var res
    required property real s
    required property string format
    required property real from
    required property real at
    property var now: () => Date.now()

    // The time shown in ms, and the tick, for the tests.
    property real elapsed: from
    readonly property alias tick: tick
    // Units of time, and the digits of the seconds and of the widest minutes the timer is measured at.
    readonly property int msPerS: 1000
    readonly property int sPerMin: 60
    readonly property int digits: 2

    text: shown(elapsed)
    width: Math.ceil(widest.advanceWidth)
    horizontalAlignment: Text.AlignHCenter
    font.family: res.fam.latin
    font.pixelSize: lk.shape.panel.count_px * s
    font.weight: lk.shape.panel.count_weight
    color: lk.palette.pop_muted

    onFromChanged: count()
    onAtChanged: count()
    Component.onCompleted: count()

    // The time `ms` as the words order its minutes and two-digit seconds; nothing before any words came.
    function shown(ms) {
        const all = Math.floor(ms / msPerS);
        if (format === "")
            return "";
        return format.arg(Math.floor(all / sPerMin)).arg(String(all % sPerMin).padStart(digits, "0"));
    }
    // Reads the clock, shows the time and waits for the next whole second.
    function count() {
        elapsed = from + Math.max(0, now() - at);
        tick.interval = msPerS - elapsed % msPerS;
        if (visible)
            tick.restart();
    }

    // Two-digit minutes and seconds, each digit as wide as a 0, so the text never moves or widens as it counts.
    TextMetrics {
        id: widest
        font: t.font
        text: t.format === "" ? "" : t.format.arg("0".repeat(t.digits)).arg("0".repeat(t.digits))
    }
    Timer {
        id: tick
        onTriggered: t.count()
    }
    onVisibleChanged: if (visible) count(); else tick.stop()
}

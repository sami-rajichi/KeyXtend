// The bar's middle (design/voice-bar.css .vbar): the wave while it listens, the shimmer while it writes, then the
// text or the note with its line.
import QtQuick

Item {
    id: mid

    // The bar it sits in.
    required property var host
    // The wave, for the bar and the tests, and the height the shown part needs, read off the bar's `mode` itself as all
    // that sizes the bar is.
    readonly property alias wave: wave
    readonly property var states: host.names.states
    readonly property real need: host.mode === states.listening ? wave.implicitHeight
        : host.mode === states.transcribing ? shim.height : words.implicitHeight

    height: need

    VoiceWave {
        id: wave
        visible: mid.host.listens
        width: parent.width
        anchors.verticalCenter: parent.verticalCenter
        lk: mid.host.lk
        s: mid.host.s
        waveMs: mid.host.field("waveMs", 0)
        live: mid.host.listens && mid.host.visible
        now: mid.host.now
    }
    Shimmer {
        id: shim
        visible: mid.host.writes && mid.host.visible
        width: parent.width
        height: mid.host.cap.shimmer_px[0] * mid.host.s
        anchors.verticalCenter: parent.verticalCenter
        lk: mid.host.lk
        s: mid.host.s
    }
    VoiceNote {
        id: words
        // The text and the line, read from the view alone, as a state change reaches bindings in any order.
        readonly property var said: mid.host.field("text", null)
        readonly property var told: mid.host.field("hint", null)
        visible: mid.host.wide
        width: parent.width
        anchors.verticalCenter: parent.verticalCenter
        lk: mid.host.lk
        res: mid.host.res
        s: mid.host.s
        ar: mid.host.ar
        text: said ? said.text : (told && told.text ? told.text : "")
        rtl: !!said && said.rtl
        arabic: !!said && said.arabic
        masked: !!said && said.masked
        note: !said && !!told && !!told.text
        line: !!said && !!told ? told.text : ""
    }
}

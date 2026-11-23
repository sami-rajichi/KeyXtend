// The mark before a text or a note, a button wide (design/voice-bar.css .slot): the text's or the dots' mark, each
// note's own by its hint id, or the red dot while the microphone is still open, so a note never hides it.
import QtQuick

Item {
    id: slot

    // The bar it sits on, whose view, look and size it reads.
    required property var host

    readonly property var shown: host.shown
    readonly property var icons: host.lk.icons.voice
    readonly property bool mic: !!shown && !!shown.mic

    width: host.lk.shape.clip.button_px[0] * host.s
    height: width

    VoiceDot {
        visible: slot.mic
        anchors.centerIn: parent
        lk: slot.host.lk
        s: slot.host.s
    }
    Icon {
        visible: !slot.mic
        anchors.centerIn: parent
        icon: slot.mark()
        base: slot.host.res.icons
        ext: slot.host.res.ext
        px: slot.host.cap.icon_px * slot.host.s
        tint: slot.host.p.pop_muted
    }

    // A text's mark, the dots', or the note's by its hint id, else the plain note mark; none in other states.
    function mark() {
        if (shown && shown.text)
            return shown.text.masked ? icons.password : icons.text;
        if (shown && shown.state === host.names.states.note)
            return icons.notes[shown.hint.id] || icons.note;
        return "";
    }
}

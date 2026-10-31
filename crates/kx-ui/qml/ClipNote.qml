// The note band under the clipboard panel's header (spec §7): why a copy was not kept, or an insert failed. It rises in
// when a new note comes and takes no room without one.
import QtQuick

Item {
    id: note

    // The note's text, or null; the look, fonts and size.
    required property var text
    required property var lk
    required property var res
    required property real s

    readonly property var sh: lk.shape.clip.note_px
    readonly property real side: lk.shape.panel.foot_pad_px[1] * s

    visible: !!text
    height: visible ? band.height + lk.shape.panel.body_gap_px * s : 0
    onTextChanged: if (text) rise.play()
    Accessible.role: Accessible.StaticText
    Accessible.name: text || ""
    Accessible.ignored: !text

    Rise {
        id: rise
        x: note.side
        y: note.lk.shape.panel.body_gap_px * note.s
        width: note.width - 2 * note.side
        height: band.height
        move: note.lk.motion.clip.swap
        from: note.lk.motion.amount.pop_rise_px * note.s
        // On pop_hover, which is Highlight in high contrast, so the text takes sel_ink.
        Rectangle {
            id: band
            width: parent.width
            height: line.height + 2 * note.sh[0] * note.s
            radius: note.sh[1] * note.s
            color: note.lk.palette.pop_hover
            Label {
                id: line
                x: note.sh[0] * note.s
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 2 * x
                text: note.text || ""
                wrapMode: Text.Wrap
                font.family: note.res.fam.ui
                font.pixelSize: note.sh[2] * note.s
                color: note.lk.sel_ink
            }
        }
    }
}

// The words on a dictation bar (design/voice-bar.html .vtx, .mask, .msg): a text in its own direction and font cut to
// its first `[voice] text_lines` lines, a password box's dots, or a note's line, which wraps the same way so a long
// fault reads in full lines; a hint line, such as Password or Typing stopped, sits under a text.
import QtQuick

Item {
    id: n

    // The look, fonts and size, whether the keyboard speaks Arabic, the words with their direction and font, whether
    // they are dots or a note's line, and the hint line under them (empty for none).
    required property var lk
    required property var res
    required property real s
    required property bool ar
    required property string text
    property bool rtl: false
    property bool arabic: false
    property bool masked: false
    property bool note: false
    property string line: ""

    // The words and the hint line, for the tests.
    readonly property alias words: words
    readonly property alias hint: hint
    readonly property var cap: lk.shape.caption
    readonly property real scale: ar ? lk.look.arabic_scale : 1
    // A note's line reads in the keyboard's language; a text in its own.
    readonly property bool rightward: note ? ar : rtl
    readonly property real pad: note ? 0 : cap.words_pad_px * s

    implicitHeight: column.height + 2 * pad

    Column {
        id: column
        x: n.cap.inset_px * n.s
        y: n.pad
        width: n.width - 2 * n.cap.inset_px * n.s
        Label {
            id: words
            width: parent.width
            text: n.text
            wrapMode: Text.Wrap
            maximumLineCount: n.lk.shape.voice.text_lines
            elide: Text.ElideRight
            horizontalAlignment: n.rightward ? Text.AlignRight : Text.AlignLeft
            font.family: n.note ? n.res.fam.ui : (n.arabic ? n.res.fam.arabic : n.res.fam.latin)
            font.pixelSize: n.size()
            // A note's line keeps the bar's status weight (.msg in .vbar); a text is plain (.vtx).
            font.weight: n.note ? n.cap.status_weight : Font.Normal
            font.letterSpacing: n.masked ? n.lk.shape.fill.mask_em * n.size() : 0
            color: n.lk.palette.pop_ink
        }
        Label {
            id: hint
            visible: n.line !== ""
            width: parent.width
            topPadding: n.lk.shape.panel.hint_px[1] * n.s
            text: n.line
            wrapMode: Text.Wrap
            horizontalAlignment: n.ar ? Text.AlignRight : Text.AlignLeft
            font.family: n.res.fam.ui
            font.pixelSize: n.lk.shape.panel.hint_px[0] * n.s * n.scale
            color: n.lk.palette.pop_muted
        }
    }

    // A note's line takes the notice's size in the keyboard's font; a text its own font's words size.
    function size() {
        if (note)
            return lk.shape.notice.font_px * s * scale;
        return cap.words_px[arabic ? 1 : 0] * s;
    }
}

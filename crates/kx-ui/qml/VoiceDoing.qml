// What a row is doing now, at its end (mock-up .chk): a short line over its own progress, the line ending at the row's
// end in its language's way; readers hear the line.
import QtQuick

Column {
    id: doing

    // The row it sits on, and what it is doing.
    required property var owner
    required property string text

    width: owner.lk.shape.voice.value_px * owner.s
    spacing: owner.pn.row_pad_px * owner.s

    Label {
        width: parent.width
        text: doing.text
        elide: Text.ElideRight
        horizontalAlignment: doing.owner.host.ar ? Text.AlignLeft : Text.AlignRight
        font.family: doing.owner.res.fam.ui
        font.pixelSize: doing.owner.pn.hint_px[0] * doing.owner.s * doing.owner.scale
        color: doing.owner.lit ? doing.owner.lk.sel_ink : doing.owner.p.pop_ink
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}

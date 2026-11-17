// A row of Compare's table (mock-up results()): one tick and the model's name, its own text under it when there was no
// set text, then its words wrong, wait and memory, each in its column.
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    tick: entry ? entry.tick : undefined
    name: entry ? entry.name : ""
    line: entry && entry.text ? entry.text : ""
    onClicked: host.send(host.names.acts.row, "", place)

    Repeater {
        model: row.entry ? [row.entry.wrong, row.entry.wait, row.entry.memory] : []
        delegate: Label {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            width: row.lk.shape.voice.col_px * row.s
            text: modelData || ""
            horizontalAlignment: Text.AlignHCenter
            font.family: row.res.fam.ui
            font.pixelSize: row.pn.row_text_px[row.host.ar ? 1 : 0] * row.s
            color: row.lit ? row.lk.sel_ink : row.p.pop_ink
            Accessible.ignored: !text
            Accessible.role: Accessible.StaticText
            Accessible.name: text
        }
    }
}

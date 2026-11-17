// The column heads over Compare's table (mock-up .pr.head): the model's above the names, past the tick box, then each
// number's above its column, small and muted, as the rows lay them out.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: heads

    // The list, whose view, look and fonts it takes.
    required property var list

    readonly property var host: list.host
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var titles: list.sheet.head || []
    readonly property real gap: pn.row_gap_px * s

    height: visible ? first.implicitHeight + 2 * pn.row_pad_px * s : 0

    Head {
        id: first
        x: (heads.pn.row_px[2] + heads.lk.shape.clip.tick_px[0]) * heads.s + heads.gap
        width: cols.x - x - heads.gap
        text: heads.titles[0] || ""
        horizontalAlignment: heads.host.ar ? Text.AlignRight : Text.AlignLeft
    }
    Row {
        id: cols
        anchors.right: parent.right
        anchors.rightMargin: heads.pn.row_px[2] * heads.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: heads.gap
        Repeater {
            model: heads.titles.slice(1)
            delegate: Head {
                required property var modelData
                width: heads.lk.shape.voice.col_px * heads.s
                text: modelData
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }

    // One head, in the hint size and the page count's weight.
    component Head: Label {
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        elide: Text.ElideRight
        font.family: heads.host.res.fam.ui
        font.pixelSize: heads.pn.hint_px[0] * heads.s * (heads.host.ar ? heads.lk.look.arabic_scale : 1)
        font.weight: heads.pn.count_weight
        color: heads.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.ColumnHeader
        Accessible.name: text
    }
}

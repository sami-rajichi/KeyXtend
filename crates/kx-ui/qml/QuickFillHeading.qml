// What a Quick-fill page shows, in its first line over the rows: a tile with the kind's or the label's icon, and the
// entry's or the label's name in the panel's title size. A name reads in its own direction and font (ADR-0020).
import QtQuick

Row {
    id: heading

    // The icon, the name (`{text, rtl, arabic}`), the room it may take, the look, fonts, icons and size.
    required property string icon
    required property var title
    required property real room
    required property var lk
    required property var res
    required property real s

    readonly property var pn: lk.shape.panel

    spacing: lk.shape.panel.row_gap_px * s

    PanelTile {
        id: tile
        anchors.verticalCenter: parent.verticalCenter
        icon: heading.icon
        lk: heading.lk
        res: heading.res
        s: heading.s
    }
    Label {
        anchors.verticalCenter: parent.verticalCenter
        width: heading.room - tile.width - heading.spacing
        text: heading.title ? heading.title.text : ""
        elide: Text.ElideRight
        horizontalAlignment: heading.title && heading.title.rtl ? Text.AlignRight : Text.AlignLeft
        font.family: heading.title && heading.title.arabic ? heading.res.fam.arabic : heading.res.fam.ui
        font.pixelSize: heading.pn.row_text_px[heading.title && heading.title.arabic ? 1 : 0] * heading.s
        font.weight: heading.pn.title_weight
        color: heading.lk.palette.pop_ink
        Accessible.ignored: false
        Accessible.role: Accessible.Heading
        Accessible.name: text
    }
}

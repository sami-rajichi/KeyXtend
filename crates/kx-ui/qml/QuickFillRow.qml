// One Quick-fill row (design "The panel"; mock-up .lrow): its tile, its title over its hint, and its buttons; a click
// on the row itself does nothing, so a slip never types, and a padding slot stays blank. An Arabic title reads from the
// right in its font while each part keeps its place (ADR-0020); a hidden value shows the mask.
import QtQuick

QuickFillSlot {
    id: r

    // The slot holds `{key, icon, title, hint, buttons}`: the title and hint are `{text, rtl, arabic}`, a null title is
    // the mask and a null hint none, and a button may name its icon and wait. Then the click names, the mask, the fonts
    // and icons.
    required property var names
    required property string mask
    required property var res

    // Button `id` was clicked on this row.
    signal pressed(string id)

    readonly property var fs: lk.shape.fill
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property var ic: lk.icons.fill
    readonly property var title: filled ? slot.title : null
    readonly property var hint: filled ? slot.hint : null
    readonly property bool masked: filled && title === null
    readonly property bool ar: !!title && title.arabic
    readonly property real gap: pn.row_gap_px * s
    readonly property string said: masked ? mask : title ? title.text : ""

    key: filled ? String(slot.key) : ""
    Accessible.role: Accessible.ListItem
    Accessible.name: filled ? said : ""
    Accessible.description: hint ? hint.text : ""
    Accessible.id: filled ? res.uid(names.group, names.ids.row + place) : ""

    PanelTile {
        id: tile
        anchors.left: parent.left
        anchors.leftMargin: r.lk.shape.panel.row_px[2] * r.s
        anchors.verticalCenter: parent.verticalCenter
        icon: r.filled ? r.slot.icon : ""
        lk: r.lk
        res: r.res
        s: r.s
    }
    Column {
        anchors.left: tile.right
        anchors.leftMargin: r.gap
        anchors.right: trail.left
        anchors.rightMargin: r.gap
        anchors.verticalCenter: parent.verticalCenter
        // User text, so its direction is the hub's; a long one ends in an ellipsis.
        Label {
            width: parent.width
            text: r.said
            elide: Text.ElideRight
            horizontalAlignment: r.title && r.title.rtl ? Text.AlignRight : Text.AlignLeft
            font.family: r.ar ? r.res.fam.arabic : r.res.fam.latin
            font.pixelSize: r.lk.shape.panel.row_text_px[r.ar ? 1 : 0] * r.s
            font.letterSpacing: r.masked ? r.fs.mask_em * r.lk.shape.panel.row_text_px[0] * r.s : 0
            color: r.masked ? r.p.pop_muted : r.p.pop_ink
        }
        Label {
            visible: !!r.hint
            width: parent.width
            topPadding: r.pn.hint_px[1] * r.s
            text: r.hint ? r.hint.text : ""
            elide: Text.ElideRight
            horizontalAlignment: r.hint && r.hint.rtl ? Text.AlignRight : Text.AlignLeft
            font.family: r.hint && r.hint.arabic ? r.res.fam.arabic : r.res.fam.latin
            font.pixelSize: r.pn.hint_px[0] * r.s
            color: r.p.pop_muted
        }
    }
    QuickFillButtons {
        id: trail
        anchors.right: parent.right
        anchors.rightMargin: r.lk.shape.panel.row_px[2] * r.s
        anchors.verticalCenter: parent.verticalCenter
        buttons: !r.filled ? [] : r.slot.buttons.map(b => ({
                    id: b.id,
                    label: b.label,
                    icon: b.icon || r.names.rowIcon(r.ic, b.id),
                    can: b.can,
                    uid: r.res.uid(r.names.group, r.names.ids.row + r.place + r.names.ids.part + b.id),
                    about: r.said
                }))
        lk: r.lk
        res: r.res
        s: r.s
        onClicked: id => r.pressed(id)
    }
}

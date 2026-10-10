// One row of the language list (mock-up .lg-it): the short-name badge, the language over its layout, and a tick on
// the layout shown now. The highlighted row wears the ring (.lrow.hi); a padding row is blank and takes no clicks.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: r

    // The row's layout (`{short, name, rtl, arabic, current, icon}`, or null for padding), its place in the whole list,
    // whether it is highlighted, the look, fonts, icons and texts, and the keyboard size.
    required property var row
    required property int place
    required property bool hi
    required property var lk
    required property var res
    required property real s

    // The row was clicked or picked by a screen reader.
    signal picked

    // A layout, not a padding row.
    readonly property bool filled: row !== null
    readonly property var sh: lk.shape.list
    readonly property var p: lk.palette
    readonly property bool ar: filled && row.arabic
    readonly property bool hot: filled && area.containsMouse
    // Hover text sits on pop_hover, which is Highlight in high contrast, so it takes sel_ink there.
    readonly property color ink: hot ? lk.sel_ink : p.pop_ink

    Accessible.role: Accessible.ListItem
    Accessible.name: filled ? res.space([row.short].concat(row.name)) : ""
    Accessible.ignored: !filled
    Accessible.selectable: true
    Accessible.selected: hi
    Accessible.checkable: true
    Accessible.checked: filled && row.current
    Accessible.onPressAction: if (r.filled) r.picked()
    // UI Automation's Toggle on a checkable row asks Qt for this action, not the press.
    Accessible.onToggleAction: if (r.filled) r.picked()

    Hover {
        lk: r.lk
        radius: r.sh.row_radius_px * r.s
        visible: r.hot
        tint: r.p.pop_hover
    }
    // The highlight ring, inside the row; on a hovered row it takes the hover's text colour, which high contrast needs.
    Rectangle {
        anchors.fill: parent
        visible: r.filled && r.hi
        radius: r.sh.row_radius_px * r.s
        color: "transparent"
        border.width: r.lk.shape.press.ring_px * r.s
        border.color: r.hot ? r.lk.sel_ink : r.p.ring
    }
    Row {
        visible: r.filled
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: r.sh.row_pad_px[1] * r.s
        anchors.rightMargin: anchors.leftMargin
        anchors.verticalCenter: parent.verticalCenter
        spacing: r.sh.gap_px * r.s

        Rectangle {
            id: badge
            anchors.verticalCenter: parent.verticalCenter
            width: r.sh.badge_px[0] * r.s
            height: r.sh.badge_px[1] * r.s
            radius: r.sh.badge_px[2] * r.s
            color: r.p.pop_hover
            // On pop_hover, which is Highlight in high contrast, so the text takes sel_ink.
            Label {
                anchors.centerIn: parent
                text: r.filled ? r.row.short : ""
                font.family: r.ar ? r.res.fam.arabic : r.res.fam.latin
                font.pixelSize: r.sh.badge_text_px[r.ar ? 1 : 0] * r.s
                font.weight: r.sh.badge_weight
                color: r.lk.sel_ink
            }
        }
        Column {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - badge.width - tick.width - 2 * parent.spacing
            // The OS's names go through the hub's `space`, which drops their hidden marks (ADR-0022).
            Label {
                width: parent.width
                text: r.filled ? r.res.space(r.row.name.slice(0, 1)) : ""
                elide: Text.ElideRight
                font.family: r.ar ? r.res.fam.arabic : r.res.fam.latin
                font.pixelSize: r.sh.name_px * r.s
                color: r.ink
            }
            Label {
                width: parent.width
                text: r.filled ? r.res.space(r.row.name.slice(1)) : ""
                elide: Text.ElideRight
                font.family: r.res.fam.latin
                font.pixelSize: r.sh.sub_px * r.s
                color: r.hot ? r.lk.sel_ink : r.p.pop_muted
            }
        }
        Icon {
            id: tick
            anchors.verticalCenter: parent.verticalCenter
            // The module names the icon's token, such as `lang.current`.
            icon: r.filled && r.row.icon ? r.row.icon.split(".").reduce((o, k) => o ? o[k] : "", r.lk.icons) : ""
            base: r.res.icons
            ext: r.res.ext
            px: r.sh.icon_px * r.s
            tint: r.hot ? r.lk.sel_ink : r.p.ring
        }
    }
    MouseArea {
        id: area
        anchors.fill: parent
        enabled: r.filled
        hoverEnabled: true
        onClicked: r.picked()
    }
}

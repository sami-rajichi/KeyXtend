// An icon button on a panel: plain (mock-up .ib, the close button) or framed (mock-up .pg, the page buttons).
import QtQuick

Rectangle {
    id: btn

    // The look and fonts, the button (icon and name), its icon size, and whether it has a frame.
    required property var lk
    required property var res
    required property var button
    required property real iconPx
    property bool framed: false

    signal clicked
    // The pointer came onto the button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var pal: lk.palette
    readonly property bool lit: area.containsMouse && enabled
    // High contrast paints pop_hover as Highlight, so a framed button rests on the panel colour there.
    readonly property color rest: framed ? (lk.contrast ? pal.pop_bg : pal.pop_hover) : "transparent"

    opacity: enabled ? 1 : lk.shape.panel.off_share
    color: lit ? (framed ? pal.pop_sel : pal.pop_hover) : rest
    border.width: framed ? lk.shape.line.px : 0
    border.color: pal.pop_line
    Accessible.role: Accessible.Button
    Accessible.name: button.name
    Accessible.onPressAction: if (enabled) btn.clicked()

    Icon {
        anchors.centerIn: parent
        icon: btn.button.icon
        base: btn.res.icons
        ext: btn.res.ext
        px: btn.iconPx
        tint: btn.lit ? btn.lk.sel_ink : (btn.framed ? btn.pal.pop_ink : btn.pal.pop_muted)
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: btn.tip(containsMouse ? btn.button.name : "", btn)
        onClicked: if (btn.enabled) btn.clicked()
    }
}

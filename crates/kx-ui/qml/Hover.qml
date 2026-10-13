// What marks a control under the pointer: the theme's tint and, in high contrast, an outline inside it in Highlight,
// since a tint is not a system colour. Fill its control and show it while the pointer is over it.
import QtQuick

Rectangle {
    // The look, and the tint this control takes.
    required property var lk
    property color tint: lk.palette.hover

    anchors.fill: parent
    color: tint
    border.width: lk.contrast ? lk.shape.hc_outline.px : 0
    border.color: lk.hover_line
}

// A thin line between groups of keys in Snip's editor and panel (snip-editor.js .sep), as tall as a key.
import QtQuick

Item {
    id: sep

    // The look, and the size it is drawn at.
    required property var lk
    property real s: 1

    width: (2 * lk.shape.snip.sep_px + lk.shape.line.px) * s
    height: lk.shape.clip.button_px[0] * s

    Rectangle {
        anchors.centerIn: parent
        width: sep.lk.shape.line.px * sep.s
        height: parent.height
        color: sep.lk.palette.pop_line
    }
}

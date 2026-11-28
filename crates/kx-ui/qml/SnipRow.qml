// One row of the Snip panel (snip-editor.js prow()): its label, its keys at the end, and a muted line under them that
// says what the choice does, if any.
import QtQuick

Item {
    id: row

    // The panel, the row's label and its line (empty for none); the keys go inside.
    required property var panel
    required property string label
    property string say: ""
    default property alias keys: keys.data

    readonly property var lk: panel.lk
    readonly property real s: panel.s
    readonly property real pad: lk.shape.snip.editor_pad_px[0] * s
    readonly property real word: lk.shape.clip.button_px[3] * s

    width: parent ? parent.width : 0
    height: Math.max(named.implicitHeight, keys.implicitHeight) + (say ? said.implicitHeight + pad : 0) + 2 * pad

    Label {
        id: named
        x: row.pad
        y: row.pad + (Math.max(height, keys.implicitHeight) - height) / 2
        width: row.width - keys.implicitWidth - 3 * row.pad
        text: row.label
        wrapMode: Text.Wrap
        font.family: row.panel.res.fam.ui
        font.pixelSize: row.word
        color: row.lk.palette.pop_ink
    }
    Row {
        id: keys
        anchors.right: parent.right
        anchors.rightMargin: row.pad
        y: row.pad
        spacing: row.lk.shape.clip.button_px[5] * row.s
        layoutDirection: Qt.LeftToRight
    }
    Label {
        id: said
        visible: !!row.say
        x: row.pad
        y: row.pad + Math.max(named.implicitHeight, keys.implicitHeight) + row.pad
        width: row.width - 2 * row.pad
        text: row.say
        wrapMode: Text.Wrap
        font.family: row.panel.res.fam.ui
        font.pixelSize: row.word
        color: row.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}

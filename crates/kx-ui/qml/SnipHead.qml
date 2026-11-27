// The editor's title bar (snip-editor.js .ed-title): the grip that moves it, Snip's mark and the editor's name, then
// Maximise or Restore and Close. While maximised it neither moves nor resizes, as a maximised window does not.
import QtQuick

Item {
    id: head

    // The editor.
    required property var editor

    readonly property var lk: editor.lk
    readonly property var res: editor.res
    readonly property var sh: lk.shape.snip
    readonly property var n: editor.n
    readonly property var ic: lk.icons.snip.edit
    readonly property var pad: sh.editor_pad_px

    height: sh.editor_rows_px[0]

    Row {
        x: head.pad[0]
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.sh.icon_gap_px
        layoutDirection: Qt.LeftToRight
        SnipButton {
            visible: !head.editor.full
            lk: head.lk
            res: head.res
            name: head.res.tr("snip.edit-" + head.n.own.move)
            icon: head.ic.move
            uid: "snip.edit." + head.n.own.move
            tip: head.editor.hint
            onClicked: head.editor.mover.startMove()
        }
        Icon {
            anchors.verticalCenter: parent.verticalCenter
            icon: head.lk.icons.keys.snip
            base: head.res.icons
            ext: head.res.ext
            px: head.sh.icon_px
            tint: head.lk.palette.pop_ink
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            text: head.res.tr("snip.editor")
            font.family: head.res.fam.ui
            font.pixelSize: head.lk.shape.clip.button_px[3]
            font.weight: head.lk.shape.clip.button_weight
            color: head.lk.palette.pop_ink
        }
    }
    Row {
        anchors.right: parent.right
        anchors.rightMargin: head.pad[0]
        anchors.verticalCenter: parent.verticalCenter
        layoutDirection: Qt.LeftToRight
        SnipButton {
            readonly property string way: head.editor.full ? head.n.own.back : head.n.own.full
            lk: head.lk
            res: head.res
            name: head.res.tr("snip.edit-" + way)
            icon: head.ic[way]
            tone: head.editor.full ? "shown" : "flat"
            uid: "snip.edit." + head.n.own.full
            tip: head.editor.hint
            onClicked: head.editor.maximise()
        }
        SnipButton {
            lk: head.lk
            res: head.res
            name: head.res.tr("snip.edit-" + head.n.edit.close)
            icon: head.ic.close
            uid: "snip.edit." + head.n.edit.close
            tip: head.editor.hint
            onClicked: head.editor.snips.act(head.n.acts.edit, head.n.edit.close)
        }
    }
}

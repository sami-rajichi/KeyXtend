// The clipboard panel's header (spec §7; mock-up .clip header): its icon, the title over the kept-days line, Delete all
// and Close, with a line under it.
import QtQuick

Item {
    id: head

    // The panel's view, the click names, whether it waits on a question or a preview, the look, fonts, icons and size.
    required property var view
    required property var names
    required property bool busy
    required property var lk
    required property var res
    required property real s

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property real gap: pn.head_gap_px * s
    // Room for the title and kept line between the icon and the buttons.
    readonly property real room: width - (pn.head_pad_px[1] + pn.head_pad_px[3]) * s - glyph.width - ends.width - 2 * gap

    height: Math.max(ends.height, words.height) + (pn.head_pad_px[0] + pn.head_pad_px[2]) * s

    Row {
        anchors.left: parent.left
        anchors.leftMargin: head.pn.head_pad_px[3] * head.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.gap
        Icon {
            id: glyph
            anchors.verticalCenter: parent.verticalCenter
            icon: head.lk.icons.clip.title
            base: head.res.icons
            ext: head.res.ext
            px: head.pn.button_px[2] * head.s
            tint: head.p.pop_ink
        }
        Column {
            id: words
            anchors.verticalCenter: parent.verticalCenter
            Label {
                width: Math.min(implicitWidth, head.room)
                text: head.view.title
                elide: Text.ElideRight
                font.family: head.res.fam.ui
                font.pixelSize: head.pn.title_px * head.s
                font.weight: head.pn.title_weight
                color: head.p.pop_ink
            }
            Label {
                width: Math.min(implicitWidth, head.room)
                text: head.view.kept
                elide: Text.ElideRight
                font.family: head.res.fam.ui
                font.pixelSize: head.lk.shape.clip.kept_px * head.s
                color: head.p.pop_muted
            }
        }
    }
    Row {
        id: ends
        anchors.right: parent.right
        anchors.rightMargin: head.pn.head_pad_px[1] * head.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.gap
        // Ruling 34: with nothing unpinned it shows dimmed; it waits while a question or a preview shows.
        ClipButton {
            label: head.view.words.deleteAll
            icon: head.lk.icons.clip.delete
            tone: "ghost"
            can: head.view.anyUnpinned && !head.busy
            lk: head.lk
            res: head.res
            s: head.s
            uid: head.res.uid(head.names.group, head.names.ids.deleteAll)
            onClicked: head.act(head.names.acts.deleteAll, 0)
        }
        ClipButton {
            label: ""
            icon: head.lk.icons.clip.close
            tone: "flat"
            can: true
            lk: head.lk
            res: head.res
            s: head.s
            uid: head.res.uid(head.names.group, head.names.ids.close)
            Accessible.name: head.view.words.close
            onClicked: head.act(head.names.acts.close, 0)
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: head.lk.shape.line.px
        color: head.p.pop_line
    }
}

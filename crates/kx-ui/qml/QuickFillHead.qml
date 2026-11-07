// Quick-fill's header (design "The panel"; mock-up .pop-h): the key icon, the title, or New and Edit's or the Backup
// page's own, then New, Backup and Close, with a line under it. New and Backup show dimmed while an entry is edited,
// as they would drop it.
import QtQuick

Item {
    id: head

    // The panel's view, the click names, the look, fonts, icons and size.
    required property var view
    required property var names
    required property var lk
    required property var res
    required property real s

    // A panel button was clicked, by its id.
    signal pressed(string id)

    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property var ic: lk.icons.fill
    readonly property var words: view.buttons
    readonly property var ids: names.buttons
    readonly property real gap: pn.head_gap_px * s
    readonly property var sheet: view.sheet
    // New and Edit, and the labels table to add a field, take neither New nor Backup.
    readonly property bool editing: sheet.is === names.pages.edit || sheet.is === names.pages.add
    // Room for the title between the icon and the buttons.
    readonly property real room: width - (pn.head_pad_px[1] + pn.head_pad_px[3]) * s - glyph.width - ends.width - 2 * gap

    height: Math.max(ends.height, title.height) + (pn.head_pad_px[0] + pn.head_pad_px[2]) * s

    Row {
        anchors.left: parent.left
        anchors.leftMargin: head.pn.head_pad_px[3] * head.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.gap
        Icon {
            id: glyph
            anchors.verticalCenter: parent.verticalCenter
            icon: head.ic.title
            base: head.res.icons
            ext: head.res.ext
            px: head.pn.button_px[2] * head.s
            tint: head.p.pop_ink
        }
        Label {
            id: title
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, head.room)
            text: typeof head.sheet.title === "string" ? head.sheet.title : head.sheet.is === head.names.pages.backup ? head.words[head.ids.backup] : head.view.title
            elide: Text.ElideRight
            font.family: head.res.fam.ui
            font.pixelSize: head.pn.title_px * head.s
            font.weight: head.pn.title_weight
            color: head.p.pop_ink
        }
    }
    Row {
        id: ends
        anchors.right: parent.right
        anchors.rightMargin: head.pn.head_pad_px[1] * head.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.gap
        ClipButton {
            label: head.words[head.ids.new]
            icon: head.ic.new
            tone: "ghost"
            can: !head.editing
            lk: head.lk
            res: head.res
            s: head.s
            uid: head.res.uid(head.names.group, head.ids.new)
            onClicked: head.pressed(head.ids.new)
        }
        ClipButton {
            label: head.words[head.ids.backup]
            icon: head.ic.backup
            tone: "ghost"
            can: !head.editing
            lk: head.lk
            res: head.res
            s: head.s
            uid: head.res.uid(head.names.group, head.ids.backup)
            onClicked: head.pressed(head.ids.backup)
        }
        ClipButton {
            label: ""
            icon: head.ic.close
            tone: "flat"
            can: true
            lk: head.lk
            res: head.res
            s: head.s
            uid: head.res.uid(head.names.group, head.ids.close)
            Accessible.name: head.words[head.ids.close]
            onClicked: head.pressed(head.ids.close)
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: head.lk.shape.line.px
        color: head.p.pop_line
    }
}

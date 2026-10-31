// The clipboard panel's storage line (spec §7): what the history takes and what the drive has free, beside Open folder,
// which shows the history's folder in the file manager.
import QtQuick

Item {
    id: room

    // The storage line, the folder button's label, the click names, the look, fonts, icons and size.
    required property string text
    required property string folder
    required property var names
    required property var lk
    required property var res
    required property real s

    // Open folder was clicked.
    signal open

    readonly property var sh: lk.shape.clip.room_px
    readonly property real side: lk.shape.panel.foot_pad_px[1] * s
    readonly property real gap: lk.shape.panel.head_gap_px * s

    height: Math.max(button.height, line.height) + 2 * sh[0] * s

    Icon {
        id: glyph
        x: room.side
        anchors.verticalCenter: parent.verticalCenter
        icon: room.lk.icons.clip.room
        base: room.res.icons
        ext: room.res.ext
        px: room.sh[2] * room.s
        tint: room.lk.palette.pop_muted
    }
    Label {
        id: line
        anchors.left: glyph.right
        anchors.leftMargin: room.gap
        anchors.right: button.left
        anchors.rightMargin: room.gap
        anchors.verticalCenter: parent.verticalCenter
        text: room.text
        wrapMode: Text.Wrap
        font.family: room.res.fam.ui
        font.pixelSize: room.sh[1] * room.s
        color: room.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    ClipButton {
        id: button
        anchors.right: parent.right
        anchors.rightMargin: room.side
        anchors.verticalCenter: parent.verticalCenter
        label: room.folder
        icon: room.lk.icons.clip.folder
        tone: "ghost"
        can: true
        lk: room.lk
        res: room.res
        s: room.s
        uid: room.res.uid(room.names.group, room.names.ids.folder)
        onClicked: room.open()
    }
}

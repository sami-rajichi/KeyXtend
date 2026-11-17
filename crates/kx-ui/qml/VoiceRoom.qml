// The Models page's folder line (mock-up .room): the drive icon, where the models are kept and the room free there,
// then Change and Open; it shows only while the page has a folder.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: room

    // The panel window, whose view, look and names it takes.
    required property var host

    readonly property var shown: host.view.sheet.room || null
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var sh: lk.shape.clip.room_px
    readonly property real side: lk.shape.panel.foot_pad_px[1] * s
    readonly property real gap: lk.shape.clip.foot_gap_px * s

    visible: shown !== null
    height: visible ? Math.max(acts.height, line.height) + 2 * sh[0] * s : 0

    Icon {
        id: glyph
        x: room.side
        anchors.verticalCenter: parent.verticalCenter
        icon: room.lk.icons.voice.panel.room
        base: room.host.res.icons
        ext: room.host.res.ext
        px: room.sh[2] * room.s
        tint: room.lk.palette.pop_muted
    }
    Label {
        id: line
        anchors.left: glyph.right
        anchors.leftMargin: room.gap
        anchors.right: acts.left
        anchors.rightMargin: room.gap
        anchors.verticalCenter: parent.verticalCenter
        text: room.shown ? room.shown.text : ""
        elide: Text.ElideMiddle
        horizontalAlignment: room.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: room.host.res.fam.ui
        font.pixelSize: room.sh[1] * room.s * (room.host.ar ? room.lk.look.arabic_scale : 1)
        color: room.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    Row {
        id: acts
        anchors.right: parent.right
        anchors.rightMargin: room.side
        anchors.verticalCenter: parent.verticalCenter
        spacing: room.gap
        Repeater {
            model: room.shown ? room.shown.buttons : []
            delegate: VoicePress {
                required property var modelData
                host: room.host
                button: modelData
                onClicked: room.host.press(modelData)
            }
        }
    }
}

// What a Quick-fill page shows where its rows would be: an icon over a whole line that readers hear, then any buttons
// it is given: the empty notebook's invitation, the Backup page's with no place, and a password page's ask.
import QtQuick

Column {
    id: invite

    // The icon, the line and its ink, the look, fonts, icons and size.
    required property string icon
    required property string text
    required property color ink
    required property var lk
    required property var res
    required property real s

    spacing: lk.shape.panel.body_gap_px * s

    Icon {
        anchors.horizontalCenter: parent.horizontalCenter
        icon: invite.icon
        base: invite.res.icons
        ext: invite.res.ext
        px: invite.lk.shape.panel.empty_px * invite.s
        tint: invite.lk.palette.pop_muted
    }
    Label {
        width: parent.width
        text: invite.text
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        font.family: invite.res.fam.ui
        font.pixelSize: invite.lk.shape.panel.title_px * invite.s
        color: invite.ink
        Accessible.ignored: !invite.visible
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}

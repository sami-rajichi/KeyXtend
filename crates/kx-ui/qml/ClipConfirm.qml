// Delete all's question (spec §7): it takes the list's place, names how many unpinned entries go, and only a click on
// Yes deletes them, as a destructive answer is never the default; No keeps them. It rises in as it comes.
import QtQuick

Item {
    id: ask

    // The question (`{text, count}`, or null), the Yes and No labels, the click names, the look, fonts, icons and size.
    required property var confirm
    required property string yes
    required property string no
    required property var names
    required property var lk
    required property var res
    required property real s

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    readonly property real side: lk.shape.panel.foot_pad_px[1] * s

    visible: !!confirm
    onVisibleChanged: if (visible) rise.play()
    Accessible.role: Accessible.AlertMessage
    Accessible.name: confirm ? confirm.text : ""

    // Takes every press and hover in the list's place while it shows.
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
    }
    Rise {
        id: rise
        anchors.fill: parent
        move: ask.lk.motion.clip.swap
        from: ask.lk.motion.amount.pop_rise_px * ask.s

        Column {
            anchors.centerIn: parent
            width: parent.width - 2 * ask.side
            spacing: ask.lk.shape.clip.question_gap_px * ask.s
            Icon {
                anchors.horizontalCenter: parent.horizontalCenter
                icon: ask.lk.icons.clip.delete
                base: ask.res.icons
                ext: ask.res.ext
                px: ask.lk.shape.clip.empty_px * ask.s
                tint: ask.lk.palette.pop_muted
            }
            Label {
                width: parent.width
                text: ask.confirm ? ask.confirm.text : ""
                wrapMode: Text.Wrap
                horizontalAlignment: Text.AlignHCenter
                font.family: ask.res.fam.ui
                font.pixelSize: ask.lk.shape.panel.title_px * ask.s
                color: ask.lk.palette.pop_ink
            }
            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: ask.lk.shape.clip.foot_gap_px * ask.s
                ClipButton {
                    label: ask.no
                    icon: ask.lk.icons.clip.cancel
                    tone: "ghost"
                    can: true
                    lk: ask.lk
                    res: ask.res
                    s: ask.s
                    uid: ask.res.uid(ask.names.group, ask.names.ids.no)
                    onClicked: ask.act(ask.names.acts.no, 0)
                }
                ClipButton {
                    label: ask.yes
                    icon: ask.lk.icons.clip.delete
                    tone: "danger"
                    can: true
                    lk: ask.lk
                    res: ask.res
                    s: ask.s
                    uid: ask.res.uid(ask.names.group, ask.names.ids.yes)
                    onClicked: ask.act(ask.names.acts.yes, 0)
                }
            }
        }
    }
}

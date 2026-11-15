// The Voice panel's header (mock-up .ph): Back, the page's icon, its title over its line, Delete all where History
// shows it, and Close, with a line under it. The title stays at the left in Arabic, as the mock-up draws it.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: head

    // The panel window, whose view, look and names it takes.
    required property var host

    readonly property var view: host.view
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property real gap: pn.head_gap_px * s
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1

    height: Math.max(lead.height, words.height, ends.height) + (pn.head_pad_px[0] + pn.head_pad_px[2]) * s

    Row {
        id: lead
        anchors.left: parent.left
        anchors.leftMargin: head.pn.head_pad_px[3] * head.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.gap
        VoicePress {
            visible: !!head.view.back
            anchors.verticalCenter: parent.verticalCenter
            host: head.host
            button: head.view.back
            bare: true
            onClicked: head.host.press(head.view.back)
        }
        Icon {
            anchors.verticalCenter: parent.verticalCenter
            icon: head.lk.icons.voice.panel.pages[head.view.page] || ""
            base: head.host.res.icons
            ext: head.host.res.ext
            px: head.pn.button_px[2] * head.s
            tint: head.lk.palette.pop_ink
        }
    }
    Column {
        id: words
        anchors.left: lead.right
        anchors.leftMargin: head.gap
        anchors.right: ends.left
        anchors.rightMargin: head.gap
        anchors.verticalCenter: parent.verticalCenter
        Label {
            width: parent.width
            text: head.view.title
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignLeft
            font.family: head.host.res.fam.ui
            font.pixelSize: head.pn.title_px * head.s * head.scale
            font.weight: head.pn.title_weight
            color: head.lk.palette.pop_ink
        }
        Label {
            width: parent.width
            text: head.view.sub
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignLeft
            font.family: head.host.res.fam.ui
            font.pixelSize: head.pn.hint_px[0] * head.s * head.scale
            color: head.lk.palette.pop_muted
        }
    }
    Row {
        id: ends
        anchors.right: parent.right
        anchors.rightMargin: head.pn.head_pad_px[1] * head.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: head.gap
        VoicePress {
            visible: !!head.view.extra
            anchors.verticalCenter: parent.verticalCenter
            host: head.host
            button: head.view.extra
            onClicked: head.host.press(head.view.extra)
        }
        VoicePress {
            anchors.verticalCenter: parent.verticalCenter
            host: head.host
            button: head.view.close
            bare: true
            onClicked: head.host.press(head.view.close)
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        width: parent.width
        height: head.lk.shape.line.px
        color: head.lk.palette.pop_line
    }
}

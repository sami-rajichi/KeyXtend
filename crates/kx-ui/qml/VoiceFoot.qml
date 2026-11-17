// The Voice panel's footer (mock-up .pf): the page's line at the left, or a question in the panel's ink, then its
// buttons at the right; with no line, every button but the strongest sits left and the strongest right. Buttons keep
// their order in Arabic (ADR-0020), and one that cannot act shows dimmed, never hidden.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: foot

    // The panel window, whose view, look and names it takes.
    required property var host

    readonly property var view: host.view
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var cl: lk.shape.clip
    readonly property bool lined: !!view.line
    readonly property var start: lined ? [] : view.foot.slice(0, -1)
    readonly property var end: lined ? view.foot : view.foot.slice(-1)
    readonly property real side: pn.foot_pad_px[1] * s
    readonly property real gap: cl.foot_gap_px * s
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1

    height: Math.max(cl.button_px[0] * s, line.height) + 2 * pn.foot_pad_px[0] * s

    Rectangle {
        width: parent.width
        height: foot.lk.shape.line.px
        color: foot.lk.palette.pop_line
    }
    Row {
        x: foot.side
        anchors.verticalCenter: parent.verticalCenter
        spacing: foot.gap
        Repeater {
            model: foot.start
            delegate: Press {}
        }
    }
    // A question asks in the buttons' size and weight, as an alert readers hear at once.
    Label {
        id: line
        visible: foot.lined
        anchors.left: parent.left
        anchors.leftMargin: foot.side
        anchors.right: ends.left
        anchors.rightMargin: foot.gap
        anchors.verticalCenter: parent.verticalCenter
        text: foot.view.line || ""
        wrapMode: Text.Wrap
        horizontalAlignment: foot.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: foot.host.res.fam.ui
        font.pixelSize: (foot.host.asking ? foot.cl.button_px[3] : foot.pn.hint_px[0]) * foot.s * foot.scale
        font.weight: foot.host.asking ? foot.cl.button_weight : Font.Normal
        color: foot.host.asking ? foot.lk.palette.pop_ink : foot.lk.palette.pop_muted
        Accessible.ignored: !foot.lined
        Accessible.role: foot.host.asking ? Accessible.AlertMessage : Accessible.StaticText
        Accessible.name: text
    }
    Row {
        id: ends
        anchors.right: parent.right
        anchors.rightMargin: foot.side
        anchors.verticalCenter: parent.verticalCenter
        spacing: foot.gap
        Repeater {
            model: foot.end
            delegate: Press {}
        }
    }

    // One of the footer's buttons; Pin shows Unpin's icon while every ticked text is pinned.
    component Press: VoicePress {
        required property var modelData
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        host: foot.host
        button: modelData
        alt: modelData.id === foot.host.names.pin && foot.view.sheet.unpin ? foot.lk.icons.voice.panel.unpin : ""
        onClicked: foot.host.press(modelData)
    }
}

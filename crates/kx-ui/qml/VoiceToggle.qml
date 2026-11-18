// A switch on its own band (mock-up .swb, .swr): its icon, its label, On or Off, then the switch, on pop_hover as a band
// is, with the hover tint under the pointer. A click anywhere on it sends its kind with the other state, at let-go
// (ADR-0024); readers hear a checkable box. Each part keeps its place in Arabic, whose words read from the right
// (ADR-0020).
import QtQuick

Rectangle {
    id: tg

    // The panel window, whose look and fonts it takes; the hub's switch (`{kind, label, value, on}`); its icon (empty for
    // none); and its gate id.
    required property var host
    required property var toggle
    property string icon: ""
    property string uid: ""

    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var sh: lk.shape.clip.note_px
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1
    readonly property real gap: pn.head_gap_px * s
    // The pointer is over it, which tints it (mock-up .swb:hover).
    readonly property bool hot: act.containsMouse

    // Sends the other state.
    function flip() {
        host.send(toggle.kind, "", toggle.on ? 0 : 1);
    }

    height: Math.max(sw.height, words.height) + 2 * sh[0] * s
    radius: sh[1] * s
    color: lk.palette.pop_hover
    Accessible.role: Accessible.CheckBox
    Accessible.name: toggle.label
    Accessible.description: toggle.value
    Accessible.checkable: true
    Accessible.checked: toggle.on
    Accessible.id: uid
    Accessible.onPressAction: tg.flip()
    Accessible.onToggleAction: tg.flip()

    Act {
        id: act
        anchors.fill: parent
        hoverEnabled: true
        onActed: tg.flip()
    }
    Hover {
        lk: tg.lk
        radius: tg.radius
        visible: tg.hot
    }
    Icon {
        id: glyph
        x: tg.pn.row_px[2] * tg.s
        anchors.verticalCenter: parent.verticalCenter
        visible: !!tg.icon
        icon: tg.icon
        base: tg.host.res.icons
        ext: tg.host.res.ext
        px: tg.lk.shape.clip.button_px[4] * tg.s
        tint: tg.lk.contrast ? tg.lk.sel_ink : tg.lk.palette.ring
    }
    Label {
        id: words
        anchors.left: tg.icon ? glyph.right : parent.left
        anchors.leftMargin: tg.icon ? tg.gap : tg.pn.row_px[2] * tg.s
        anchors.right: value.left
        anchors.rightMargin: tg.gap
        anchors.verticalCenter: parent.verticalCenter
        text: tg.toggle.label
        wrapMode: Text.Wrap
        horizontalAlignment: tg.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: tg.host.res.fam.ui
        font.pixelSize: tg.sh[2] * tg.s * tg.scale
        color: tg.lk.sel_ink
    }
    Label {
        id: value
        anchors.right: sw.left
        anchors.rightMargin: tg.gap
        anchors.verticalCenter: parent.verticalCenter
        text: tg.toggle.value
        font.family: tg.host.res.fam.ui
        font.pixelSize: tg.pn.hint_px[0] * tg.s * tg.scale
        color: tg.lk.sel_ink
    }
    VoiceSwitch {
        id: sw
        anchors.right: parent.right
        anchors.rightMargin: tg.pn.row_px[2] * tg.s
        anchors.verticalCenter: parent.verticalCenter
        on: tg.toggle.on
        lit: true
        lk: tg.lk
        s: tg.s
    }
}

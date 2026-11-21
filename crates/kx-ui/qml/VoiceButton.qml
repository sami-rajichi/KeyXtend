// A dictation bar button (design/voice-bar.html .vb, .vt): a round icon (Cancel, the gear, Stop or Close) or a pill
// with its word (Copy, Undo insert, a note's action). Stop is a dark disc and a note's action takes Enter's colours; one
// that cannot act shows dimmed in its place, and it acts at let-go (ADR-0024).
import QtQuick

Item {
    id: b

    // The button as the hub writes it (`{kind, id, arg, label, on, tone}`, maybe an `icon` in place of its id's), round
    // or a pill, the look, fonts, icons and size, whether the keyboard speaks Arabic, its gate id, and Copy's word and
    // icon once it copied (empty before).
    required property var button
    required property bool round
    required property var lk
    required property var res
    required property real s
    required property bool ar
    required property string uid
    property string done: ""

    signal clicked

    readonly property var sh: lk.shape.clip.button_px
    readonly property var p: lk.palette
    readonly property bool can: button.on
    readonly property bool hot: can && area.containsMouse
    readonly property string word: done !== "" ? done : button.label
    readonly property string icon: done !== "" ? lk.icons.voice.copied : (lk.icons.voice[button.icon || button.id] || "")
    // Each tone's fill and ink, round then pill; kx-theme checks each pair, under the pointer too (drawn/plate.rs,
    // drawn/clip.rs).
    readonly property var looks: ({
            plain: [p.pop_hover, lk.sel_ink],
            primary: round ? [p.pop_ink, p.pop_bg] : [p.enter, p.enter_ink],
            danger: [p.danger, p.danger_ink]
        })
    readonly property color fill: looks[button.tone][0]
    readonly property color ink: looks[button.tone][1]

    width: round ? height : row.width + 2 * sh[2] * s
    height: sh[0] * s
    enabled: can
    opacity: can ? 1 : lk.shape.panel.off_share
    Accessible.role: Accessible.Button
    Accessible.name: word
    Accessible.id: uid
    Accessible.onPressAction: if (b.can) b.clicked()

    // A round dark Stop has no visible edge of its own, as the mock-up's border takes its fill.
    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: b.fill
        border.width: b.lk.shape.line.px
        border.color: b.round && b.button.tone === "primary" ? b.fill : b.p.pop_line
    }
    Hover {
        lk: b.lk
        radius: height / 2
        visible: b.hot
    }
    Row {
        id: row
        anchors.centerIn: parent
        spacing: b.sh[5] * b.s
        Icon {
            anchors.verticalCenter: parent.verticalCenter
            visible: b.icon !== ""
            icon: b.icon
            base: b.res.icons
            ext: b.res.ext
            px: (b.round ? b.lk.shape.caption.icon_px : b.sh[4]) * b.s
            tint: b.ink
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            visible: !b.round
            text: b.word
            font.family: b.res.fam.ui
            font.pixelSize: b.sh[3] * b.s * (b.ar ? b.lk.look.arabic_scale : 1)
            font.weight: b.lk.shape.clip.button_weight
            color: b.ink
        }
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onActed: if (b.can) b.clicked()
    }
}

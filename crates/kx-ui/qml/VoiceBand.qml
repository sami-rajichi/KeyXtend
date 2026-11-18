// A band above a Voice panel list (mock-up .pn): its icon in the ring colour and its line, a name in bold after it,
// on pop_hover, where high contrast's Highlight takes sel_ink (drawn/voicepanel.rs). Both are plain text (U-S1). A note
// band is an alert, so readers hear each new one; Arabic reads from the right while the icon keeps its place. A warning
// takes the warning fill with no sign of its own, a line in parts keeps a service's own words apart (ADR-0047), and a
// band may end with the button it leads to.
pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: band

    // The panel window, whose look and fonts it takes; the band's icon (empty for none), line and bold name (empty for
    // none); and whether it is a note.
    required property var host
    required property string icon
    required property string text
    required property string name
    required property bool alert
    // Its line in parts (`[{text, own}]`, empty for the line whole), whether it warns, and its button (null for none).
    property var parts: []
    property bool warn: false
    property var button: null

    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var sh: lk.shape.clip.note_px
    readonly property color fill: warn ? lk.palette.warn : lk.palette.pop_hover
    readonly property color ink: warn && !lk.contrast ? lk.palette.pop_ink : lk.sel_ink
    readonly property color tint: lk.contrast ? lk.sel_ink : lk.palette.ring
    readonly property var pieces: parts.length > 0 ? parts : [{ text: text, own: false }]

    height: Math.max(glyph.height, line.height, press.visible ? press.height : 0) + 2 * sh[0] * s
    radius: sh[1] * s
    color: fill
    Accessible.role: alert ? Accessible.AlertMessage : Accessible.StaticText
    Accessible.name: name ? text + " " + name : text

    Icon {
        id: glyph
        x: band.pn.row_px[2] * band.s
        anchors.verticalCenter: parent.verticalCenter
        visible: !!band.icon
        icon: band.icon
        base: band.host.res.icons
        ext: band.host.res.ext
        px: band.lk.shape.clip.button_px[4] * band.s
        tint: band.tint
    }
    // The name follows the line a space apart, on the next line when it does not fit, from the right in Arabic; a line in
    // parts adds no space inside its sentence, whose parts carry their own.
    Flow {
        id: line
        anchors.left: band.icon ? glyph.right : parent.left
        anchors.leftMargin: band.icon ? band.pn.head_gap_px * band.s : band.pn.row_px[2] * band.s
        anchors.right: press.visible ? press.left : parent.right
        anchors.rightMargin: band.pn.row_px[2] * band.s
        anchors.verticalCenter: parent.verticalCenter
        layoutDirection: band.host.ar ? Qt.RightToLeft : Qt.LeftToRight
        spacing: band.parts.length > 0 ? 0 : space.advanceWidth(" ")
        Repeater {
            model: band.pieces
            delegate: Part {
                required property var modelData
                text: modelData.text
                wrapMode: Text.Wrap
            }
        }
        Part {
            visible: !!band.name
            text: band.name
            elide: Text.ElideRight
            font.weight: band.pn.title_weight
        }
    }
    VoicePress {
        id: press
        visible: !!band.button
        anchors.right: parent.right
        anchors.rightMargin: band.pn.row_px[2] * band.s
        anchors.verticalCenter: parent.verticalCenter
        host: band.host
        button: band.button
        alt: band.button && band.button.kind === band.host.names.acts.open ? band.lk.icons.voice.panel.pages[band.button.id] || "" : ""
        onClicked: band.host.press(band.button)
    }
    FontMetrics {
        id: space
        font.family: band.host.res.fam.ui
        font.pixelSize: band.sh[2] * band.s * (band.host.ar ? band.lk.look.arabic_scale : 1)
    }

    // A part of the line, as wide as it reads up to the line's width.
    component Part: Label {
        width: Math.min(implicitWidth, line.width)
        horizontalAlignment: band.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: band.host.res.fam.ui
        font.pixelSize: band.sh[2] * band.s * (band.host.ar ? band.lk.look.arabic_scale : 1)
        color: band.ink
    }
}

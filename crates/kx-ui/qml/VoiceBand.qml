// A band above a Voice panel list (mock-up .pn): its icon in the ring colour and its line, a name in bold after it,
// on pop_hover, where high contrast's Highlight takes sel_ink (drawn/voicepanel.rs). Both are plain text (U-S1). A note
// band is an alert, so readers hear each new one; Arabic reads from the right while the icon keeps its place.
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

    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var sh: lk.shape.clip.note_px
    readonly property color fill: lk.palette.pop_hover
    readonly property color ink: lk.sel_ink
    readonly property color tint: lk.contrast ? lk.sel_ink : lk.palette.ring

    height: Math.max(glyph.height, line.height) + 2 * sh[0] * s
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
    // The name follows the line a space apart, on the next line when it does not fit, from the right in Arabic.
    Flow {
        id: line
        anchors.left: glyph.right
        anchors.leftMargin: band.pn.head_gap_px * band.s
        anchors.right: parent.right
        anchors.rightMargin: band.pn.row_px[2] * band.s
        anchors.verticalCenter: parent.verticalCenter
        layoutDirection: band.host.ar ? Qt.RightToLeft : Qt.LeftToRight
        spacing: space.advanceWidth(" ")
        Part {
            id: words
            text: band.text
            wrapMode: Text.Wrap
        }
        Part {
            visible: !!band.name
            text: band.name
            elide: Text.ElideRight
            font.weight: band.pn.title_weight
        }
    }
    FontMetrics {
        id: space
        font: words.font
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

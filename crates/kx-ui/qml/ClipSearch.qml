// The clipboard panel's search line (spec §7): the letters clicked on our keyboard with a caret after them, Arabic
// from the right in the Arabic font, or its hint. It is no text field and never takes focus (ADR-0013); a ring-coloured
// line under it shows where letters go, as WinUI's focused TextBox draws its accent line.
import QtQuick

Rectangle {
    id: line

    // The search (`{text, hint, arabic, rtl}`), whether the keyboard's language reads right to left, the look, fonts,
    // icons and size, and its gate id.
    required property var search
    required property bool rtl
    required property var lk
    required property var res
    required property real s
    property string uid: ""

    readonly property var f: lk.shape.panel.field_px
    readonly property var c: lk.shape.clip
    readonly property var p: lk.palette
    readonly property bool typed: search.text.length > 0
    readonly property real pad: f[2] * s
    // The hint keeps clear of the caret, which waits on its language's start side.
    readonly property real clear: (c.caret_px + c.caret_gap_px) * s

    height: f[0] * s
    radius: f[1] * s
    color: "transparent"
    border.width: lk.shape.line.px
    border.color: p.pop_line
    Accessible.role: Accessible.EditableText
    Accessible.name: search.hint
    Accessible.description: search.text
    Accessible.id: uid

    Icon {
        id: glyph
        x: line.pad
        anchors.verticalCenter: parent.verticalCenter
        icon: line.lk.icons.clip.search
        base: line.res.icons
        ext: line.res.ext
        px: line.c.search_icon_px * line.s
        tint: line.p.pop_muted
    }
    Item {
        id: box
        anchors.left: glyph.right
        anchors.leftMargin: line.pad
        anchors.right: parent.right
        anchors.rightMargin: line.pad
        height: parent.height
        clip: true

        Label {
            visible: !line.typed
            x: line.rtl ? 0 : line.clear
            width: parent.width - line.clear
            anchors.verticalCenter: parent.verticalCenter
            text: line.search.hint
            elide: Text.ElideRight
            font.family: line.res.fam.ui
            font.pixelSize: line.f[3] * line.s
            color: line.p.pop_muted
        }
        // The newest letters stay in sight: the text gives up its start, which is the right side for right to left.
        Label {
            id: said
            visible: line.typed
            width: parent.width - caret.width
            x: line.search.rtl ? caret.width : 0
            anchors.verticalCenter: parent.verticalCenter
            text: line.search.text
            horizontalAlignment: line.search.rtl ? Text.AlignRight : Text.AlignLeft
            elide: line.search.rtl ? Text.ElideRight : Text.ElideLeft
            font.family: line.search.arabic ? line.res.fam.arabic : line.res.fam.latin
            font.pixelSize: line.f[3] * line.s
            color: line.p.pop_ink
        }
        Rectangle {
            id: caret
            // After the text's end, or on the hint's start side while nothing is typed.
            readonly property real used: Math.min(said.contentWidth, said.width)
            x: !line.typed ? (line.rtl ? box.width - width : 0) : line.search.rtl ? said.x + said.width - used - width : used
            anchors.verticalCenter: parent.verticalCenter
            width: line.c.caret_px * line.s
            height: line.f[3] * line.s
            color: line.p.ring
        }
    }
    Rectangle {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        width: parent.width - 2 * line.radius
        height: line.c.underline_px * line.s
        color: line.p.ring
    }
}

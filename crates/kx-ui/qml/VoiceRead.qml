// Compare's set text (mock-up compare()): what to do, the text to read aloud in a box, in its language's direction and
// font, then how many models it runs on which device, or the recording's line while the user reads; it takes a page
// of rows' height, so the panel keeps its size.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: read

    // The panel window, whose view, look and fonts it takes.
    required property var host

    readonly property var sheet: host.view.sheet
    readonly property var lang: host.view.lang
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property real side: pn.head_pad_px[3] * s
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1

    height: (pn.body_pad_px[0] + pn.body_pad_px[2]) * s + host.pageH

    Column {
        x: read.side
        width: read.width - 2 * read.side
        anchors.verticalCenter: parent.verticalCenter
        spacing: read.pn.body_gap_px * read.s
        Line {
            text: read.sheet.intro || ""
        }
        Rectangle {
            width: parent.width
            height: big.height + 2 * read.side
            radius: read.lk.shape.clip.note_px[1] * read.s
            color: read.lk.palette.pop_hover
            border.width: read.lk.shape.line.px
            border.color: read.lk.palette.pop_line
            // On pop_hover, which is Highlight in high contrast, so the text takes sel_ink.
            Label {
                id: big
                x: read.side
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 2 * read.side
                text: read.sheet.text || ""
                wrapMode: Text.Wrap
                horizontalAlignment: read.lang.rtl ? Text.AlignRight : Text.AlignLeft
                font.family: read.lang.arabic ? read.host.res.fam.arabic : read.host.res.fam.latin
                font.pixelSize: read.lk.shape.caption.words_px[read.lang.arabic ? 1 : 0] * read.s
                color: read.lk.sel_ink
                Accessible.ignored: false
                Accessible.role: Accessible.StaticText
                Accessible.name: text
            }
        }
        Line {
            text: read.sheet.recording || read.sheet.line || ""
        }
    }

    // A line about the text, in the panel's muted ink.
    component Line: Label {
        width: parent ? parent.width : 0
        wrapMode: Text.Wrap
        horizontalAlignment: read.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: read.host.res.fam.ui
        font.pixelSize: read.pn.hint_px[0] * read.s * read.scale
        color: read.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}

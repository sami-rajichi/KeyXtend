// A service's card (mock-up cardPage()): each fact's label in a column, then its sentence, a warning fact in warn_ink;
// under a rule, the day the facts were read with the link to the service's page, the day, its icon and the words after it
// in warn_ink and bold once they are old (mock-up .ro.old), and the Untested line. It takes a page of rows' height, its
// facts in two parts the pager turns, so the panel keeps its size (ADR-0008, ADR-0047).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: card

    // The panel window, whose view, look and fonts it takes.
    required property var host

    readonly property var sheet: host.view.sheet
    readonly property bool shows: sheet.is === host.names.sheets.card
    readonly property var read: shows ? sheet.read : null
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property real side: pn.row_px[2] * s
    readonly property real gap: pn.row_gap_px * s
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1
    readonly property real hintPx: pn.hint_px[0] * s * scale
    // The pager's room beside the facts while the card has more than one page.
    readonly property real pager: host.view.pages.count > 1 ? (pn.body_gap_px + pn.page_px[0]) * s : 0
    readonly property alias body: col

    height: (pn.body_pad_px[0] + pn.body_pad_px[2]) * s + host.pageH

    Column {
        id: col
        x: card.pn.body_pad_px[1] * card.s + card.side
        width: card.width - 2 * x - card.pager
        anchors.verticalCenter: parent.verticalCenter
        spacing: card.pn.body_gap_px * card.s
        Column {
            width: parent.width
            spacing: card.lk.shape.chip.gap_px * card.s
            Repeater {
                model: card.shows ? card.sheet.facts : []
                delegate: Item {
                    id: fact
                    required property var modelData
                    width: col.width
                    height: Math.max(key.height, words.height)
                    Accessible.role: Accessible.StaticText
                    Accessible.name: modelData.said
                    Label {
                        id: key
                        width: card.lk.shape.voice.fact_px * card.s
                        text: fact.modelData.label
                        wrapMode: Text.Wrap
                        horizontalAlignment: card.host.ar ? Text.AlignRight : Text.AlignLeft
                        font.family: card.host.res.fam.ui
                        font.pixelSize: card.hintPx
                        color: card.p.pop_muted
                    }
                    Parts {
                        id: words
                        anchors.left: key.right
                        anchors.leftMargin: card.gap
                        anchors.right: parent.right
                        parts: fact.modelData.value.parts
                        warn: fact.modelData.warn
                    }
                }
            }
        }
        Rectangle {
            width: parent.width
            height: card.lk.shape.line.px
            color: card.p.pop_line
        }
        Note {
            icon: card.lk.icons.voice.panel.cloud.read
            text: card.read ? card.read.text : ""
            more: card.read && card.read.old ? card.read.old : ""
            said: card.read ? card.read.said : ""
            button: card.read ? card.read.link : null
        }
        Note {
            visible: card.shows && !!card.sheet.untested
            icon: card.lk.icons.voice.panel.cloud.untested
            text: card.shows ? card.sheet.untested || "" : ""
        }
    }

    VoicePager {
        anchors.right: parent.right
        anchors.rightMargin: card.pn.body_pad_px[1] * card.s
        anchors.verticalCenter: parent.verticalCenter
        host: card.host
    }

    // A sentence in parts, a service's own words apart from the rest (ADR-0022), from the right in Arabic.
    component Parts: Flow {
        id: flow
        property var parts: []
        property bool warn: false
        layoutDirection: card.host.ar ? Qt.RightToLeft : Qt.LeftToRight
        Repeater {
            model: flow.parts
            delegate: Label {
                required property var modelData
                width: Math.min(implicitWidth, flow.width)
                text: modelData.text
                wrapMode: Text.Wrap
                horizontalAlignment: card.host.ar ? Text.AlignRight : Text.AlignLeft
                font.family: card.host.res.fam.ui
                font.pixelSize: card.pn.row_text_px[card.host.ar ? 1 : 0] * card.s
                font.weight: flow.warn ? card.pn.count_weight : Font.Normal
                color: flow.warn ? card.p.warn_ink : card.p.pop_ink
            }
        }
    }
    // A muted line under the facts with its icon and a button at its end; with words after it, all of it warns.
    component Note: Item {
        id: note
        property string icon: ""
        property string text: ""
        property string more: ""
        // The line readers hear, which the hub joins in the language; the line alone when empty.
        property string said: ""
        property var button: null
        width: col.width
        height: Math.max(glyph.height, line.height, press.visible ? press.height : 0)
        Accessible.role: Accessible.StaticText
        Accessible.name: said || text
        Icon {
            id: glyph
            anchors.verticalCenter: parent.verticalCenter
            icon: note.icon
            base: card.host.res.icons
            ext: card.host.res.ext
            px: card.lk.shape.clip.button_px[4] * card.s
            tint: note.more ? card.p.warn_ink : card.p.pop_muted
        }
        Flow {
            id: line
            anchors.left: glyph.right
            anchors.leftMargin: card.pn.head_gap_px * card.s
            anchors.right: press.visible ? press.left : parent.right
            anchors.rightMargin: press.visible ? card.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            layoutDirection: card.host.ar ? Qt.RightToLeft : Qt.LeftToRight
            spacing: space.advanceWidth(" ")
            Muted {
                text: note.text
                font.weight: note.more ? card.pn.count_weight : Font.Normal
                color: note.more ? card.p.warn_ink : card.p.pop_muted
            }
            Muted {
                visible: !!note.more
                text: note.more
                font.weight: card.pn.count_weight
                color: card.p.warn_ink
            }
        }
        VoicePress {
            id: press
            visible: !!note.button
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            host: card.host
            button: note.button
            onClicked: card.host.press(note.button)
        }
    }
    component Muted: Label {
        width: Math.min(implicitWidth, parent ? parent.width : 0)
        wrapMode: Text.Wrap
        horizontalAlignment: card.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: card.host.res.fam.ui
        font.pixelSize: card.hintPx
        color: card.p.pop_muted
    }
    FontMetrics {
        id: space
        font.family: card.host.res.fam.ui
        font.pixelSize: card.hintPx
    }
}

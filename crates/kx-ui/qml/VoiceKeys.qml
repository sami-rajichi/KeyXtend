// A service's key page (mock-up svcPage()): the key line as dots with Paste, Save and Test and the test's answer, Key
// works as a lit mark and a refused one as a warning band, the address and Other model lines with their Save and why one
// was refused, the keep-nothing switch, and where keys are made. The lines take our keyboard's letters while aimed but
// only draw: the key never shows as text, and a click never focuses them (ADR-0013, ADR-0049). It takes a page of rows'
// height; a typed address puts its lines on a second page the pager turns to (ADR-0008).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: keys

    // The panel window, whose view, look, names and fonts it takes.
    required property var host

    readonly property var sheet: host.view.sheet
    readonly property bool shows: sheet.is === host.names.sheets.service
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property var names: host.names
    readonly property real gap: pn.row_gap_px * s
    readonly property real hintPx: pn.hint_px[0] * s * (host.ar ? lk.look.arabic_scale : 1)
    readonly property var lines: shows ? [sheet.key, sheet.address, sheet.model].filter(l => !!l) : []
    // The pager's room beside the lines while the page has more than one part.
    readonly property real pager: host.view.pages.count > 1 ? (pn.body_gap_px + pn.page_px[0]) * s : 0
    // The key works, which shows as a lit mark; or its answer warns, which shows as a warning band (mock-up .res.bad).
    readonly property bool works: shows && !!sheet.check && sheet.check.works
    readonly property bool refused: shows && !!sheet.check && sheet.check.warns
    readonly property alias body: col

    height: (pn.body_pad_px[0] + pn.body_pad_px[2]) * s + host.pageH

    Column {
        id: col
        x: keys.pn.body_pad_px[1] * keys.s + keys.pn.row_px[2] * keys.s
        width: keys.width - 2 * x - keys.pager
        anchors.verticalCenter: parent.verticalCenter
        spacing: keys.pn.body_gap_px * keys.s
        Repeater {
            model: keys.lines
            delegate: Column {
                id: entry
                required property var modelData
                required property int index
                width: col.width
                spacing: keys.pn.body_gap_px * keys.s
                Item {
                    width: parent.width
                    height: field.height
                    PanelLine {
                        id: field
                        // The key shows the typed key's dots while one is typed or the line takes the letters, else the
                        // sealed key's; an address or model id reads left to right in every language (ADR-0049).
                        readonly property var d: entry.modelData
                        width: parent.width - (ends.width > 0 ? ends.width + keys.gap : 0)
                        said: d.mask && count > 0 ? null : ({ text: d.text, rtl: false, arabic: false })
                        hint: d.hint
                        caption: d.caption
                        at: d.at
                        takes: d.typing && keys.host.aimed
                        aims: !keys.host.asking
                        search: true
                        secret: !!d.mask
                        mask: d.mask || ""
                        count: !d.mask ? 0 : d.typed > 0 || takes ? d.typed : d.dots
                        rtl: !!d.mask && keys.host.ar
                        lk: keys.lk
                        res: keys.host.res
                        s: keys.s
                        uid: keys.host.res.uid(keys.names.group, keys.names.ids.line + d.id)
                        // A press, a reader's too, picks the line with its caret at the end and the key sound; a click
                        // on the text then places the caret silently.
                        onClicked: keys.host.send(keys.names.acts.pick, d.id, field.drawn.length)
                        onPlaced: at => keys.host.send(keys.names.acts.caret, d.id, at)
                    }
                    Row {
                        id: ends
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: keys.gap
                        Repeater {
                            model: entry.modelData.buttons
                            delegate: VoicePress {
                                required property var modelData
                                anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                                host: keys.host
                                button: modelData
                                onClicked: keys.host.press(modelData)
                            }
                        }
                    }
                }
                Said {
                    visible: entry.index === 0 && !!keys.sheet.check && !keys.works && !keys.refused
                    icon: keys.sheet.check ? keys.lk.icons.voice.panel.cloud[keys.sheet.check.id] || "" : ""
                    text: keys.sheet.check ? keys.sheet.check.text : ""
                }
                VoiceMark {
                    visible: entry.index === 0 && keys.works
                    host: keys.host
                    mark: keys.works ? keys.sheet.check : ({ id: "", text: "" })
                    glyph: keys.lk.icons.voice.panel.cloud.ok
                    badge: true
                    lit: false
                }
                Refused {
                    visible: entry.index === 0 && keys.refused
                    text: keys.refused ? keys.sheet.check.text : ""
                }
                Refused {
                    visible: !!entry.modelData.fault
                    text: entry.modelData.fault || ""
                }
            }
        }
        VoiceToggle {
            visible: keys.shows && !!keys.sheet.keep
            width: col.width
            host: keys.host
            toggle: keys.shows && keys.sheet.keep ? keys.sheet.keep : ({ kind: "", label: "", value: "", on: false })
            uid: keys.host.res.uid(keys.names.group, keys.names.acts.keep)
        }
        VoiceBand {
            visible: keys.shows && !!keys.sheet.link
            width: col.width
            host: keys.host
            icon: keys.lk.icons.voice.panel.cloud.site
            text: keys.shows && keys.sheet.link ? keys.sheet.link.text : ""
            name: ""
            alert: false
            button: keys.shows && keys.sheet.link ? keys.sheet.link.button : null
        }
    }

    VoicePager {
        anchors.right: parent.right
        anchors.rightMargin: keys.pn.body_pad_px[1] * keys.s
        anchors.verticalCenter: parent.verticalCenter
        host: keys.host
    }

    // A line under a typed line, such as Testing, in the muted ink with its icon.
    component Said: Row {
        id: said
        property string icon: ""
        property string text: ""
        width: parent ? parent.width : 0
        spacing: keys.pn.head_gap_px * keys.s
        layoutDirection: keys.host.ar ? Qt.RightToLeft : Qt.LeftToRight
        Accessible.role: Accessible.StaticText
        Accessible.name: text
        Icon {
            id: sign
            visible: !!said.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: said.icon
            base: keys.host.res.icons
            ext: keys.host.res.ext
            px: keys.lk.shape.chip.icon_px * keys.s
            tint: keys.p.pop_muted
        }
        Label {
            width: Math.min(implicitWidth, said.width - (sign.visible ? sign.width + said.spacing : 0))
            text: said.text
            wrapMode: Text.Wrap
            horizontalAlignment: keys.host.ar ? Text.AlignRight : Text.AlignLeft
            font.family: keys.host.res.fam.ui
            font.pixelSize: keys.hintPx
            color: keys.p.pop_muted
        }
    }
    // A refused answer or line as a warning band, which readers hear at once.
    component Refused: VoiceBand {
        width: parent ? parent.width : 0
        host: keys.host
        icon: ""
        name: ""
        alert: true
        warn: true
    }
}

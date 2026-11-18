// A service's key page (mock-up svcPage()): the key line as dots with Paste and Test and the test's answer, the address
// and Other model lines with why one was refused, the keep-nothing switch, and where keys are made. The lines only draw:
// the key never shows as text, and a click never focuses them (ADR-0013, ADR-0040). It takes a page of rows' height; a
// typed address puts its lines on a second page the pager turns to (ADR-0008).
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
                        width: parent.width - (ends.width > 0 ? ends.width + keys.gap : 0)
                        said: entry.modelData.mask && entry.modelData.dots > 0 ? null : ({ text: entry.modelData.text, rtl: false, arabic: false })
                        hint: entry.modelData.hint
                        caption: entry.modelData.caption
                        aims: false
                        secret: !!entry.modelData.mask
                        mask: entry.modelData.mask || ""
                        count: entry.modelData.dots
                        rtl: keys.host.ar
                        lk: keys.lk
                        res: keys.host.res
                        s: keys.s
                        uid: keys.host.res.uid(keys.names.group, keys.names.ids.line + entry.modelData.id)
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
                    visible: entry.index === 0 && !!keys.sheet.check
                    icon: keys.sheet.check ? keys.lk.icons.voice.panel.cloud[keys.sheet.check.id] || "" : ""
                    text: keys.sheet.check ? keys.sheet.check.text : ""
                    warn: !!keys.sheet.check && keys.names.refused.indexOf(keys.sheet.check.id) >= 0
                }
                Said {
                    visible: !!entry.modelData.fault
                    text: entry.modelData.fault || ""
                    warn: true
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

    // A line under a typed line: the test's answer or why it was refused, in warn_ink when it warns, read at once.
    component Said: Row {
        id: said
        property string icon: ""
        property string text: ""
        property bool warn: false
        width: parent ? parent.width : 0
        spacing: keys.pn.head_gap_px * keys.s
        layoutDirection: keys.host.ar ? Qt.RightToLeft : Qt.LeftToRight
        Accessible.role: warn ? Accessible.AlertMessage : Accessible.StaticText
        Accessible.name: text
        Icon {
            id: sign
            visible: !!said.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: said.icon
            base: keys.host.res.icons
            ext: keys.host.res.ext
            px: keys.lk.shape.chip.icon_px * keys.s
            tint: said.warn ? keys.p.warn_ink : keys.p.pop_muted
        }
        Label {
            width: Math.min(implicitWidth, said.width - (sign.visible ? sign.width + said.spacing : 0))
            text: said.text
            wrapMode: Text.Wrap
            horizontalAlignment: keys.host.ar ? Text.AlignRight : Text.AlignLeft
            font.family: keys.host.res.fam.ui
            font.pixelSize: keys.hintPx
            font.weight: said.warn ? keys.pn.count_weight : Font.Normal
            color: said.warn ? keys.p.warn_ink : keys.p.pop_muted
        }
    }
}

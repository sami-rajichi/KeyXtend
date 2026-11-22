// A mark on a Voice panel row (mock-up .mk): a pill with its icon and word. In use and Key works take the badge colours,
// a missing, failed, other, off, unavailable or waiting one is outlined, a warning takes the warning colours with no sign
// of its own, the rest sit on the chip colour; kx-theme checks each pair on every row (drawn/voicepanel.rs). Readers
// hear its word.
import QtQuick

Rectangle {
    id: m

    // The panel window, whose look, names and fonts it takes; the hub's mark (`{id, text}`); whether its row is under
    // the pointer or ticked.
    required property var host
    required property var mark
    required property bool lit
    // Its own icon in place of its id's (empty for its id's), and the badge colours whatever its id: Key works takes both.
    property string glyph: ""
    property bool badge: false
    // The widest it may be; past that its word is shortened, while readers still hear it whole.
    property real most: Infinity
    // Its width with its whole word.
    readonly property real natural: word.implicitWidth + (pic.visible ? pic.width + row.spacing : 0) + 2 * sh.pad_px * host.s

    readonly property string markId: mark.id
    readonly property var names: host.names
    readonly property bool shown: badge || names.lit.indexOf(markId) >= 0
    readonly property bool outlined: names.outlined.indexOf(markId) >= 0
    readonly property bool warned: names.warned.indexOf(markId) >= 0
    readonly property string tone: shown ? "lit" : outlined ? "out" : warned ? "warn" : "plain"
    readonly property string icon: glyph || host.lk.icons.voice.panel.marks[markId] || ""
    readonly property var sh: host.lk.shape.chip
    readonly property var p: host.lk.palette
    readonly property bool hc: host.lk.contrast
    // Each tone's fill, word and icon. An outlined mark sits on its row, so on a lit row its parts take sel_ink; high
    // contrast draws a chip's in ButtonText, and a warning's on Highlight in sel_ink.
    readonly property var looks: ({
            lit: [p.badge_bg, p.badge_ink, p.badge_ink],
            out: ["transparent", lit ? host.lk.sel_ink : p.pop_ink, lit ? host.lk.sel_ink : p.pop_muted],
            plain: [p.chip_bg, hc ? p.legend : p.pop_ink, hc ? p.legend : p.ring],
            warn: [p.warn, hc ? host.lk.sel_ink : p.pop_ink, hc ? host.lk.sel_ink : p.pop_ink]
        })
    readonly property color fill: looks[tone][0]
    readonly property color ink: looks[tone][1]
    readonly property color tint: looks[tone][2]

    width: row.width + 2 * sh.pad_px * host.s
    height: sh.height_px * host.s
    radius: height / 2
    color: fill
    border.width: host.lk.shape.line.px
    border.color: tone === "lit" ? p.badge_bg : tone === "warn" ? p.warn : p.pop_line
    Accessible.role: Accessible.StaticText
    Accessible.name: mark.text

    Row {
        id: row
        anchors.centerIn: parent
        spacing: m.sh.gap_px * m.host.s
        Icon {
            id: pic
            visible: !!m.icon
            anchors.verticalCenter: parent.verticalCenter
            icon: m.icon
            base: m.host.res.icons
            ext: m.host.res.ext
            px: m.sh.icon_px * m.host.s
            tint: m.tint
        }
        Label {
            id: word
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(implicitWidth, Math.max(0, implicitWidth - (m.natural - m.most)))
            elide: Text.ElideRight
            text: m.mark.text
            font.family: m.host.ar ? m.host.res.fam.arabic : m.host.res.fam.latin
            font.pixelSize: m.sh.font_px[m.host.ar ? 1 : 0] * m.host.s
            font.weight: m.sh.weight
            color: m.ink
        }
    }
}

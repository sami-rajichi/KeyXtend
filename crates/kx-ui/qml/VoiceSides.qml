// The Models page's Local | Cloud chips (mock-up sides()): their caption, then a chip for each side, the shown one
// filled and checked as the panels' filters are (PanelChip); a click shows that side, which pages alone (edge 29). Each
// part keeps its place in Arabic, as the filters and Options' paired chips do (ADR-0020).
pragma ComponentBehavior: Bound
import QtQuick

Row {
    id: row

    // The panel window, whose look, names and fonts it takes; the hub's chips (`{name, shown, chips}`).
    required property var host
    required property var sides

    readonly property var lk: host.lk
    readonly property real s: host.s

    spacing: lk.shape.panel.filter_gap_px * s
    Accessible.role: Accessible.Grouping
    Accessible.name: sides.name

    Label {
        anchors.verticalCenter: parent.verticalCenter
        text: row.sides.name
        font.family: row.host.res.fam.ui
        font.pixelSize: row.lk.shape.panel.hint_px[0] * row.s * (row.host.ar ? row.lk.look.arabic_scale : 1)
        color: row.lk.palette.pop_muted
    }
    Repeater {
        model: row.sides.chips
        delegate: PanelChip {
            required property var modelData
            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            label: modelData.label
            shown: modelData.id === row.sides.shown
            check: row.lk.icons.voice.panel.ticked
            rtl: row.host.ar
            move: row.lk.motion.clip.highlight
            uid: row.host.res.uid(row.host.names.group, row.host.names.ids.side + modelData.id)
            lk: row.lk
            res: row.host.res
            s: row.s
            onPicked: row.host.press(modelData)
        }
    }
}

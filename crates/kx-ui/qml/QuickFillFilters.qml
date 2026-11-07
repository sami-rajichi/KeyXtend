// Quick-fill's filters (design "The panel"; mock-up .filters): All, People, Logins, Wi-Fi, Cards, Notes and By label as
// the panels' filter chips (PanelChip). The chips stay put as views change.
pragma ComponentBehavior: Bound
import QtQuick

Row {
    id: bar

    // The chips (`[{id, label, on}]`), the click names, whether the keyboard's language reads right to left, the look,
    // fonts, icons and size.
    required property var filters
    required property var names
    required property bool rtl
    required property var lk
    required property var res
    required property real s

    // Chip `id` was clicked.
    signal picked(string id)

    spacing: lk.shape.panel.filter_gap_px * s

    Repeater {
        model: bar.filters.length
        delegate: PanelChip {
            required property int index
            readonly property var f: bar.filters[index] || { id: "", label: "", on: false }

            anchors.verticalCenter: parent.verticalCenter
            label: f.label
            shown: f.on
            check: bar.lk.icons.fill.shown
            rtl: bar.rtl
            move: bar.lk.motion.fill.highlight
            uid: bar.res.uid(bar.names.group, bar.names.ids.filter + f.id)
            lk: bar.lk
            res: bar.res
            s: bar.s
            onPicked: bar.picked(f.id)
        }
    }
}

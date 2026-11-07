// The clipboard panel's filters (spec §7; mock-up .filters): All, Text, Images, Files and Media as the panels' filter
// chips (PanelChip). The chips stay put as views change.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: bar

    // The chips (`[{id, label, on}]`), the click names, whether the keyboard's language reads right to left, the look,
    // fonts, icons and size.
    required property var filters
    required property var names
    required property bool rtl
    required property var lk
    required property var res
    required property real s

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    readonly property var c: lk.shape.clip
    readonly property var pn: lk.shape.panel

    height: (c.filters_pad_px[0] + pn.filter_px[0] + c.filters_pad_px[2]) * s

    Row {
        x: bar.c.filters_pad_px[1] * bar.s
        y: bar.c.filters_pad_px[0] * bar.s
        spacing: bar.pn.filter_gap_px * bar.s
        Repeater {
            model: bar.filters.length
            delegate: PanelChip {
                required property int index
                readonly property var f: bar.filters[index] || { id: "", label: "", on: false }

                label: f.label
                shown: f.on
                check: bar.lk.icons.clip.shown
                rtl: bar.rtl
                move: bar.lk.motion.clip.highlight
                uid: bar.res.uid(bar.names.group, bar.names.ids.filter + f.id)
                lk: bar.lk
                res: bar.res
                s: bar.s
                onPicked: bar.act(bar.names.acts.filter, index)
            }
        }
    }
}

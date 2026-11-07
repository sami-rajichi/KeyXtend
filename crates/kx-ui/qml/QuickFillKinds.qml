// New's kind chips (design "New and Edit"): Person, Login, Wi-Fi, Card, Note and Other, in the theme's rows in the rows'
// place, each a big target with its kind's icon over its name. A click picks the kind, which brings its fields.
pragma ComponentBehavior: Bound
import QtQuick

Grid {
    id: grid

    // The kinds (`[{id, label}]`), the click names, whether the keyboard's language reads right to left, the look, fonts,
    // icons and size.
    required property var kinds
    required property var names
    required property bool rtl
    required property var lk
    required property var res
    required property real s

    // Kind `id` was picked.
    signal picked(string id)

    readonly property var sh: lk.shape.fill.kind_px
    readonly property real gap: sh[0] * s
    // The rows the theme gives them (shape.toml `[fill] kind_rows`), and the columns they need.
    readonly property int lines: lk.shape.fill.kind_rows
    readonly property int cols: Math.ceil(kinds.length / lines)

    columns: Math.max(1, cols)
    spacing: gap

    Repeater {
        model: grid.kinds.length
        delegate: Rectangle {
            id: chip
            required property int index
            readonly property var k: grid.kinds[index] || { id: "", label: "" }
            readonly property string label: k.label

            width: (grid.width - (grid.columns - 1) * grid.gap) / grid.columns
            height: (grid.height - (grid.lines - 1) * grid.gap) / grid.lines
            radius: grid.sh[1] * grid.s
            color: grid.lk.palette.pop_hover
            border.width: grid.lk.shape.line.px
            border.color: grid.lk.palette.pop_line
            Accessible.role: Accessible.Button
            Accessible.name: label
            Accessible.id: grid.res.uid(grid.names.group, grid.names.ids.kind + k.id)
            Accessible.onPressAction: grid.picked(chip.k.id)

            Hover {
                lk: grid.lk
                radius: chip.radius
                visible: area.containsMouse
            }
            Column {
                anchors.centerIn: parent
                width: parent.width
                spacing: grid.sh[3] * grid.s
                Icon {
                    anchors.horizontalCenter: parent.horizontalCenter
                    icon: grid.names.kindIcon(grid.lk.icons.fill, chip.k.id)
                    base: grid.res.icons
                    ext: grid.res.ext
                    px: grid.sh[2] * grid.s
                    tint: grid.lk.sel_ink
                }
                Label {
                    width: parent.width
                    text: chip.label
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                    font.family: grid.res.fam.ui
                    font.pixelSize: grid.lk.shape.panel.row_text_px[grid.rtl ? 1 : 0] * grid.s
                    font.weight: grid.lk.shape.chip.weight
                    color: grid.lk.sel_ink
                }
            }
            Act {
                id: area
                anchors.fill: parent
                hoverEnabled: true
                onActed: grid.picked(chip.k.id)
            }
        }
    }
}

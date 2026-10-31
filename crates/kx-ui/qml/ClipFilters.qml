// The clipboard panel's filters (spec §7; mock-up .filters and .fl): All, Text, Images, Files and Media as pill chips,
// the shown one filled and checked, so it reads by more than its colour. Each chip stays put as views change, so its
// fill fades to the new state; each one is a big target for one finger (WCAG 2.5.8).
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
    readonly property var p: lk.palette

    height: (c.filters_pad_px[0] + c.filter_px[0] + c.filters_pad_px[2]) * s

    Row {
        x: bar.c.filters_pad_px[1] * bar.s
        y: bar.c.filters_pad_px[0] * bar.s
        spacing: bar.c.filter_gap_px * bar.s
        Repeater {
            model: bar.filters.length
            delegate: Rectangle {
                id: chip
                required property int index
                readonly property var f: bar.filters[index] || { id: "", label: "", on: false }
                readonly property bool hot: area.containsMouse && !f.on
                // Shown, it takes pop_sel with sel_ink; else its text sits on the panel, or on the hover tint.
                readonly property color ink: f.on ? bar.lk.sel_ink : bar.p.pop_ink

                width: words.width + 2 * bar.c.filter_px[1] * bar.s
                height: bar.c.filter_px[0] * bar.s
                radius: height / 2
                color: f.on ? bar.p.pop_sel : "transparent"
                border.width: f.on ? 0 : bar.lk.shape.line.px
                border.color: bar.p.pop_line
                Accessible.role: Accessible.RadioButton
                Accessible.name: f.label
                Accessible.checkable: true
                Accessible.checked: f.on
                Accessible.id: bar.res.uid(bar.names.group, bar.names.ids.filter + f.id)
                Accessible.onPressAction: bar.act(bar.names.acts.filter, chip.index)
                Behavior on color {
                    ColourTween {
                        move: bar.lk.motion.clip.highlight
                    }
                }

                Hover {
                    lk: bar.lk
                    radius: chip.radius
                    visible: chip.hot
                }
                Row {
                    id: words
                    anchors.centerIn: parent
                    spacing: bar.c.filter_px[3] * bar.s
                    Icon {
                        visible: chip.f.on
                        anchors.verticalCenter: parent.verticalCenter
                        icon: bar.lk.icons.clip.shown
                        base: bar.res.icons
                        ext: bar.res.ext
                        px: bar.c.filter_px[2] * bar.s
                        tint: chip.ink
                    }
                    Label {
                        anchors.verticalCenter: parent.verticalCenter
                        text: chip.f.label
                        font.family: bar.res.fam.ui
                        font.pixelSize: bar.lk.shape.chip.font_px[bar.rtl ? 1 : 0] * bar.s
                        font.weight: bar.lk.shape.chip.weight
                        color: chip.ink
                    }
                }
                Act {
                    id: area
                    anchors.fill: parent
                    hoverEnabled: true
                    onActed: bar.act(bar.names.acts.filter, chip.index)
                }
            }
        }
    }
}

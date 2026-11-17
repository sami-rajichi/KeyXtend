// An option (mock-up options()): its name, then its switch with On or Off, which a click on the row flips, a stepper
// with Less and More or the last and next choice, or two chips, the first picked by minus and the second by plus; a
// hint row shows its line under a bare icon, which the chips above read as their description. Never a slider
// (ADR-0008).
pragma ComponentBehavior: Bound
import QtQuick

VoiceRow {
    id: row

    readonly property var kinds: host.names.options
    readonly property bool flips: !!entry && entry.kind === kinds.switch
    readonly property bool pair: !!entry && entry.kind === kinds.pair
    readonly property bool hint: !!entry && entry.kind === kinds.hint
    // A pair's hint row below, which its chips read as their description (mock-up aria-describedby).
    readonly property var below: pair ? list.entry(index + 1) : null
    readonly property string about: (!!below && below.kind === kinds.hint) ? below.label : ""

    name: entry ? entry.label : ""
    tile: hint ? lk.icons.voice.panel.buttons.insert : ""
    quiet: hint
    takes: flips
    Accessible.role: flips ? Accessible.CheckBox : Accessible.ListItem
    Accessible.description: entry ? entry.value : ""
    Accessible.checkable: flips
    Accessible.checked: flips && entry.on
    onClicked: host.send(host.names.acts.flip, "", place)

    Label {
        visible: row.flips
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        text: row.flips ? row.entry.value : ""
        font.family: row.res.fam.ui
        font.pixelSize: row.pn.hint_px[0] * row.s * row.scale
        color: row.lit ? row.lk.sel_ink : row.p.pop_muted
    }
    VoiceSwitch {
        visible: row.flips
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        on: row.flips && row.entry.on
        lit: row.lit
        lk: row.lk
        s: row.s
    }
    VoiceStepper {
        visible: row.filled && !!row.entry.steps
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        owner: row
    }
    Row {
        visible: row.pair
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        spacing: row.pn.filter_gap_px * row.s
        Repeater {
            model: row.pair ? row.entry.chips.length : 0
            delegate: PanelChip {
                required property int index
                readonly property var chip: (row.pair && row.entry.chips[index]) || ({ label: "", on: false })
                // Minus picks the first chip and plus the second, as a stepper would.
                readonly property string way: index > 0 ? row.host.names.acts.plus : row.host.names.acts.minus

                label: chip.label
                shown: chip.on
                check: row.lk.icons.voice.panel.ticked
                rtl: row.host.ar
                move: row.lk.motion.clip.highlight
                uid: row.res.uid(row.host.names.group, row.host.names.ids[way] + row.place)
                Accessible.description: row.about
                lk: row.lk
                res: row.res
                s: row.s
                onPicked: row.host.send(row.host.names.acts.step, way, row.place)
            }
        }
    }
}

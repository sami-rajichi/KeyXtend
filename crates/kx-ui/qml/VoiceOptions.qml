// An option (mock-up options()): its name, then its switch with On or Off, which a click on the row flips, or a
// stepper with Less and More, or the last and next choice; never a slider (ADR-0008).
import QtQuick

VoiceRow {
    id: row

    readonly property bool flips: !!entry && entry.kind === host.names.options.switch

    name: entry ? entry.label : ""
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
        visible: row.filled && !row.flips
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        owner: row
    }
}

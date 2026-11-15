// An option's stepper (mock-up .ctl): Less, the value and More, or the last and next choice with chevrons. A step past
// either end shows dimmed, and the whole stepper while its option waits on another, never hidden.
pragma ComponentBehavior: Bound
import QtQuick

Row {
    id: st

    // The option's row, whose entry, place, look and panel it takes.
    required property var owner

    readonly property var host: owner.host
    readonly property var option: owner.entry
    readonly property bool live: !!option && option.live
    readonly property bool choice: !!option && option.kind === host.names.options.choice
    readonly property var steps: option && option.steps ? option.steps : ({ low: true, high: true, minus: "", plus: "" })
    readonly property var icons: owner.lk.icons.voice.panel

    spacing: owner.pn.pager_gap_px * owner.s

    Step {
        way: st.host.names.acts.minus
        icon: st.choice ? st.icons.before : st.icons.minus
        can: st.live && !st.steps.low
        Accessible.name: st.steps.minus
    }
    Label {
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(implicitWidth, st.owner.lk.shape.voice.value_px * st.owner.s)
        text: st.option ? st.option.value : ""
        horizontalAlignment: Text.AlignHCenter
        opacity: st.live ? 1 : st.owner.pn.off_share
        font.family: st.owner.res.fam.ui
        font.pixelSize: st.owner.lk.shape.clip.button_px[3] * st.owner.s * st.owner.scale
        font.weight: st.owner.lk.shape.clip.button_weight
        color: st.owner.lit ? st.owner.lk.sel_ink : st.owner.p.pop_ink
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    Step {
        way: st.host.names.acts.plus
        icon: st.choice ? st.icons.after : st.icons.plus
        can: st.live && !st.steps.high
        Accessible.name: st.steps.plus
    }

    // Less or More, by `way`: a square button with its icon alone, which steps the option at the row's place.
    component Step: ClipButton {
        required property string way
        anchors.verticalCenter: parent.verticalCenter
        label: ""
        tone: "ghost"
        lk: st.owner.lk
        res: st.owner.res
        s: st.owner.s
        uid: res.uid(st.host.names.group, st.host.names.ids[way] + st.owner.place)
        onClicked: st.host.send(st.host.names.acts.step, way, st.owner.place)
    }
}

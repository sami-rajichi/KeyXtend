// A clipboard row's tick box (mock-up .ck): an outline in the row's mark colour, or filled in the badge colours with
// the row's place in the tick order, which pops in as the row is ticked.
import QtQuick

Rectangle {
    id: tick

    // The tick number (null when unticked), the outline's colour, the look, fonts and size.
    required property var number
    required property color mark
    required property var lk
    required property var res
    required property real s

    readonly property var sh: lk.shape.clip.tick_px
    readonly property bool on: number !== null && number !== undefined

    // Pops the number in, from small to full size (motion.toml [clip] tick, a spring).
    function pop() {
        grow.restart();
    }

    width: sh[0] * s
    height: width
    radius: sh[1] * s
    color: on ? lk.palette.badge_bg : "transparent"
    // High contrast keeps the outline on a ticked box, as its fill is the ticked row's Highlight.
    border.width: on && !lk.contrast ? 0 : sh[2] * s
    border.color: mark

    // Fills the box, so the number sits in the same place on every row, Arabic ones too.
    Label {
        anchors.fill: parent
        visible: tick.on
        text: tick.on ? String(tick.number) : ""
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        font.family: tick.res.fam.latin
        font.pixelSize: tick.sh[3] * tick.s
        font.weight: tick.lk.shape.clip.tick_weight
        color: tick.lk.palette.badge_ink
    }
    Tween {
        id: grow
        target: tick
        property: "scale"
        from: tick.lk.motion.amount.tick_scale
        to: 1
        move: tick.lk.motion.clip.tick
    }
}

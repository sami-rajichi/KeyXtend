// An option's switch (mock-up .sw): a pill whose knob slides to the end, in the badge colours when on, outlined in
// pop_muted when off, or in sel_ink on a row under the pointer (drawn/voicepanel.rs). Its row names and toggles it.
import QtQuick

Rectangle {
    id: sw

    // Whether it is on; whether its row is under the pointer; the look and size.
    required property bool on
    required property bool lit
    required property var lk
    required property real s

    readonly property var sh: lk.shape.voice.switch_px
    readonly property color line: lit ? lk.sel_ink : lk.palette.pop_muted
    readonly property color track: on ? lk.palette.badge_bg : "transparent"
    readonly property color knob: on ? lk.palette.badge_ink : line
    // The knob's gap to the track's ends, as it sits centred top to bottom.
    readonly property real inset: (height - dot.height) / 2

    width: sh[0] * s
    height: sh[1] * s
    radius: height / 2
    color: track
    border.width: lk.shape.clip.tick_px[2] * s
    border.color: on ? lk.palette.badge_bg : line
    Accessible.ignored: true

    Rectangle {
        id: dot
        anchors.verticalCenter: parent.verticalCenter
        x: sw.on ? sw.width - width - sw.inset : sw.inset
        width: sw.sh[2] * sw.s
        height: width
        radius: width / 2
        color: sw.knob
        Behavior on x {
            Tween {
                move: sw.lk.motion.clip.swap
            }
        }
    }
}

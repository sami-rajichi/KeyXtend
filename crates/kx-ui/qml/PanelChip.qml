// A list panel's filter chip (spec §7; mock-up .fl), as the clipboard and Quick-fill draw it: a pill, the shown one
// filled and checked so it reads by more than colour. Its fill fades, and it is a big target (WCAG 2.5.8).
import QtQuick

Rectangle {
    id: chip

    // Its label, whether it is the shown one, its check icon, whether the keyboard's language reads right to left, the
    // move its fill fades with, its gate id, the look, fonts, icons and size.
    required property string label
    required property bool shown
    required property string check
    required property bool rtl
    required property var move
    required property string uid
    required property var lk
    required property var res
    required property real s

    // A click or a screen reader's press picked it.
    signal picked

    readonly property var sh: lk.shape.panel.filter_px
    readonly property bool hot: area.containsMouse && !shown
    // Shown, it takes pop_sel with sel_ink; else its text sits on the panel, or on the hover tint.
    readonly property color ink: shown ? lk.sel_ink : lk.palette.pop_ink

    width: words.width + 2 * sh[1] * s
    height: sh[0] * s
    radius: height / 2
    color: shown ? lk.palette.pop_sel : "transparent"
    border.width: shown ? 0 : lk.shape.line.px
    border.color: lk.palette.pop_line
    Accessible.role: Accessible.RadioButton
    Accessible.name: label
    Accessible.checkable: true
    Accessible.checked: shown
    Accessible.id: uid
    Accessible.onPressAction: chip.picked()
    Behavior on color {
        ColourTween {
            move: chip.move
        }
    }

    Hover {
        lk: chip.lk
        radius: chip.radius
        visible: chip.hot
    }
    Row {
        id: words
        anchors.centerIn: parent
        spacing: chip.sh[3] * chip.s
        Icon {
            visible: chip.shown
            anchors.verticalCenter: parent.verticalCenter
            icon: chip.check
            base: chip.res.icons
            ext: chip.res.ext
            px: chip.sh[2] * chip.s
            tint: chip.ink
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            text: chip.label
            font.family: chip.res.fam.ui
            font.pixelSize: chip.lk.shape.chip.font_px[chip.rtl ? 1 : 0] * chip.s
            font.weight: chip.lk.shape.chip.weight
            color: chip.ink
        }
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onActed: chip.picked()
    }
}

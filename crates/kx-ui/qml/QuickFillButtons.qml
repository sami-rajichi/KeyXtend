// A row of Quick-fill's buttons, in the order given (design "The panel"): each the panels' button with its label, its
// icon and its look, dimmed when it cannot act and read as on or off when it is a switch.
pragma ComponentBehavior: Bound
import QtQuick

Row {
    id: bar

    // The buttons (`[{id, label, icon, tone, can, checked, uid, about}]`; `tone` is ghost, `can` true and `checked`
    // null unless given), the look, fonts, icons and size.
    required property var buttons
    required property var lk
    required property var res
    required property real s

    // Button `id` was clicked.
    signal clicked(string id)

    spacing: lk.shape.clip.foot_gap_px * s

    Repeater {
        model: bar.buttons.length
        delegate: ClipButton {
            required property int index
            readonly property var b: bar.buttons[index] || {}

            label: b.label || ""
            icon: b.icon || ""
            tone: b.tone || "ghost"
            can: b.can !== false
            checked: b.checked === undefined ? null : b.checked
            lk: bar.lk
            res: bar.res
            s: bar.s
            uid: b.uid || ""
            about: b.about || ""
            onClicked: bar.clicked(b.id)
        }
    }
}

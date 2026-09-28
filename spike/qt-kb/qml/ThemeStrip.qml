// The test strip under the keyboard (not in the design): one click per theme and per mode, the ring test's switch and the
// status line.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: strip

    // The look, fonts, theme names, mode labels, the chosen theme and mode, and the status line.
    required property var lk
    required property var res
    required property var themes
    required property var modes
    required property int theme
    required property int mode
    required property string status
    // The ring test's switch: its label and whether the test runs.
    required property string ringLabel
    required property bool ringOn

    signal pickTheme(int i)
    signal pickMode(int i)
    signal flipRing

    readonly property var px: lk.shape.extra.strip_px
    readonly property var p: lk.palette

    // One choice, a radio button unless `role` says otherwise: selected ones use the panel's selection colour.
    component Choice: Rectangle {
        id: ch
        property string label: ""
        property bool picked: false
        property int role: Accessible.RadioButton
        signal clicked
        width: word.width + 2 * strip.px[2]
        height: strip.height - strip.px[2]
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        radius: strip.lk.look.key_radius_px
        color: picked ? strip.p.pop_sel : area.containsMouse ? strip.p.pop_hover : "transparent"
        Accessible.role: ch.role
        Accessible.name: ch.label
        Accessible.checkable: true
        Accessible.checked: ch.picked
        Accessible.onPressAction: ch.clicked()
        Accessible.onToggleAction: ch.clicked()
        Text {
            id: word
            anchors.centerIn: parent
            text: ch.label
            font.family: strip.res.fam.latin
            font.pixelSize: strip.px[3]
            color: strip.p.pop_ink
            Accessible.ignored: true
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: ch.clicked()
        }
    }

    // A divider between groups.
    component Split: Rectangle {
        width: strip.lk.shape.line.px
        height: strip.height - 2 * strip.px[2]
        anchors.verticalCenter: parent ? parent.verticalCenter : undefined
        color: strip.p.pop_line
    }

    Rectangle {
        anchors.fill: parent
        radius: strip.lk.look.window_radius_px
        color: strip.p.pop_bg
        border.width: strip.lk.shape.line.px
        border.color: strip.p.pop_line
    }
    Row {
        id: row
        anchors.left: parent.left
        anchors.leftMargin: strip.px[2]
        anchors.verticalCenter: parent.verticalCenter
        height: parent.height
        spacing: strip.px[2]
        Repeater {
            model: strip.themes
            delegate: Choice {
                required property string modelData
                required property int index
                label: modelData
                picked: strip.theme === index
                onClicked: strip.pickTheme(index)
            }
        }
        Split {}
        Repeater {
            model: strip.modes
            delegate: Choice {
                required property string modelData
                required property int index
                label: modelData
                picked: strip.mode === index
                onClicked: strip.pickMode(index)
            }
        }
        Split {}
        Choice {
            label: strip.ringLabel
            picked: strip.ringOn
            role: Accessible.CheckBox
            onClicked: strip.flipRing()
        }
    }
    Text {
        anchors.left: row.right
        anchors.right: parent.right
        anchors.margins: strip.px[2]
        anchors.verticalCenter: parent.verticalCenter
        elide: Text.ElideRight
        text: strip.status
        font.family: strip.res.fam.latin
        font.pixelSize: strip.px[3]
        color: strip.p.pop_muted
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}

// The resize corner (not in the mock-up): a round button on the plate's corner; click, move, click resizes.
import QtQuick

Item {
    id: corner

    // The look, fonts and icons, size, and the button's icon and name.
    required property var lk
    required property var res
    required property real s
    required property var button

    signal clicked
    // The pointer came onto the button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var px: lk.shape.extra.corner_px

    width: px[0] * s
    height: px[0] * s
    Accessible.role: Accessible.Button
    Accessible.name: button.name
    Accessible.onPressAction: corner.clicked()

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: corner.lk.palette.plate
        border.width: corner.lk.shape.line.px
        border.color: corner.lk.palette.pop_line
    }
    Rectangle {
        anchors.fill: parent
        radius: width / 2
        visible: area.containsMouse
        color: corner.lk.palette.pop_hover
    }
    Icon {
        anchors.centerIn: parent
        icon: corner.button.icon
        base: corner.res.icons
        ext: corner.res.ext
        px: corner.px[1] * corner.s
        tint: area.containsMouse ? corner.lk.palette.legend : corner.lk.palette.legend_2
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: corner.tip(containsMouse ? corner.button.name : "", corner)
        onClicked: corner.clicked()
    }
}

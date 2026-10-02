// The resize corner (not in the mock-up): a round button on the plate's corner; click, move, click resizes.
import QtQuick

Item {
    id: corner

    // The look, fonts, icons and texts, and size.
    required property var lk
    required property var res
    required property real s

    signal clicked
    // The pointer came onto the button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var px: lk.shape.extra.corner_px
    readonly property string name: res.bar("corner")
    readonly property bool hot: area.containsMouse

    width: px[0] * s
    height: px[0] * s
    Accessible.role: Accessible.Button
    Accessible.name: name
    Accessible.id: res.uid("bar", "corner")
    Accessible.onPressAction: corner.clicked()

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: corner.lk.palette.plate
        border.width: corner.lk.shape.line.px
        border.color: corner.lk.palette.pop_line
    }
    Hover {
        lk: corner.lk
        radius: width / 2
        visible: corner.hot
        tint: corner.lk.palette.pop_hover
    }
    Icon {
        anchors.centerIn: parent
        icon: corner.lk.icons.bar.corner
        base: corner.res.icons
        ext: corner.res.ext
        px: corner.px[1] * corner.s
        tint: corner.hot ? corner.lk.sel_ink : corner.lk.palette.legend_2_plate
    }
    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: corner.tip(containsMouse ? corner.name : "", corner)
        onClicked: corner.clicked()
    }
}

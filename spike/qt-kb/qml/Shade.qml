// One outer shadow layer under a rounded box, as in CSS box-shadow; a sharp layer is a plain grown box.
import QtQuick
import QtQuick.Effects

Item {
    id: shade

    // The layer: x, y, blur, spread and color, from the theme.
    required property var shadow
    // Corner radius of the box it sits under.
    required property real radius

    readonly property real grow: shadow.spread

    Rectangle {
        visible: shade.shadow.blur <= 0
        x: shade.shadow.x - shade.grow
        y: shade.shadow.y - shade.grow
        width: shade.width + 2 * shade.grow
        height: shade.height + 2 * shade.grow
        radius: shade.radius + shade.grow
        color: shade.shadow.color
    }
    RectangularShadow {
        visible: shade.shadow.blur > 0
        anchors.fill: parent
        offset: Qt.vector2d(shade.shadow.x, shade.shadow.y)
        blur: shade.shadow.blur
        spread: shade.grow
        radius: shade.radius
        color: shade.shadow.color
    }
}

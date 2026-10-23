// One outer shadow layer under a rounded box, as in CSS box-shadow; a sharp layer is a plain grown box. It makes only
// the item its layer draws, sized from the box it sits in, its parent, so it takes no anchors.
pragma ComponentBehavior: Bound
import QtQuick

Loader {
    id: shade

    // The layer: x, y, blur, spread and color, from the theme.
    required property var shadow
    // Corner radius of the box it sits under.
    required property real radius

    // The box it shades, and its size: none once the box is taken down.
    readonly property Item box: parent
    readonly property size room: box ? Qt.size(box.width, box.height) : Qt.size(0, 0)
    readonly property real grow: shadow.spread

    sourceComponent: shadow.blur > 0 ? softC : sharpC

    Component {
        id: sharpC
        Rectangle {
            x: shade.shadow.x - shade.grow
            y: shade.shadow.y - shade.grow
            width: shade.room.width + 2 * shade.grow
            height: shade.room.height + 2 * shade.grow
            radius: shade.radius + shade.grow
            color: shade.shadow.color
        }
    }
    Component {
        id: softC
        Soft {
            width: shade.room.width
            height: shade.room.height
            offset: Qt.vector2d(shade.shadow.x, shade.shadow.y)
            blur: shade.shadow.blur
            spread: shade.grow
            radius: shade.radius
            color: shade.shadow.color
        }
    }
}

// A key's inner shading, drawn as the shade colour with the face laid back over it, moved by the offset. It takes the
// key's values rather than the key, so nothing reads a key that is being taken down.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: shading

    // The key's inset shadow layers, and its cap as CapFace takes it.
    required property var layers
    required property var cap
    required property color topColour
    required property color skirt
    required property real radius
    required property real s
    required property bool pressed
    required property real sink

    clip: true

    Repeater {
        model: shading.layers
        delegate: Item {
            id: inner
            required property var modelData
            readonly property bool edge: modelData.x === 0 && modelData.y === 0
            anchors.fill: parent
            Rectangle {
                visible: !inner.edge
                anchors.fill: parent
                radius: shading.radius
                color: inner.modelData.color
            }
            CapFace {
                visible: !inner.edge
                x: inner.modelData.x
                y: inner.modelData.y
                width: parent.width
                height: parent.height
                cap: shading.cap
                topColour: shading.topColour
                skirt: shading.skirt
                radius: shading.radius
                s: shading.s
                pressed: shading.pressed
                sink: shading.sink
            }
            Rectangle {
                visible: inner.edge
                anchors.fill: parent
                radius: shading.radius
                color: "transparent"
                border.width: inner.modelData.spread
                border.color: inner.modelData.color
            }
        }
    }
}

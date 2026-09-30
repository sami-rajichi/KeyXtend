// A key's inner shading, drawn as the shade colour with the face laid back over it, moved by the offset.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: shading

    // The key it shades.
    required property Item key

    clip: true

    Repeater {
        model: shading.key.p.key_shadow.filter(l => l.inset)
        delegate: Item {
            id: inner
            required property var modelData
            readonly property bool edge: modelData.x === 0 && modelData.y === 0
            readonly property Item key: shading.key
            anchors.fill: parent
            Rectangle {
                visible: !inner.edge
                anchors.fill: parent
                radius: inner.key.radius
                color: inner.modelData.color
            }
            CapFace {
                visible: !inner.edge
                x: inner.modelData.x
                y: inner.modelData.y
                width: parent.width
                height: parent.height
                cap: inner.key.look.cap
                topColour: inner.key.capColour
                skirt: inner.key.capSkirt
                radius: inner.key.radius
                s: inner.key.s
                pressed: inner.key.down
                sink: inner.key.sink
            }
            Rectangle {
                visible: inner.edge
                anchors.fill: parent
                radius: inner.key.radius
                color: "transparent"
                border.width: inner.modelData.spread
                border.color: inner.modelData.color
            }
        }
    }
}

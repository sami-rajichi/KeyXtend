// The D-pad: four arrows cut from a disc and turned, a hub, and Stop while scrolling. Each arrow is its own button.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: pad

    // Its box, the look, fonts and icons, size, and the arrows' and Stop's icons and names.
    required property var box
    required property var lk
    required property var res
    required property real s
    required property var buttons

    // The arrow that is scrolling, by index (up, right, left, down); -1 for none.
    property int dir: -1

    // The pointer came onto an arrow or Stop (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var d: lk.dpad
    readonly property var sh: lk.shape.dpad
    readonly property real r: width / 2
    readonly property real g: sh.gap_px * s / 2
    readonly property real h: Math.sqrt(Math.max(0, r * r - g * g))
    // Each arrow's quarter, as signs of x and y around the centre before the turn.
    readonly property var quarters: [[-1, -1], [1, -1], [-1, 1], [1, 1]]

    x: box.x
    y: box.y
    width: box.w
    height: box.h

    Repeater {
        model: pad.lk.palette.key_shadow.filter(l => !l.inset)
        delegate: Shade {
            required property var modelData
            anchors.fill: parent
            shadow: modelData
            radius: pad.r
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: pad.r
        color: pad.lk.palette.plate
    }

    Item {
        anchors.fill: parent
        rotation: pad.sh.turn_deg
        Repeater {
            model: 4
            delegate: Shape {
                id: arrow
                required property int index
                readonly property var q: pad.quarters[index]
                readonly property bool lit: pad.dir === index
                readonly property color base: lit ? pad.d.on : pad.d.face
                readonly property real cell: pad.r - pad.g
                readonly property real icon: pad.sh.icon_px * pad.s
                readonly property real reach: pad.r - pad.sh.icon_pad * cell - icon / 2
                anchors.fill: parent
                containsMode: Shape.FillContains
                Accessible.role: Accessible.Button
                Accessible.name: pad.buttons[index].name
                Accessible.onPressAction: pad.dir = arrow.lit ? -1 : arrow.index

                ShapePath {
                    strokeWidth: -1
                    fillColor: hit.pressed ? Qt.tint(arrow.base, pad.lk.common.press)
                        : hit.containsMouse ? Qt.tint(arrow.base, pad.lk.palette.hover) : arrow.base
                    startX: pad.r + arrow.q[0] * pad.g
                    startY: pad.r + arrow.q[1] * pad.g
                    PathLine {
                        x: pad.r + arrow.q[0] * pad.g
                        y: pad.r + arrow.q[1] * pad.h
                    }
                    PathArc {
                        x: pad.r + arrow.q[0] * pad.h
                        y: pad.r + arrow.q[1] * pad.g
                        radiusX: pad.r
                        radiusY: pad.r
                        direction: arrow.q[0] * arrow.q[1] > 0 ? PathArc.Counterclockwise : PathArc.Clockwise
                    }
                    PathLine {
                        x: pad.r + arrow.q[0] * pad.g
                        y: pad.r + arrow.q[1] * pad.g
                    }
                }
                Icon {
                    id: glyph
                    x: pad.r + arrow.q[0] * arrow.reach - width / 2
                    y: pad.r + arrow.q[1] * arrow.reach - height / 2
                    rotation: -pad.sh.turn_deg
                    icon: pad.buttons[arrow.index].icon
                    base: pad.res.icons
                    ext: pad.res.ext
                    px: arrow.icon
                    tint: arrow.lit ? pad.d.on_ink : pad.d.ink
                }
                MouseArea {
                    id: hit
                    anchors.fill: parent
                    containmentMask: arrow
                    hoverEnabled: true
                    onContainsMouseChanged: pad.tip(containsMouse ? pad.buttons[arrow.index].name : "", glyph)
                    onClicked: {
                        pad.tip("", null);
                        pad.dir = arrow.lit ? -1 : arrow.index;
                    }
                }
            }
        }
    }

    // The hub takes the pointer, so the arrows under it neither light nor scroll (mock-up .dp-hub).
    Rectangle {
        id: hub
        anchors.fill: parent
        anchors.margins: pad.width * pad.sh.hub
        radius: width / 2
        color: pad.lk.palette.plate
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            containmentMask: QtObject {
                function contains(point: point): bool {
                    return Math.hypot(point.x - hub.radius, point.y - hub.radius) < hub.radius;
                }
            }
        }
    }
    // Stop, shown while an arrow scrolls.
    Rectangle {
        id: stop
        visible: pad.dir >= 0
        anchors.fill: parent
        anchors.margins: pad.width * pad.sh.stop
        radius: width / 2
        color: pad.d.stop
        Accessible.role: Accessible.Button
        Accessible.name: pad.buttons[4].name
        Accessible.onPressAction: pad.dir = -1
        Icon {
            anchors.centerIn: parent
            icon: pad.buttons[4].icon
            base: pad.res.icons
            ext: pad.res.ext
            px: pad.sh.stop_icon_px * pad.s
            tint: pad.d.stop_ink
        }
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            onContainsMouseChanged: pad.tip(containsMouse ? pad.buttons[4].name : "", stop)
            onClicked: {
                pad.tip("", null);
                pad.dir = -1;
            }
        }
    }
}

// The D-pad: four arrows cut from a disc and turned, a hub, and Stop while scrolling. Each arrow is its own button;
// while it comes in a later version it is drawn dimmed and is one button that says so.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: pad

    // Its cell, name and tip (board JSON), the look, fonts, icons and texts, and size.
    required property var box
    required property var lk
    required property var res
    required property real s

    // The arrow that is scrolling, by index (up, right, left, down); -1 for none.
    property int dir: -1

    // The pointer came onto an arrow or Stop (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property bool dim: !!box.dim
    readonly property string name: res.tr(box.name)
    readonly property string later: dim && box.tip ? res.tr(box.tip) : ""
    readonly property var d: lk.dpad
    readonly property var p: lk.palette
    readonly property var sh: lk.shape.dpad
    readonly property var mv: lk.motion.moves
    readonly property real r: width / 2
    readonly property real g: sh.gap_px * s / 2
    readonly property real h: Math.sqrt(Math.max(0, r * r - g * g))
    // Each arrow's `icons.toml [dpad]` id and quarter, as signs of x and y around the centre before the turn.
    readonly property var arrows: [
        { id: "up", q: [-1, -1] },
        { id: "right", q: [1, -1] },
        { id: "left", q: [-1, 1] },
        { id: "down", q: [1, 1] }
    ]
    readonly property string stopId: "stop"
    // The pad sits half a gap inside its cell, like a key.
    readonly property real inset: lk.look.gap_px * s / 2

    x: box.x + inset
    y: box.y + inset
    width: box.w - 2 * inset
    height: box.h - 2 * inset

    Accessible.role: dim ? Accessible.Button : Accessible.Grouping
    Accessible.name: name
    Accessible.id: box.name
    Accessible.description: later

    Repeater {
        model: pad.lk.palette.key_shadow.filter(l => !l.inset)
        delegate: Shade {
            required property var modelData
            shadow: modelData
            radius: pad.r
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: pad.r
        color: pad.lk.palette.plate
        Behavior on color {
            ColourTween {
                move: pad.mv.plate
            }
        }
    }

    Item {
        anchors.fill: parent
        rotation: pad.sh.turn_deg
        Repeater {
            model: pad.arrows
            delegate: Shape {
                id: arrow
                required property var modelData
                required property int index
                readonly property var q: modelData.q
                readonly property string name: pad.res.dpad(modelData.id)
                readonly property bool lit: pad.dir === index
                // Colours ease like the keys' (mock-up .dp transition).
                property color base: pad.dim ? pad.p.dim : lit ? pad.d.on : pad.d.face
                property color ink: pad.dim ? pad.p.dim_ink : lit ? pad.d.on_ink : pad.d.ink
                readonly property real cell: pad.r - pad.g
                readonly property real icon: pad.sh.icon_px * pad.s
                readonly property real reach: pad.r - pad.sh.icon_pad * cell - icon / 2
                anchors.fill: parent
                containsMode: Shape.FillContains
                Accessible.role: Accessible.Button
                Accessible.name: name
                Accessible.id: pad.res.uid("dpad", modelData.id)
                Accessible.ignored: pad.dim
                Accessible.onPressAction: if (!pad.dim) pad.dir = arrow.lit ? -1 : arrow.index

                Behavior on base {
                    ColourTween {
                        move: pad.mv.colour
                    }
                }
                Behavior on ink {
                    ColourTween {
                        move: pad.mv.colour
                    }
                }

                // High contrast outlines the arrow under the pointer in Highlight.
                ShapePath {
                    strokeWidth: pad.lk.contrast && hit.containsMouse ? pad.lk.shape.hc_outline.px : -1
                    strokeColor: pad.lk.hover_line
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
                    icon: pad.lk.icons.dpad[arrow.modelData.id]
                    base: pad.res.icons
                    ext: pad.res.ext
                    px: arrow.icon
                    tint: arrow.ink
                }
                // It acts at let-go, wherever the pointer went (ADR-0024).
                Act {
                    id: hit
                    anchors.fill: parent
                    enabled: !pad.dim
                    containmentMask: arrow
                    hoverEnabled: true
                    onContainsMouseChanged: pad.tip(containsMouse ? arrow.name : "", glyph)
                    onActed: {
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
        Behavior on color {
            ColourTween {
                move: pad.mv.plate
            }
        }
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
    // A dimmed pad shows its name and when it comes over the whole disc.
    MouseArea {
        visible: pad.dim
        anchors.fill: parent
        hoverEnabled: true
        containmentMask: QtObject {
            function contains(point: point): bool {
                return Math.hypot(point.x - pad.r, point.y - pad.r) < pad.r;
            }
        }
        onContainsMouseChanged: pad.tip(containsMouse ? pad.res.noteTip(pad.name, pad.later) : "", pad)
    }
    // Stop pops in while an arrow scrolls and out after (mock-up .dp-stop); a spring curve overshoots a little.
    Rectangle {
        id: stop
        scale: pad.dir >= 0 ? 1 : 0
        visible: scale > 0
        anchors.fill: parent
        anchors.margins: pad.width * pad.sh.stop
        radius: width / 2
        color: pad.d.stop
        Accessible.role: Accessible.Button
        Accessible.name: pad.res.dpad(pad.stopId)
        Accessible.id: pad.res.uid("dpad", pad.stopId)
        Accessible.onPressAction: pad.dir = -1
        Behavior on scale {
            Tween {
                move: pad.mv.stop
            }
        }
        Icon {
            anchors.centerIn: parent
            icon: pad.lk.icons.dpad[pad.stopId]
            base: pad.res.icons
            ext: pad.res.ext
            px: pad.sh.stop_icon_px * pad.s
            tint: pad.d.stop_ink
        }
        Act {
            anchors.fill: parent
            hoverEnabled: true
            onContainsMouseChanged: pad.tip(containsMouse ? stop.Accessible.name : "", stop)
            onActed: {
                pad.tip("", null);
                pad.dir = -1;
            }
        }
    }
}

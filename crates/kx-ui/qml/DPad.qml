// The D-pad: four arrows cut from a disc and turned, each centred in its quarter of the ring as the iPod click wheel's
// labels, a hub, and Stop with its speed dots while scrolling. Each arrow is its own button; while the scroll module is
// off the pad is dimmed and is one button that says so.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: pad

    // Its cell and name (board JSON) with its dim, tip, way and speed (state JSON), the look, fonts, icons and texts,
    // and size.
    required property var box
    required property var lk
    required property var res
    required property real s

    // The pointer came onto an arrow or Stop (its name) or left it (empty).
    signal tip(string name, Item at)
    // An arrow was tapped, by its `icons.toml [dpad]` id, or Stop, as empty.
    signal tapped(string dir)
    // The pointer came onto the pad or left it, also by the window hiding with the pointer there.
    signal hovered(bool on)

    readonly property bool dim: !!box.dim
    readonly property string name: res.tr(box.name)
    // Why it is dimmed, the scroll-off tip, else empty.
    readonly property string off: dim && box.tip ? res.tr(box.tip) : ""
    // The arrow scrolling by its id, empty while still, and its speed, as the badge words it.
    readonly property string dir: box.dir || ""
    readonly property int pace: box.fast ? 1 : 0
    readonly property var speeds: ["badge.slow", "badge.fast"]
    readonly property string speed: dir !== "" ? res.tr(speeds[pace]) : ""
    readonly property var d: lk.dpad
    readonly property var p: lk.palette
    readonly property var sh: lk.shape.dpad
    readonly property var mv: lk.motion.moves
    readonly property real r: width / 2
    readonly property real g: sh.gap_px * s / 2
    readonly property real h: Math.sqrt(Math.max(0, r * r - g * g))
    // The hub's radius, and how far out each arrow's icon sits: `reach` of the way across the ring of arrows.
    readonly property real hubR: r * (1 - 2 * sh.hub)
    readonly property real out: hubR + sh.reach * (r - hubR)
    // Each arrow's upright box for readers, around its icon: half its depth along its way, as far as the nearer of the
    // hub's edge and the pad's, and half its width across, so the four boxes meet without overlap.
    readonly property real deep: Math.min(out - hubR, r - out)
    readonly property real wide: out - deep
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
    // The pointer is on the disc of a shown pad, and what the keyboard was last told.
    readonly property bool shown: !!Window.window && Window.window.visible
    readonly property bool over: shown && hover.hovered && Math.hypot(hover.point.position.x - r, hover.point.position.y - r) < r
    property bool told: false

    x: box.x + inset
    y: box.y + inset
    width: box.w - 2 * inset
    height: box.h - 2 * inset

    Accessible.role: dim ? Accessible.Button : Accessible.Grouping
    Accessible.name: name
    Accessible.id: box.name
    Accessible.description: off

    // Told once the event's changes settled, so moving between arrows, gaps and the hub tells nothing.
    onOverChanged: Qt.callLater(pad.tell)
    // A pad made again, as for a new board, leaves no pointer told on the old one.
    Component.onDestruction: if (told) hovered(false)
    function tell() {
        if (over === told)
            return;
        told = over;
        hovered(over);
    }

    // It watches the pointer without taking it, so the arrows under it still hover.
    HoverHandler {
        id: hover
    }
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
                readonly property var q: modelData.q
                readonly property string name: pad.res.dpad(modelData.id)
                readonly property bool lit: pad.dir === modelData.id
                // Colours ease like the keys' (mock-up .dp transition).
                property color base: pad.dim ? pad.p.dim : lit ? pad.d.on : pad.d.face
                property color ink: pad.dim ? pad.p.dim_ink : lit ? pad.d.on_ink : pad.d.ink
                // Pressed, it takes the keys' press shade, and in high contrast their press text, at once (Key.qml).
                readonly property color fill: hit.pressed ? Qt.tint(base, pad.p.press) : hit.containsMouse ? Qt.tint(base, pad.p.hover) : base
                readonly property color drawn: hit.pressed && pad.p.press_ink ? pad.p.press_ink : ink
                readonly property real icon: pad.sh.icon_px * pad.s
                anchors.fill: parent
                containsMode: Shape.FillContains
                // Readers find the arrow by its upright box below, as a turned item's box is the whole pad's.
                Accessible.ignored: true

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
                    fillColor: arrow.fill
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
                // On its quarter's middle line, `out` from the centre.
                Icon {
                    id: glyph
                    x: pad.r + arrow.q[0] * pad.out * Math.SQRT1_2 - width / 2
                    y: pad.r + arrow.q[1] * pad.out * Math.SQRT1_2 - height / 2
                    rotation: -pad.sh.turn_deg
                    icon: pad.lk.icons.dpad[arrow.modelData.id]
                    base: pad.res.icons
                    ext: pad.res.ext
                    px: arrow.icon
                    tint: arrow.drawn
                }
                // The arrow for readers and voice control: an upright box centred on its icon, its depth along the
                // arrow's way on screen after the turn.
                Item {
                    readonly property real way: Math.atan2(arrow.q[1], arrow.q[0]) + pad.sh.turn_deg * Math.PI / 180
                    readonly property real along: Math.abs(Math.cos(way))
                    readonly property real down: Math.abs(Math.sin(way))
                    x: glyph.x + (glyph.width - width) / 2
                    y: glyph.y + (glyph.height - height) / 2
                    width: 2 * (along * pad.deep + down * pad.wide)
                    height: 2 * (down * pad.deep + along * pad.wide)
                    rotation: -pad.sh.turn_deg
                    Accessible.role: Accessible.Button
                    Accessible.name: arrow.name
                    Accessible.id: pad.res.uid("dpad", arrow.modelData.id)
                    Accessible.description: arrow.lit ? pad.speed : ""
                    Accessible.ignored: pad.dim
                    Accessible.onPressAction: if (!pad.dim) pad.tapped(arrow.modelData.id)
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
                        pad.tapped(arrow.modelData.id);
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
    // A dimmed pad shows its name and why over the whole disc.
    MouseArea {
        visible: pad.dim
        anchors.fill: parent
        hoverEnabled: true
        containmentMask: QtObject {
            function contains(point: point): bool {
                return Math.hypot(point.x - pad.r, point.y - pad.r) < pad.r;
            }
        }
        onContainsMouseChanged: pad.tip(containsMouse ? pad.res.noteTip(pad.name, pad.off) : "", pad)
    }
    Stop {
        pad: pad
    }
}

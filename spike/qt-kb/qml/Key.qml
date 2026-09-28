// One key: its shadows, cap, inner shading, legends or icon, LED and pointer feedback. Readers hear its name.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects

Item {
    id: key

    // The key's box and look, its changing state, the look, the language key, fonts and icons, and size.
    required property var kv
    required property var ks
    required property var lk
    required property var lang
    required property var res
    required property real s

    // A click; `back` is the language key's left part.
    signal hit(bool back)
    // The pointer came onto an icon key (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var p: lk.palette
    readonly property var look: lk.look
    readonly property var press: lk.shape.press
    readonly property bool on: ks ? ks.on : false
    readonly property bool down: area.pressed
    // The palette tokens for the cap, legend and skirt, which spike-core picks from the kind and state.
    readonly property var paint: ks ? ks.paint : kv.paint
    readonly property color fillColour: p[paint.fill]
    readonly property color ink: p[paint.ink]
    readonly property color skirt: p.skirt ? p.skirt[paint.skirt] : fillColour
    readonly property real radius: look.key_radius_px
    readonly property color onColour: p.on
    readonly property string name: ks && ks.name.length > 0 ? ks.name : kv.name

    x: kv.x
    y: kv.y
    width: kv.w
    height: kv.h

    Accessible.role: Accessible.Button
    Accessible.name: key.name
    Accessible.checkable: kv.kind === "mod" || kv.side
    Accessible.checked: key.on
    Accessible.onPressAction: key.hit(false)

    // A locked key's glow (Dolch).
    RectangularShadow {
        visible: key.on && !key.lk.contrast && key.look.lock_glow_px > 0
        anchors.fill: cap
        blur: key.look.lock_glow_px * key.s
        radius: key.radius
        color: Qt.rgba(key.onColour.r, key.onColour.g, key.onColour.b, key.look.lock_glow_mix)
    }

    Item {
        id: cap
        width: key.width
        height: key.height
        transform: [
            Scale {
                origin.x: cap.width / 2
                origin.y: cap.height / 2
                xScale: key.down ? key.press.scale : 1
                yScale: key.down ? key.press.scale : 1
            },
            Translate {
                y: key.down ? key.press.sink_px * key.s : 0
            }
        ]

        Repeater {
            model: key.p.key_shadow.filter(l => !l.inset)
            delegate: Shade {
                required property var modelData
                anchors.fill: parent
                shadow: modelData
                radius: key.radius
            }
        }
        CapFace {
            anchors.fill: parent
            cap: key.look.cap
            topColour: key.fillColour
            skirt: key.skirt
            radius: key.radius
            s: key.s
            pressed: key.down
        }
        // Inner shading, drawn as the shade colour with the face laid back over it, moved by the offset.
        Item {
            anchors.fill: parent
            clip: true
            Repeater {
                model: key.p.key_shadow.filter(l => l.inset)
                delegate: Item {
                    id: inner
                    required property var modelData
                    readonly property bool edge: modelData.x === 0 && modelData.y === 0
                    anchors.fill: parent
                    Rectangle {
                        visible: !inner.edge
                        anchors.fill: parent
                        radius: key.radius
                        color: inner.modelData.color
                    }
                    CapFace {
                        visible: !inner.edge
                        x: inner.modelData.x
                        y: inner.modelData.y
                        width: parent.width
                        height: parent.height
                        cap: key.look.cap
                        topColour: key.fillColour
                        skirt: key.skirt
                        radius: key.radius
                        s: key.s
                        pressed: key.down
                    }
                    Rectangle {
                        visible: inner.edge
                        anchors.fill: parent
                        radius: key.radius
                        color: "transparent"
                        border.width: inner.modelData.spread
                        border.color: inner.modelData.color
                    }
                }
            }
        }
        Rectangle {
            anchors.fill: parent
            radius: key.radius
            visible: area.containsMouse || key.down
            color: key.down ? key.lk.common.press : key.p.hover
        }
        KeyText {
            anchors.fill: parent
            kv: key.kv
            ks: key.ks
            lk: key.lk
            lang: key.lang
            res: key.res
            s: key.s
            ink: key.ink
        }
        // The LED, with its glow while lit.
        Item {
            readonly property real d: key.lk.shape.led.size_px * key.s
            readonly property real inset: key.lk.shape.led.inset_px * key.s
            x: cap.width - inset - d
            y: inset
            width: d
            height: d
            RectangularShadow {
                visible: key.on && !key.lk.contrast
                anchors.fill: parent
                blur: key.look.led_glow_px * key.s
                radius: parent.width / 2
                color: key.p.led_on
            }
            Rectangle {
                anchors.fill: parent
                radius: width / 2
                color: key.on ? key.p.led_on : key.p.led_off
            }
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: if (key.kv.icon) key.tip(containsMouse ? key.name : "", key)
        onClicked: mouse => key.hit(key.kv.kind === "lang" && mouse.x < width * key.lk.shape.lang.back_share)
    }
}

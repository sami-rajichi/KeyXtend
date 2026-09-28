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
    readonly property var mv: lk.motion.moves
    readonly property bool on: ks ? ks.on : false
    readonly property bool rec: ks ? ks.rec : false
    readonly property bool down: area.pressed
    // The palette tokens for the cap, legend and skirt, which spike-core picks from the kind and state.
    readonly property var paint: ks ? ks.paint : kv.paint
    // Colours ease to each new state or theme (mock-up .cap transition).
    property color fillColour: p[paint.fill]
    property color ink: p[paint.ink]
    property color skirt: p.skirt ? p.skirt[paint.skirt] : p[paint.fill]
    // How far the cap has sunk, 0 to 1, and how far a recording cap has brightened, 0 to 1.
    property real sink: down ? 1 : 0
    property real lift: 0
    readonly property real glowBy: 1 + (lk.motion.amount.glow_lift - 1) * lift
    readonly property color capColour: lift > 0 ? Qt.lighter(fillColour, glowBy) : fillColour
    readonly property color capSkirt: lift > 0 ? Qt.lighter(skirt, glowBy) : skirt
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

    Behavior on fillColour {
        ColourTween {
            move: key.mv.colour
        }
    }
    Behavior on ink {
        ColourTween {
            move: key.mv.colour
        }
    }
    Behavior on skirt {
        ColourTween {
            move: key.mv.colour
        }
    }
    Behavior on sink {
        Tween {
            move: key.mv.press
        }
    }
    // A recording cap glows (mock-up .key.rec .cap); loops never run at 0 ms.
    SequentialAnimation on lift {
        running: key.rec && key.mv.glow.ms > 0
        loops: Animation.Infinite
        onStopped: key.lift = 0

        Tween {
            move: key.mv.glow
            share: 0.5
            to: 1
        }
        Tween {
            move: key.mv.glow
            share: 0.5
            to: 0
        }
    }

    // A locked key's glow (Dolch); the recording key has its own.
    RectangularShadow {
        opacity: key.on && !key.rec ? 1 : 0
        visible: opacity > 0 && !key.lk.contrast && key.look.lock_glow_px > 0
        anchors.fill: cap
        blur: key.look.lock_glow_px * key.s
        radius: key.radius
        color: Qt.rgba(key.onColour.r, key.onColour.g, key.onColour.b, key.look.lock_glow_mix)
        Behavior on opacity {
            Tween {
                move: key.mv.colour
            }
        }
    }

    Item {
        id: cap
        width: key.width
        height: key.height
        transform: [
            Scale {
                id: shrink
                origin.x: cap.width / 2
                origin.y: cap.height / 2
                xScale: 1 - (1 - key.press.scale) * key.sink
                yScale: shrink.xScale
            },
            Translate {
                y: key.press.sink_px * key.s * key.sink
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
            topColour: key.capColour
            skirt: key.capSkirt
            radius: key.radius
            s: key.s
            pressed: key.down
            sink: key.sink
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
                        topColour: key.capColour
                        skirt: key.capSkirt
                        radius: key.radius
                        s: key.s
                        pressed: key.down
                        sink: key.sink
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
        // The LED, with its glow while lit; it pulses while recording (mock-up .key.rec .led).
        Item {
            id: led
            readonly property real d: key.lk.shape.led.size_px * key.s
            readonly property real inset: key.lk.shape.led.inset_px * key.s
            x: cap.width - inset - d
            y: inset
            width: d
            height: d

            SequentialAnimation on opacity {
                running: key.rec && key.mv.led.ms > 0
                loops: Animation.Infinite
                onStopped: led.opacity = 1

                Tween {
                    move: key.mv.led
                    share: 0.5
                    to: key.lk.motion.amount.pulse_low
                }
                Tween {
                    move: key.mv.led
                    share: 0.5
                    to: 1
                }
            }
            RectangularShadow {
                opacity: key.on ? 1 : 0
                visible: opacity > 0 && !key.lk.contrast
                anchors.fill: parent
                blur: key.look.led_glow_px * key.s
                radius: parent.width / 2
                color: key.p.led_on
                Behavior on opacity {
                    Tween {
                        move: key.mv.colour
                    }
                }
            }
            Rectangle {
                anchors.fill: parent
                radius: width / 2
                color: key.on ? key.p.led_on : key.p.led_off
                Behavior on color {
                    ColourTween {
                        move: key.mv.colour
                    }
                }
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

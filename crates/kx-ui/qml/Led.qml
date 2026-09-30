// A key's LED, with its glow while lit; it pulses while recording (mock-up .key.rec .led).
import QtQuick
import QtQuick.Effects

Item {
    id: led

    // The key it sits on.
    required property Item key
    readonly property real d: key.lk.shape.led.size_px * key.s
    readonly property real inset: key.lk.shape.led.inset_px * key.s
    // The recording pulse runs.
    readonly property bool pulsing: pulse.running

    x: key.width - inset - d
    y: inset
    width: d
    height: d

    SequentialAnimation on opacity {
        id: pulse
        running: led.key.rec && led.key.mv.led.ms > 0
        loops: Animation.Infinite
        onStopped: led.opacity = 1

        Tween {
            move: led.key.mv.led
            share: 0.5
            to: led.key.lk.motion.amount.pulse_low
        }
        Tween {
            move: led.key.mv.led
            share: 0.5
            to: 1
        }
    }
    RectangularShadow {
        opacity: led.key.on ? 1 : 0
        visible: opacity > 0 && !led.key.lk.contrast
        anchors.fill: parent
        blur: led.key.look.led_glow_px * led.key.s
        radius: parent.width / 2
        color: led.key.p.led_on
        Behavior on opacity {
            Tween {
                move: led.key.mv.colour
            }
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: led.key.on ? led.key.p.led_on : led.key.p.led_off
        Behavior on color {
            ColourTween {
                move: led.key.mv.colour
            }
        }
    }
}

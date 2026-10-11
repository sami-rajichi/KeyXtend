// A key's LED, with its glow made only while lit; it pulses while recording (mock-up .key.rec .led). Its colour and
// glow ease on the key (`ledColour`, `ledLit`).
pragma ComponentBehavior: Bound
import QtQuick

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
    // High contrast has no glow.
    Loader {
        anchors.fill: parent
        active: led.key.ledLit > 0 && !led.key.lk.contrast
        sourceComponent: Soft {
            opacity: led.key.ledLit
            blur: led.key.look.led_glow_px * led.key.s
            radius: led.width / 2
            color: led.key.p.led_on
        }
    }
    Rectangle {
        anchors.fill: parent
        radius: width / 2
        color: led.key.ledColour
    }
}

// Turns a list panel's page at each notch of the mouse wheel or touchpad (ADR-0040), adding a fine wheel's small steps up
// to a notch. It takes no press and moves no aim or focus, and nothing needs it (ADR-0008).
import QtQuick

WheelHandler {
    // What one notch reports.
    required property real notch
    // How long the wheel rests, in seconds, before steps short of a notch are dropped.
    required property real rest
    // What has come since the last whole notch: above 0 goes back.
    property real held: 0

    // A notch asks for the page before (by -1) or after (by 1).
    signal turn(int by)

    target: null
    acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
    activeTimeout: rest
    onActiveChanged: {
        if (!active)
            held = 0;
    }
    onWheel: event => {
        held += event.angleDelta.y;
        const notches = Math.trunc(held / notch);
        held -= notches * notch;
        for (let i = 0; i < Math.abs(notches); i++)
            turn(notches > 0 ? -1 : 1);
    }
}

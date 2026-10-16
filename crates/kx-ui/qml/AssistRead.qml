// The pointer overlay's reader: a wake reads the hub's state on the event loop's next pass, never on a frame, or once
// `paceMs` has passed since the last read while changes stream. Only a read lets the hub's next change wake, and
// nothing runs while no wake waits.
import QtQuick

QtObject {
    id: reader

    // The hub's Ui object, the least time between reads in ms (`[ui] poll_ms`), and the clock in ms.
    required property QtObject ui
    required property int paceMs
    required property var now
    // The state as last read, null before the first read, and when that read was, by `now`.
    property var state: null
    property real readAt: -Infinity
    // Holds a wake back for the rest of the pace after a read; it runs only while that wake waits.
    property Timer pace: Timer {
        onTriggered: reader.read()
    }

    // A wake: the state is read on the event loop's next pass, or once the pace since the last read is up; a clock
    // that stepped back waits one pace at most, and a wake while a read waits adds nothing.
    function woke() {
        if (pace.running)
            return;
        const rest = Math.min(readAt + paceMs - now(), paceMs);
        if (rest > 0) {
            pace.interval = Math.ceil(rest);
            pace.start();
        } else {
            Qt.callLater(reader.read);
        }
    }
    // Reads the state, which lets the hub's next change wake again.
    function read() {
        pace.stop();
        readAt = now();
        state = JSON.parse(ui.assistJson());
    }
}

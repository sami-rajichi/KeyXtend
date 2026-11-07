// Lies over a window's parts and hears each press without taking it, so the press still reaches what lies below.
import QtQuick

Item {
    id: watch

    // How long after a press another counts as following it, in ms: the system's double-click time.
    property int doubleMs: Qt.styleHints.mouseDoubleClickInterval
    // When the last press landed, in ms; 0 with none.
    property real last: 0

    // A press landed on the window.
    signal touched
    // That press does not follow another inside the wait, so a hand's second, third or later tap never says it.
    signal alone

    // Tells `touched`, then `alone` unless this press follows the one before inside the wait.
    function heard() {
        touched();
        const now = Date.now();
        const follows = last > 0 && now >= last && now - last < doubleMs;
        last = now;
        if (!follows)
            alone();
    }

    anchors.fill: parent

    PointHandler {
        onActiveChanged: if (active) watch.heard()
    }
}

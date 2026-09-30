// Lies over a window's parts and hears each press without taking it, so the press still reaches what lies below.
import QtQuick

Item {
    id: watch

    // A press landed on the window.
    signal touched

    anchors.fill: parent

    PointHandler {
        onActiveChanged: if (active) watch.touched()
    }
}

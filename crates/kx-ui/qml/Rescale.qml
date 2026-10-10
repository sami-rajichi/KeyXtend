// Holds its window to the size and place its bindings give in Qt units: on a new display scale Qt draws at that scale
// but leaves the native window as it was, and may resize it during a move, so each binding through `keep` runs again.
import QtQuick

Item {
    id: rescale

    // The window's scale, which tests set; a new one sets every kept value again.
    property real ratio: Window.window ? Window.window.devicePixelRatio : 1
    // Bumped after the window's size or place changed, so a change from outside gives way to the bindings.
    property int turn: 0

    // `v`, set again on a new scale or after a change from outside; a window's size and place bindings go through it.
    function keep(v) {
        void [rescale.ratio, rescale.turn];
        return v;
    }
    function bump() {
        rescale.turn++;
    }

    Connections {
        target: rescale.Window.window
        function onXChanged() {
            Qt.callLater(rescale.bump);
        }
        function onYChanged() {
            Qt.callLater(rescale.bump);
        }
        function onWidthChanged() {
            Qt.callLater(rescale.bump);
        }
        function onHeightChanged() {
            Qt.callLater(rescale.bump);
        }
    }
}

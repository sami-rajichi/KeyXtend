// The keyboard window's Quick-fill panel: it reads the QuickFill object's view at each `viewChanged` wake, which lets
// the next wake come, and holds the panel with its shadow while the keyboard shows and a view is open.
import QtQuick

Loader {
    id: fills

    // The keyboard window, whose QuickFill object holds the view; a window without one shows no panel.
    required property Window win

    // Bumped at each wake, so the view is read again.
    property int woke: 0
    // The view as last read, null while the panel is closed.
    readonly property var view: read(win.fill, woke)

    // The view `fill` gives now, none without the object; `woke` only reruns the binding that calls it.
    function read(fill) {
        return fill ? JSON.parse(fill.view()) : null;
    }

    active: win.ready && win.visible && view !== null
    sourceComponent: QuickFillPanel {
        win: fills.win
        view: fills.view
    }

    Connections {
        target: fills.win.fill
        function onViewChanged() {
            fills.woke++;
        }
    }
}

// The keyboard window's clipboard panel: it reads the hub's view at each `clipChanged` wake, which lets the next wake
// come, and holds the panel with its shadow while the keyboard shows and a view is open.
import QtQuick

Loader {
    id: clips

    // The keyboard window, whose Ui object holds the view.
    required property Window win

    // Bumped at each wake, so the view is read again.
    property int woke: 0
    // The view as last read, null while the panel is closed.
    readonly property var view: read(win.ui, woke)

    // The view `ui` gives now, none once it has gone; `woke` only reruns the binding that calls it.
    function read(ui) {
        return ui ? JSON.parse(ui.clip()) : null;
    }

    active: win.ready && win.visible && view !== null
    sourceComponent: ClipPanel {
        win: clips.win
        view: clips.view
    }

    Connections {
        target: clips.win.ui
        function onClipChanged() {
            clips.woke++;
        }
    }
}

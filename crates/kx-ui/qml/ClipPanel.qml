// The clipboard panel with its shadow (spec §7): centred above the keyboard, or below it without room (§3.4), placed
// again as the keyboard moves, then opened. A press on it counts as the keyboard's.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: panel

    // The keyboard window, whose look, keyboard size and texts the panel takes, and the panel's view (`Ui.clip`).
    required property Window win
    required property var view

    // The panel's own window, which the guard marks and the pointer overlay's badge hides over.
    readonly property alias pane: pane

    lk: win.lk
    box: Qt.rect(pane.x, pane.y, pane.width, pane.height)
    radius: pane.radius
    follow: pane.body

    // Puts the panel above the keyboard, centred on it.
    function place() {
        const p = panel.win.spots.popSpot(pane.width, pane.height, NaN);
        if (p) {
            pane.x = p.x;
            pane.y = p.y;
        }
    }
    // Placed again as the keyboard moves or changes size.
    Connections {
        target: panel.win
        function onAtChanged() {
            panel.place();
        }
        function onDrawnChanged() {
            panel.place();
        }
    }

    ClipBox {
        id: pane
        ui: panel.win.ui
        lk: panel.win.lk
        res: panel.win.res
        s: panel.win.s
        view: panel.view
        rows: panel.win.cfg.pageRows
        rtl: !!panel.win.tongue.rtl
        onTouched: panel.win.touched()
        onRescaled: Qt.callLater(panel.place)
        onWidthChanged: if (visible) panel.place()
        onHeightChanged: if (visible) panel.place()
        Component.onCompleted: Qt.callLater(() => {
            panel.place();
            panel.visible = true;
            pane.open();
            panel.win.guardRun();
        })
    }
}

// The clipboard panel with its shadow (spec §7): centred above the keyboard, else below it, else shrunk to the room but
// never below the theme's least fit (§3.4). A press on it counts as the keyboard's.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: panel

    // The keyboard window, whose look, keyboard size and texts the panel takes, and the panel's view (`Ui.clip`).
    required property Window win
    required property var view

    // The panel's own window, which the guard marks and the pointer overlay's badge hides over.
    readonly property alias pane: pane
    // The share of its size the panel takes, as the hub works it out for the room beside the keyboard.
    property real fit: 1
    // A note came or went, or the display scale changed, so the next settle takes the hub's fit as it comes.
    property bool renew: false
    readonly property bool noted: !!view.note

    lk: win.lk
    box: Qt.rect(pane.x, pane.y, pane.width, pane.height)
    radius: pane.radius
    follow: pane.body

    // Places the panel at the hub's spot and fit for its full size, from its last layout. A fresh fit is taken as it
    // comes; after a layout the fit only shrinks, so text that wraps differently at a new size never makes it swing.
    function place(fresh) {
        const p = panel.win.spots.popFit(pane.pn.width_px * panel.win.s, pane.unit * panel.win.s, NaN);
        if (!p)
            return;
        if (fresh || p.fit < fit)
            fit = p.fit;
        pane.x = p.x;
        pane.y = p.y;
    }
    // Places it again once a layout has settled.
    function settle() {
        if (!pane.visible)
            return;
        place(renew);
        renew = false;
    }
    // Placed again as the keyboard moves or changes size.
    Connections {
        target: panel.win
        function onAtChanged() {
            panel.place(true);
        }
        function onDrawnChanged() {
            panel.place(true);
        }
    }
    onNotedChanged: renew = true
    // Settles once the layout is done, and never after the panel is gone, as it goes with the panel.
    Timer {
        id: settling
        interval: 0
        onTriggered: panel.settle()
    }

    ClipBox {
        id: pane
        ui: panel.win.ui
        lk: panel.win.lk
        res: panel.win.res
        s: panel.win.s * panel.fit
        view: panel.view
        rows: panel.win.cfg.pageRows
        rtl: !!panel.win.tongue.rtl
        onTouched: panel.win.touched()
        onRescaled: {
            panel.renew = true;
            settling.restart();
        }
        onUnitChanged: settling.restart()
        Component.onCompleted: Qt.callLater(() => {
            panel.place(true);
            panel.visible = true;
            pane.open();
            panel.win.guardRun();
        })
    }
}

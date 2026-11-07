// A list panel with its shadow, as the clipboard and Quick-fill open (spec §7): centred above the keyboard, else below
// it, else shrunk to the room but never below the theme's least fit (§3.4). The panel's own window is its `pane`.
pragma ComponentBehavior: Bound
import QtQuick

Glow {
    id: panel

    // The keyboard window, whose look, keyboard size and texts the panel takes; the panel's view; its own window, which
    // the guard marks and the pointer overlay's badge hides over; and that window's width at size 1.
    required property Window win
    required property var view
    required property Window pane
    required property real wide

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
    // comes; after a layout the fit only shrinks, so rewrapped text never makes it swing, and a smaller fit it keeps
    // takes the hub's spot for its own size, so it stays centred.
    function place(fresh) {
        const w = wide * win.s, h = pane.unit * win.s;
        let p = win.spots.popFit(w, h, NaN);
        if (!p)
            return;
        if (fresh || p.fit < fit)
            fit = p.fit;
        else if (fit < p.fit)
            p = win.spots.popFit(w * fit, h * fit, NaN) || p;
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
    // The display scale changed, so the pane's place, set in the old Qt units, is stale.
    function rescaled() {
        renew = true;
        settling.restart();
    }
    // The pane laid out again.
    function relaid() {
        settling.restart();
    }
    // Opens the pane once it is made: placed, shown with its shadow, popped in and guarded.
    function start() {
        Qt.callLater(() => {
            place(true);
            panel.visible = true;
            pane.open();
            win.guardRun();
        });
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
}

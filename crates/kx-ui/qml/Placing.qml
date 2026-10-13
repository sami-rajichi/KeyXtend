// Where the keyboard window sits: it tells the keyboard, and places the window again on a new display scale, screen,
// screen size or corner. A change that comes while the window moves or resizes is placed once that ends.
import QtQuick

Item {
    id: placing

    // The keyboard window.
    required property var win
    // The keyboard has placed the window; a restarted keyboard places it again.
    property bool placed: false
    // A change came while the window moved or resized, so it is placed again once that ends.
    property bool owed: false
    // The screen the window is on, whose size it follows; tests give a stand-in.
    property var display: win.screen

    // Tells the keyboard where the window is, its corner and all; the first time, it answers with the saved or home
    // place. The state is read again, so `st.at` holds the place the keyboard keeps even when it answers no change.
    function place() {
        const w = win;
        if (!w.running)
            return;
        // A window made again, as after a device loss, first goes where the keyboard keeps it, so nothing moves.
        if (!placed && w.st && w.st.at)
            w.moveTo(w.st.at);
        const p = w.spots.physical(w.at.x, w.at.y);
        placed = true;
        owed = false;
        w.redraw(w.keyboard.placed(p.x, p.y, w.width * p.dpr, w.height * p.dpr, w.unitOverhang), true);
    }
    // The cut again and, unless the window moves or resizes, a new placement where it is in pixels, for its size in the
    // new ones; the window then goes where the keyboard puts it, in the new Qt units.
    function rescreen() {
        const w = win;
        w.reshape();
        if (w.mover && w.mover.busy) {
            owed = true;
        } else if (placed && w.st && w.st.at) {
            owed = false;
            w.redraw(w.keyboard.placed(w.st.at.x, w.st.at.y, w.wide * w.ratio, w.high * w.ratio, w.unitOverhang), true);
        }
    }

    Connections {
        target: placing.win
        function onRunningChanged() {
            if (!placing.win.running)
                placing.placed = false;
        }
        function onScreenChanged() {
            Qt.callLater(placing.rescreen);
        }
        // A new corner, or a look read after the first placement, so the keyboard keeps the whole corner on screen.
        function onUnitOverhangChanged() {
            if (placing.placed)
                Qt.callLater(placing.rescreen);
        }
    }
    // A screen came or went: the keyboard is placed again on the screens left.
    Connections {
        target: placing.win.spots
        function onScreensChanged() {
            Qt.callLater(placing.rescreen);
        }
    }
    // The screen's size changed at the same scale, as with a new resolution.
    Connections {
        target: placing.display
        function onWidthChanged() {
            Qt.callLater(placing.rescreen);
        }
        function onHeightChanged() {
            Qt.callLater(placing.rescreen);
        }
    }
    // A move is placed where it ended; a placement owed during a resize comes now.
    Connections {
        target: placing.win.mover
        function onEnded(wasMove) {
            if (wasMove)
                placing.place();
            else if (placing.owed)
                placing.rescreen();
        }
    }
}

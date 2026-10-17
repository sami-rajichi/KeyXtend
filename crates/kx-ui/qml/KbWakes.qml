// The keyboard window's answers to the Ui object's wakes: it reads again what changed.
import QtQuick

Connections {
    // The keyboard window, which reads again.
    required property Window win

    target: win.ui
    function onSettingsChanged() {
        win.reread();
    }
    function onLookChanged() {
        win.relook();
    }
    function onTextsChanged() {
        win.retext();
    }
    function onFontsChanged() {
        win.faces++;
    }
    // A mouse mode or a used latch changed the keys on the bus: their lights show now, not at the next relabel.
    function onKeysChanged() {
        win.restate();
    }
    // The scroll module started or stopped, which starts or ends the entry watch; read on the loop's next pass.
    function onScrollChanged() {
        Qt.callLater(win.rescroll);
    }
}

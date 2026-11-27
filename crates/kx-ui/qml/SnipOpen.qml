// Times one Snip window's open for the debug log: when it showed, first drew and its pop ended, which Snip's windows
// log in ms from the view that brought its stage, with the click before that view.
import QtQuick

QtObject {
    id: open

    // Snip's windows, the shadow of the window it times, whether that window drew with its pop ended, and its stage.
    required property var snips
    required property Glow shadow
    required property bool ready
    required property string stage

    // When the window showed and first drew this time, on the wall clock in ms; 0 before.
    property real shown: 0
    property real drawn: 0

    property Connections watch: Connections {
        target: open.shadow
        function onVisibleChanged() {
            open.shown = open.shadow.visible ? Date.now() : 0;
            open.drawn = 0;
        }
        function onDrawnChanged() {
            if (open.shadow.drawn && open.shown)
                open.drawn = Date.now();
        }
    }

    onReadyChanged: {
        if (!ready || !shown)
            return;
        const now = Date.now();
        snips.timed(stage, shown, drawn || now, now);
        shown = 0;
    }
}

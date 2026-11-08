// The selection strip's side of the stand-in Ui object, which FakeUi builds on: the strip as the hub writes it, where
// the hub places it, and every measure, click and fade QML sends.
import QtQuick

QtObject {
    // The strip as the hub writes it, null while none shows; setting it sends `selChanged`.
    property var selView: null
    // Where `selAt` puts the strip's top-left, in physical pixels, or null for nowhere.
    property var selSpot: ({ x: 300, y: 400 })
    // Each `selAt` ask as `{round, w, h, size, clear}`, and each click and fade as `selAct round chip` and
    // `selFaded round`.
    property var selAsks: []
    property var selCalls: []
    // The round whose fade ended, which the hub then forgets without a wake, and how often QML read the strip.
    property int selGone: -1
    property int selReads: 0

    // A new strip or none: read `sel` again.
    signal selChanged

    onSelViewChanged: selChanged()

    function sel() {
        selReads++;
        const v = selView && selView.round !== selGone ? selView : null;
        return JSON.stringify(v);
    }
    function selAt(round, w, h, size, clear) {
        selAsks = selAsks.concat([{ round: round, w: w, h: h, size: size, clear: JSON.parse(clear) }]);
        return JSON.stringify(selSpot);
    }
    function selAct(round, chip) {
        selCalls = selCalls.concat(["selAct " + round + " " + chip]);
    }
    function selFaded(round) {
        selCalls = selCalls.concat(["selFaded " + round]);
        if (selView && selView.round === round && selView.acted !== null)
            selGone = round;
    }
}

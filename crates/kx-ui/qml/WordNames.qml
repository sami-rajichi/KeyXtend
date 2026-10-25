// The names the chip lane's JSON, `Ui.wordsAsk` and the gates use, held once in QML: tst_words checks the asks against
// the hub's own (`kx_ui::asks`) and draws the lanes the hub writes.
import QtQuick

QtObject {
    // Each `Ui.wordsAsk` kind.
    readonly property var asks: ({
            forget: "forget",
            cancel: "cancel",
            pause: "pause",
            power: "power",
            undo: "undo",
            more: "more",
            page: "page",
            pick: "pick",
            again: "again"
        })
    // The lane's kinds the top bar tells apart: the password note, and a lane with nothing to offer.
    readonly property string locked: "locked"
    readonly property string empty: "empty"
    // The chip kind that opens the email list.
    readonly property string more: "more"
    // The gate-id group of the lane, its menu and the email list.
    readonly property string group: "words"
}

// The names the chip lane's JSON, `Ui.wordsAsk` and the gate ids use, held once in QML: tst_words checks the asks
// against the hub's own (`kx_ui::asks`), and kx-gates' P7 tests check the ids its config names.
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
    // The lane's kinds the top bar tells apart: the password note, a lane with nothing to offer, and no module running.
    readonly property string locked: "locked"
    readonly property string empty: "empty"
    readonly property string stopped: "stopped"
    // The chip kind that opens the email list.
    readonly property string more: "more"
    // The gate-id group of the lane, its menu and the email list.
    readonly property string group: "words"
    // Each part's gate id in `group`; a chip's, a menu item's and a list row's come before their place or name.
    readonly property var ids: ({
            lane: "lane",
            chip: "chip-",
            cancel: "cancel",
            note: "locked",
            menu: "menu",
            item: "menu-",
            list: "list",
            row: "list-row-",
            back: "list-back",
            on: "list-on",
            close: "list-cancel"
        })
}

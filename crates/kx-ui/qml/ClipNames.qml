// The names the clipboard panel's clicks and gate ids use, held once in QML: tst_clip_panel checks the clicks against
// the hub's own (`kx_ui::clip_acts`), and gates find each part by `group` and its id.
import QtQuick

QtObject {
    // Each `Ui.clipAct` kind.
    readonly property var acts: ({
            tick: "tick",
            filter: "filter",
            page: "page",
            insert: "insert",
            pin: "pin",
            delete: "delete",
            deleteAll: "delete-all",
            yes: "yes",
            no: "no",
            preview: "preview",
            back: "back",
            close: "close"
        })
    // The gate-id group of the panel.
    readonly property string group: "clip"
    // Each part's gate id in `group`; a row's and a filter's come before their place or filter id.
    readonly property var ids: ({
            list: "list",
            row: "row-",
            close: "close",
            deleteAll: "delete-all",
            search: "search",
            filter: "filter-",
            pageBack: "page-back",
            pageOn: "page-on",
            preview: "preview",
            pin: "pin",
            delete: "delete",
            insert: "insert",
            yes: "yes",
            no: "no",
            folder: "open-folder",
            back: "back",
            textBack: "text-back",
            textOn: "text-on"
        })
}

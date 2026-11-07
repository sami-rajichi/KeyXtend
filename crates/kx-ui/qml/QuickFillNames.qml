// The names Quick-fill's clicks, panel buttons and gate ids use, held once in QML: tst_quickfill_list checks the clicks
// against the hub's (`kx_ui::fill_acts`) and tst_quickfill_panel the buttons against the view's; gates find a part by
// `group` and id.
import QtQuick

QtObject {
    // Each `QuickFill.act` kind.
    readonly property var acts: ({
            row: "row",
            field: "field",
            filter: "filter",
            kind: "kind",
            page: "page",
            button: "button",
            answer: "answer",
            checks: "checks",
            caret: "caret",
            wheel: "wheel"
        })
    // The panel buttons QML sends by id, each a key of the view's `buttons`.
    readonly property var buttons: ({
            new: "new",
            backup: "backup",
            close: "close",
            back: "back",
            edit: "edit",
            name: "name",
            addField: "add-field",
            ownLabel: "own-label",
            save: "save",
            cancel: "cancel",
            delete: "delete",
            backUpNow: "back-up-now",
            addPlace: "add-place",
            changePassword: "change-password",
            bringBack: "bring-back",
            keepass: "import",
            otherFolder: "other-folder",
            here: "here",
            done: "done"
        })
    // The field lines a click aims the letters at, as their field button ids.
    readonly property var lines: ({
            value: "value",
            label: "label"
        })
    // Each page's sheet, by the `is` the view gives it.
    readonly property var pages: ({
            list: "list",
            labels: "labels",
            values: "values",
            entry: "entry",
            edit: "edit",
            add: "add-field",
            backup: "backup",
            places: "places",
            folders: "folders",
            password: "password",
            found: "found",
            foreign: "foreign"
        })
    // What the folder list picks, by the `pick` the view gives it, and the password page's ask that names no backup.
    readonly property var picks: ({
            folder: "folder",
            keepass: "keepass"
        })
    readonly property var asks: ({
            keepass: "keepass"
        })
    // Each hub id's icon role in icons.toml `[fill]`: the entry kinds, the places, the row buttons, and the field buttons
    // off and on.
    readonly property var kindIcons: ({
            person: "kind_person",
            login: "kind_login",
            wifi: "kind_wifi",
            card: "kind_card",
            note: "kind_note",
            other: "kind_other"
        })
    readonly property var placeIcons: ({
            cloud: "place_cloud",
            removable: "place_removable",
            drive: "place_drive",
            folder: "place_folder",
            file: "place_file"
        })
    readonly property var rowIcons: ({
            user: "user",
            password: "password",
            both: "both",
            insert: "insert",
            open: "open",
            remove: "remove"
        })
    readonly property var fieldIcons: ({
            insert: ["insert"],
            [field.show]: ["show", "hide"],
            [field.hidden]: ["hidden_off", "hidden_on"],
            paste: ["paste"],
            make: ["make"]
        })
    // The field buttons that read as on: Show while the value shows, and the hidden switch while it is on.
    readonly property var field: ({
            show: "show",
            hidden: "hidden"
        })
    // The icon of entry kind `kind` among the fill icons `ic`, Other's for a kind it does not know.
    function kindIcon(ic, kind) {
        return ic[kindIcons[kind] || kindIcons.other];
    }
    // The icon of place kind `kind` among the fill icons `ic`, a folder's for a kind it does not know.
    function placeIcon(ic, kind) {
        return ic[placeIcons[kind] || placeIcons.folder];
    }
    // Whether place kind `kind` is a file, which the folder list picks rather than opens.
    function isFile(kind) {
        return placeIcons[kind] === placeIcons.file;
    }
    // The icon of row button `id` among the fill icons `ic`, or none.
    function rowIcon(ic, id) {
        return rowIcons[id] ? ic[rowIcons[id]] : "";
    }
    // The icon of field button `id` among the fill icons `ic`, as it looks while `on`, or none.
    function fieldIcon(ic, id, on) {
        const roles = fieldIcons[id];
        return roles ? ic[roles[on ? roles.length - 1 : 0]] : "";
    }
    // The gate-id group of the panel; a panel button's id is its own.
    readonly property string group: "fill"
    // Each other part's gate id in `group`; a row's, a field's, a filter's and a kind's come before their place or id.
    readonly property var ids: ({
            list: "list",
            row: "row-",
            field: "field-",
            filter: "filter-",
            kind: "kind-",
            search: "search",
            checks: "checks",
            password: "password",
            pageBack: "page-back",
            pageOn: "page-on",
            invite: "invite",
            yes: "yes",
            no: "no",
            part: "-"
        })
}

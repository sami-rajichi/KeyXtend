// A Quick-fill page in the panel's frame: what every page reads, and the view it draws. A view for another page reaches
// this one before the panel swaps it out, so it keeps its last view and its bindings never read another shape.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    // The panel's window, whose sizes and clicks the page takes, and the sheets it draws, by their `is`.
    required property var box
    required property var draws

    // A plain store, so keeping a view wakes nothing.
    readonly property var kept: ({})
    readonly property var view: {
        const v = box.view;
        if (draws.indexOf(v.sheet.is) >= 0)
            kept.view = v;
        return kept.view;
    }
    readonly property var sheet: view.sheet
    readonly property var names: box.names
    readonly property var ids: names.buttons
    readonly property var lk: box.lk
    readonly property var res: box.res
    readonly property real s: box.s
    readonly property var ic: lk.icons.fill
    readonly property real side: box.side
}

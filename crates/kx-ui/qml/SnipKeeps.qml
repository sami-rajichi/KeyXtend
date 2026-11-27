// Copy and Save at the end of the editor's first row (snip-editor.js acts): each shows its tick and done word once it
// acted, and Save says why it waits.
pragma ComponentBehavior: Bound
import QtQuick

Row {
    id: keeps

    // The editor.
    required property var editor

    readonly property var lk: editor.lk
    readonly property var res: editor.res
    readonly property var n: editor.n
    readonly property var ed: editor.ed

    spacing: lk.shape.clip.button_px[5]
    layoutDirection: Qt.LeftToRight

    Repeater {
        model: keeps.n.ids.keeps
        delegate: SnipButton {
            required property string modelData
            readonly property bool done: !!keeps.ed && keeps.ed.done === modelData
            readonly property bool saves: modelData === keeps.n.keep.save
            lk: keeps.lk
            res: keeps.res
            name: keeps.res.tr((done ? "snip.done-" : "snip.strip-") + modelData)
            icon: done ? keeps.lk.icons.snip.tick : keeps.lk.icons.snip.strip[modelData]
            worded: true
            tone: saves ? "enter" : "ghost"
            why: saves && keeps.ed && keeps.ed.save_dim ? keeps.res.tr("snip.dim-" + keeps.ed.save_dim) : ""
            uid: "snip.keep." + modelData
            tip: keeps.editor.hint
            onClicked: keeps.editor.keep(modelData)
        }
    }
}

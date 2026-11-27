// One ink of the editor's options row (snip-editor.js .ink-b): a round swatch edged so white shows on the panel, inside
// the theme's ring once picked, named in its tip; a click at let-go picks it.
import QtQuick

Item {
    id: swatch

    // The editor, the ink's id and whether it is picked.
    required property var editor
    required property string ink
    required property bool on

    readonly property var lk: editor.lk
    readonly property var sh: lk.shape.snip
    // Its size across, its edge and how far apart inks sit (`ink_px`).
    readonly property var px: sh.ink_px
    readonly property string name: editor.res.tr("snip.ink-" + ink)
    readonly property string uid: "snip.ink." + ink

    width: px[0]
    height: px[0]
    Accessible.role: Accessible.RadioButton
    Accessible.name: name
    Accessible.id: uid
    Accessible.checkable: true
    Accessible.checked: on
    Accessible.onPressAction: swatch.pick()

    function pick() {
        editor.snips.act(editor.n.acts.ink, ink);
    }

    Rectangle {
        anchors.fill: parent
        visible: swatch.on
        radius: width / 2
        color: "transparent"
        border.width: swatch.sh.ring_px
        border.color: swatch.lk.palette.ring
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: swatch.on ? swatch.sh.ring_px + swatch.px[1] : 0
        radius: width / 2
        color: swatch.lk.common[swatch.editor.n.ink + swatch.ink]
        border.width: swatch.px[1]
        border.color: swatch.lk.palette.pop_muted
    }
    Hover {
        lk: swatch.lk
        radius: width / 2
        visible: area.containsMouse
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onActed: swatch.pick()
        onContainsMouseChanged: swatch.editor.hint(containsMouse ? swatch.name : "", swatch)
    }
}

// A list panel's tile (mock-up .th2), for a kind, a label, a place or a row's question: its icon on pop_hover, in the
// panel's ink so it reads at 4.5:1, and in Highlight text in high contrast, where the tile is Highlight.
import QtQuick

Rectangle {
    id: tile

    // The icon, the look, icons and size.
    required property string icon
    required property var lk
    required property var res
    required property real s

    readonly property var sh: lk.shape.panel.tile_px

    width: sh[0] * s
    height: sh[1] * s
    radius: sh[2] * s
    color: lk.palette.pop_hover

    Icon {
        anchors.centerIn: parent
        icon: tile.icon
        base: tile.res.icons
        ext: tile.res.ext
        px: tile.sh[3] * tile.s
        tint: tile.lk.sel_ink
    }
}

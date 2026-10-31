// A clipboard row's thumbnail tile (mock-up .th2): the picture the provider serves, inset in the tile, or the kind's
// icon when the row has none, the picture has not come, or the provider gave its one clear pixel for an old round.
import QtQuick

Rectangle {
    id: tile

    // The picture's URL (null for none), the kind's id (`text`, `link`, `image`, `file`, `media`), whether the row is lit
    // (under the pointer or ticked), the look, icons and size.
    required property var url
    required property string kind
    required property bool lit
    required property var lk
    required property var res
    required property real s

    readonly property var sh: lk.shape.clip.thumb_px

    width: sh[0] * s
    height: sh[1] * s
    radius: sh[2] * s
    // pop_hover over the row's own colour; high contrast makes it Highlight, so the icon takes sel_ink there.
    color: lk.palette.pop_hover

    Item {
        anchors.fill: parent
        anchors.margins: tile.lk.shape.clip.thumb_inset_px * tile.s
        clip: true
        // Never cached, so an old round's texture goes as the next view comes.
        Image {
            id: picture
            anchors.fill: parent
            visible: status === Image.Ready && implicitWidth > 1
            source: tile.url || ""
            cache: false
            fillMode: Image.PreserveAspectCrop
            Accessible.ignored: true
        }
    }
    Icon {
        anchors.centerIn: parent
        visible: !picture.visible
        icon: tile.lk.icons.clip["kind_" + tile.kind] || tile.lk.icons.clip.kind_text
        base: tile.res.icons
        ext: tile.res.ext
        px: tile.sh[3] * tile.s
        tint: tile.lit || tile.lk.contrast ? tile.lk.sel_ink : tile.lk.palette.pop_muted
    }
}

// The dim over a frozen screen (design/snip-parts.css .dim, .hole, .tag, .hover): all over, or round a clear hole with
// its edge and its size in physical pixels above it, or all over with a ring round the window or screen a click takes.
import QtQuick

Item {
    id: shade

    // The look, texts and tag colour; the hole in Qt units and its physical box (`{w, h}`) for the tag, or null; the ring,
    // or null.
    required property var lk
    required property var res
    required property color tag
    required property var hole
    required property var size
    required property var ring
    // A frame with handles lies on the hole, so the hole's own edge stays hidden.
    property bool framed: false

    readonly property var sh: lk.shape.snip
    readonly property var c: lk.common
    // The clear box, or an empty one at the top-left, which leaves the dim whole.
    readonly property rect h: hole ? Qt.rect(hole.x, hole.y, hole.width, hole.height) : Qt.rect(0, 0, 0, 0)

    // Four bands round the hole, which together dim the whole screen when there is none.
    Rectangle {
        x: 0
        y: 0
        width: shade.width
        height: shade.h.y
        color: shade.c.snip_dim
    }
    Rectangle {
        x: 0
        y: shade.h.y + shade.h.height
        width: shade.width
        height: shade.height - y
        color: shade.c.snip_dim
    }
    Rectangle {
        x: 0
        y: shade.h.y
        width: shade.h.x
        height: shade.h.height
        color: shade.c.snip_dim
    }
    Rectangle {
        x: shade.h.x + shade.h.width
        y: shade.h.y
        width: shade.width - x
        height: shade.h.height
        color: shade.c.snip_dim
    }
    // The hole's edge, just outside it (.hole box-shadow 0 0 0 1.5px), unless a frame draws its own.
    Rectangle {
        objectName: "edge"
        visible: !!shade.hole && !shade.framed
        x: shade.h.x - border.width
        y: shade.h.y - border.width
        width: shade.h.width + 2 * border.width
        height: shade.h.height + 2 * border.width
        color: "transparent"
        border.width: shade.sh.hole_px
        border.color: shade.c.snip_edge
    }
    // The size in physical pixels above the hole, or inside its top-left when no room is above (.tag).
    Rectangle {
        id: chip
        objectName: "tag"
        visible: !!shade.hole && !!shade.size
        readonly property real room: shade.h.y - shade.sh.tag_gap_px
        x: room >= 0 ? shade.h.x : shade.h.x + shade.sh.tag_pad_px[1]
        y: room >= 0 ? room : shade.h.y + shade.sh.tag_pad_px[0]
        width: said.implicitWidth + 2 * shade.sh.tag_pad_px[1]
        height: said.implicitHeight + 2 * shade.sh.tag_pad_px[0]
        radius: shade.sh.tag_radius_px
        color: shade.tag
        Label {
            id: said
            anchors.centerIn: parent
            text: shade.size ? shade.res.tr("snip.size").arg(shade.size.w).arg(shade.size.h) : ""
            font.family: shade.res.fam.latin
            font.pixelSize: shade.sh.tag_px
            font.weight: shade.sh.tag_weight
            color: shade.c.snip_tag_ink
        }
    }
    // The ring round the window or screen under the pointer (.hover).
    Rectangle {
        objectName: "ring"
        visible: !!shade.ring
        x: shade.ring ? shade.ring.x : 0
        y: shade.ring ? shade.ring.y : 0
        width: shade.ring ? shade.ring.width : 0
        height: shade.ring ? shade.ring.height : 0
        radius: shade.sh.hover_radius_px
        color: "transparent"
        border.width: shade.sh.hover_px
        border.color: shade.lk.palette.ring
    }
}

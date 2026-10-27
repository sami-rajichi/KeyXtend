// A paged list's button (mock-up .pg): its icon in the selection's text colour, dimmed when it cannot be used. It
// acts at let-go (ADR-0024), and readers and gates find it by its name and id.
import QtQuick

Rectangle {
    id: pb

    // Its icon and name, whether it can be used, the look, icons and size, and its gate id.
    required property string icon
    required property string name
    required property bool can
    required property var lk
    required property var res
    required property real s
    property string uid: ""

    signal clicked

    readonly property var pn: lk.shape.panel

    width: pn.page_px[0] * s
    height: pn.page_px[1] * s
    radius: pn.page_px[2] * s
    color: lk.palette.pop_hover
    border.width: lk.shape.line.px
    border.color: lk.palette.pop_line
    enabled: can
    opacity: can ? 1 : pn.off_share
    Accessible.role: Accessible.Button
    Accessible.name: name
    Accessible.id: uid
    Accessible.onPressAction: if (can) pb.clicked()

    Hover {
        lk: pb.lk
        radius: pb.radius
        tint: pb.lk.palette.pop_sel
        visible: area.containsMouse
    }
    Icon {
        anchors.centerIn: parent
        icon: pb.icon
        base: pb.res.icons
        ext: pb.res.ext
        px: pb.pn.page_px[3] * pb.s
        tint: pb.lk.sel_ink
    }
    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onActed: if (pb.can) pb.clicked()
    }
}

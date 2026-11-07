// The paged folder list (design "The backup box"): what it picks, the folder shown or This PC, then the drives, folders
// and files by their kind's icon. A drive or a folder opens and a file is chosen; Use this folder shows only while
// picking a folder, and waits at the drives.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.folders]
    // The icon that heads the list.
    property string icon: sheet.pick === names.picks.folder ? ic.place_folder : ic.bring_back

    readonly property real gap: lk.shape.clip.foot_gap_px * s

    QuickFillTop {
        id: top
        box: page.box
        icon: page.icon
        title: page.box.worded(page.sheet.text)
        starts: [page.box.button(page.ids.back, page.ic.back)]
        ends: page.sheet.pick === page.names.picks.folder ? [page.box.button(page.ids.here, page.ic.here, "ghost", !page.sheet.drives)] : []
    }
    // The folder shown, between the buttons: a name reads in its own direction and font, and a long one ends in an
    // ellipsis.
    Label {
        anchors.left: parent.left
        anchors.leftMargin: page.side + top.lead.width + page.gap
        anchors.right: parent.right
        anchors.rightMargin: page.side + top.trail.width + page.gap
        y: page.box.bandY
        height: page.lk.shape.clip.button_px[0] * page.s
        verticalAlignment: Text.AlignVCenter
        text: page.sheet.folder.text
        elide: Text.ElideRight
        horizontalAlignment: page.sheet.folder.rtl ? Text.AlignRight : Text.AlignLeft
        font.family: page.sheet.folder.arabic ? page.res.fam.arabic : page.res.fam.latin
        font.pixelSize: page.box.pn.hint_px[0] * page.s
        color: page.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    QuickFillRows {
        box: page.box
        rows: page.sheet.rows.map(r => page.box.placeRow(r, null, { icon: page.names.isFile(r.kind) ? page.ic.pick : page.ic.open }))
        name: page.sheet.folder.text
    }
}

// Quick-fill's footer (design "The panel"; mock-up .lfoot): the note, or the backup's waiting line, then the pager laid
// flat, Back, "n of m" and Next, shown only when there is more than one page; never a scroll bar (ADR-0008). The note
// is an alert readers hear, and it takes the waiting line's place while it shows.
import QtQuick

Item {
    id: foot

    // The panel's view, whether a question waits, which stops the pager, the click names, the app's Ui object for the page
    // number, the look, fonts, icons and size.
    required property var view
    required property bool asking
    required property var names
    required property var ui
    required property var lk
    required property var res
    required property real s

    // Back (by -1) or Next (by 1) was clicked.
    signal turn(int by)

    readonly property var pn: lk.shape.panel
    readonly property var pg: view.page
    readonly property real pad: pn.foot_pad_px[1] * s
    readonly property real gap: lk.shape.clip.foot_gap_px * s
    readonly property bool paged: pg.count > 1
    // The room the note or the waiting line takes beside the pager.
    readonly property real room: width - 2 * pad - (paged ? pager.width + gap : 0)

    // The note's band keeps its room, shown or not, so the panel keeps its size as notes come and go.
    height: Math.max(pn.page_px[1] * s, note.height) + 2 * pn.foot_pad_px[0] * s

    Rectangle {
        width: parent.width
        height: foot.lk.shape.line.px
        color: foot.lk.palette.pop_line
    }
    ClipNote {
        id: note
        x: foot.pad
        anchors.verticalCenter: parent.verticalCenter
        width: foot.room
        text: foot.view.note
        lk: foot.lk
        res: foot.res
        s: foot.s
    }
    // Where the backup waits, as a muted line with the backup's icon.
    Row {
        visible: !foot.view.note && !!foot.view.waiting
        x: foot.pad
        anchors.verticalCenter: parent.verticalCenter
        spacing: foot.gap
        Icon {
            id: mark
            anchors.verticalCenter: parent.verticalCenter
            icon: foot.lk.icons.fill.waiting
            base: foot.res.icons
            ext: foot.res.ext
            px: foot.lk.shape.panel.line_px[0] * foot.s
            tint: foot.lk.palette.pop_muted
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            width: foot.room - mark.width - foot.gap
            text: foot.view.waiting ? foot.view.waiting.text : ""
            elide: Text.ElideRight
            font.family: foot.res.fam.ui
            font.pixelSize: foot.lk.shape.panel.hint_px[0] * foot.s
            color: foot.lk.palette.pop_muted
            Accessible.ignored: !parent.visible
            Accessible.role: Accessible.StaticText
            Accessible.name: text
        }
    }
    Row {
        id: pager
        visible: foot.paged
        anchors.right: parent.right
        anchors.rightMargin: foot.pad
        anchors.verticalCenter: parent.verticalCenter
        spacing: foot.pn.pager_gap_px * foot.s
        PageButton {
            icon: foot.lk.icons.page.back
            name: foot.res.tr("page.back")
            can: !foot.asking && foot.pg.at > 0
            lk: foot.lk
            res: foot.res
            s: foot.s
            uid: foot.res.uid(foot.names.group, foot.names.ids.pageBack)
            onClicked: foot.turn(-1)
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            // A torn-down panel can lose its Ui object first.
            text: foot.paged && foot.ui ? foot.ui.pageText(foot.pg.at + 1, foot.pg.count) : ""
            font.family: foot.res.fam.ui
            font.pixelSize: foot.pn.count_px * foot.s
            font.weight: foot.pn.count_weight
            color: foot.lk.palette.pop_muted
            Accessible.ignored: !foot.paged
            Accessible.role: Accessible.StaticText
            Accessible.name: text
        }
        PageButton {
            icon: foot.lk.icons.page.on
            name: foot.res.tr("page.on")
            can: !foot.asking && foot.pg.at < foot.pg.count - 1
            lk: foot.lk
            res: foot.res
            s: foot.s
            uid: foot.res.uid(foot.names.group, foot.names.ids.pageOn)
            onClicked: foot.turn(1)
        }
    }
}

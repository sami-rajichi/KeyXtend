// A list's pager (mock-up .pg): Back, the page number and Next in a column, shown only when there is more than one
// page, never a scroll bar (ADR-0008).
import QtQuick

Column {
    id: pager

    // The page shown (from 0) and how many; whether Back and Next can act; the app's Ui object for the page number;
    // the look, fonts, icons and texts, and size.
    required property int page
    required property int pages
    required property bool canBack
    required property bool canOn
    required property var ui
    required property var lk
    required property var res
    required property real s
    // The buttons' UI Automation ids, when gates know them.
    property string backId: ""
    property string onId: ""
    // The page count is final; until then no number shows, so a reader never hears a total that still grows.
    property bool counted: true

    // Back (by -1) or Next (by 1) was clicked.
    signal turn(int by)

    readonly property var pn: lk.shape.panel

    visible: pages > 1
    spacing: pn.pager_gap_px * s

    PageButton {
        icon: pager.lk.icons.page.back
        name: pager.res.tr("page.back")
        can: pager.canBack
        lk: pager.lk
        res: pager.res
        s: pager.s
        uid: pager.backId
        onClicked: pager.turn(-1)
    }
    Label {
        width: pager.pn.page_px[0] * pager.s
        text: pager.pages > 1 && pager.counted && pager.ui ? pager.ui.pageText(pager.page + 1, pager.pages) : ""
        font.family: pager.res.fam.ui
        font.pixelSize: pager.pn.count_px * pager.s
        font.weight: pager.pn.count_weight
        color: pager.lk.palette.pop_muted
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.Wrap
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    PageButton {
        icon: pager.lk.icons.page.on
        name: pager.res.tr("page.on")
        can: pager.canOn
        lk: pager.lk
        res: pager.res
        s: pager.s
        uid: pager.onId
        onClicked: pager.turn(1)
    }
}

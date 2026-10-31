// The clipboard panel's list (spec §7; mock-up .list and .lrow): five row slots on every page, a short page padded so
// nothing jumps, the highlight ring gliding to the highlighted row, and the pager beside them, never a scroll bar
// (ADR-0008). An empty history shows one line with its icon and no rows.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: list

    // The panel's view, rows per page and a row's height, the click names, the app's Ui object for the page number, the
    // look, fonts, icons and size.
    required property var view
    required property int rows
    required property real rowH
    required property var names
    required property var ui
    required property var lk
    required property var res
    required property real s

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    // The highlight ring, for the tests.
    readonly property alias ring: ring
    readonly property var pn: lk.shape.panel
    readonly property int per: Math.max(1, rows)
    readonly property real gap: pn.row_px[1] * s
    readonly property real side: pn.body_pad_px[1] * s
    readonly property var hi: view.highlight
    // Each slot's row, null past the page's end, with the page it shows on, so a row sees a turn and a swap together.
    readonly property var shown: [...Array(per).keys()].map(i => ({ r: view.rows[i] || null, at: view.page.at }))
    // The highlighted row's slot, whose mark colours the ring.
    property Item lit: null

    function relit() {
        lit = hi === null || hi === undefined ? null : rep.itemAt(hi);
    }

    height: (pn.body_pad_px[0] + pn.body_pad_px[2]) * s + per * rowH + (per - 1) * gap
    onHiChanged: relit()

    Item {
        x: list.side
        y: list.pn.body_pad_px[0] * list.s
        // The pager's room stays, so rows keep their width as views change.
        width: list.width - 2 * list.side - (list.pn.body_gap_px + list.pn.page_px[0]) * list.s
        height: list.per * list.rowH + (list.per - 1) * list.gap

        Column {
            id: col
            width: parent.width
            spacing: list.gap
            Accessible.role: Accessible.List
            Accessible.name: list.view.title
            Accessible.id: list.res.uid(list.names.group, list.names.ids.list)
            Repeater {
                id: rep
                model: list.view.empty ? 0 : list.per
                onItemAdded: list.relit()
                onItemRemoved: list.relit()
                delegate: ClipRow {
                    required property int index
                    width: col.width
                    height: list.rowH
                    slot: list.shown[index]
                    place: index
                    hi: index === list.hi
                    expires: list.view.words.expires
                    names: list.names
                    lk: list.lk
                    res: list.res
                    s: list.s
                    onTicked: list.act(list.names.acts.tick, index)
                }
            }
        }
        // One ring over the rows, so a new highlight glides to its row (motion.toml [clip] highlight).
        Rectangle {
            id: ring
            visible: !!list.lit && list.lit.filled
            y: (list.hi || 0) * (list.rowH + list.gap)
            width: col.width
            height: list.rowH
            radius: list.pn.row_px[3] * list.s
            color: "transparent"
            border.width: list.lk.shape.press.ring_px * list.s
            border.color: list.lit ? list.lit.mark : list.lk.palette.ring
            Behavior on y {
                Tween {
                    move: list.lk.motion.clip.highlight
                }
            }
        }
    }
    Pager {
        anchors.right: parent.right
        anchors.rightMargin: list.side
        anchors.verticalCenter: parent.verticalCenter
        page: list.view.page.at
        pages: list.view.page.count
        canBack: list.view.page.at > 0
        canOn: list.view.page.at < list.view.page.count - 1
        ui: list.ui
        lk: list.lk
        res: list.res
        s: list.s
        backId: list.res.uid(list.names.group, list.names.ids.pageBack)
        onId: list.res.uid(list.names.group, list.names.ids.pageOn)
        onTurn: by => list.act(list.names.acts.page, by)
    }
    Column {
        visible: !!list.view.empty
        anchors.centerIn: parent
        spacing: list.pn.body_gap_px * list.s
        Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            icon: list.lk.icons.clip.title
            base: list.res.icons
            ext: list.res.ext
            px: list.lk.shape.clip.empty_px * list.s
            tint: list.lk.palette.pop_muted
        }
        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            width: Math.min(implicitWidth, list.width - 2 * list.side)
            text: list.view.empty || ""
            wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter
            font.family: list.res.fam.ui
            font.pixelSize: list.pn.title_px * list.s
            color: list.lk.palette.pop_muted
            Accessible.ignored: false
            Accessible.role: Accessible.StaticText
            Accessible.name: text
        }
    }
}

// The panel's paged list (mock-up .lbody): a page of rows and a pager beside it, never a scroll bar (ADR-0008).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: list

    // The look and icons, the text font, the keyboard size, the panel's fixed data, and this page: rows, at, count, label.
    required property var lk
    required property var res
    required property string fam
    required property real s
    required property var fx
    required property var pg
    // The picked row, counted over all pages, or -1.
    property int picked: -1

    // Asks for page `at`.
    signal turn(int at)
    // The pointer came onto a page button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var sh: lk.shape.panel
    readonly property var pal: lk.palette
    readonly property real rowH: sh.row_px[0] * s
    readonly property real gap: sh.row_px[1] * s
    readonly property real side: sh.body_pad_px[1] * s

    implicitHeight: (sh.body_pad_px[0] + sh.body_pad_px[2]) * s + fx.rows * rowH + (fx.rows - 1) * gap

    Column {
        id: rows
        // Anchors, not x, so a right-to-left panel mirrors the list and the pager.
        anchors.left: parent.left
        anchors.leftMargin: list.side
        anchors.top: parent.top
        anchors.topMargin: list.sh.body_pad_px[0] * list.s
        width: list.width - 2 * list.side - (list.sh.body_gap_px + list.sh.page_px[0]) * list.s
        spacing: list.gap
        Accessible.role: Accessible.List
        Accessible.name: list.fx.title

        Repeater {
            model: list.pg.rows
            delegate: Rectangle {
                id: row
                required property var modelData
                required property int index
                // Padding rows on the last page are blank, take no clicks and are skipped by readers.
                readonly property bool real: modelData !== null
                readonly property int item: list.pg.at * list.fx.rows + index
                readonly property bool sel: real && list.picked === item
                readonly property bool lit: sel || (area.containsMouse && real)

                width: rows.width
                height: list.rowH
                radius: list.sh.row_px[3] * list.s
                color: sel ? list.pal.pop_sel : (area.containsMouse && real ? list.pal.pop_hover : "transparent")
                Accessible.role: Accessible.ListItem
                Accessible.name: real ? modelData : ""
                Accessible.ignored: !real
                Accessible.selectable: true
                Accessible.selected: sel
                Accessible.onPressAction: if (real) list.picked = item

                Text {
                    anchors.fill: parent
                    anchors.leftMargin: list.sh.row_px[2] * list.s
                    anchors.rightMargin: anchors.leftMargin
                    text: row.real ? row.modelData : ""
                    textFormat: Text.PlainText
                    font.family: list.fam
                    font.pixelSize: list.sh.row_text_px[list.fx.rtl ? 1 : 0] * list.s
                    color: row.lit ? list.lk.sel_ink : list.pal.pop_ink
                    verticalAlignment: Text.AlignVCenter
                    elide: Text.ElideRight
                    // The row already carries the name; hide the text so readers say it once.
                    Accessible.ignored: true
                }
                MouseArea {
                    id: area
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: if (row.real) list.picked = row.item
                }
            }
        }
    }
    Column {
        id: pager
        readonly property var px: list.sh.page_px
        anchors.right: parent.right
        anchors.rightMargin: list.side
        anchors.verticalCenter: rows.verticalCenter
        spacing: list.sh.pager_gap_px * list.s

        PopButton {
            width: pager.px[0] * list.s
            height: pager.px[1] * list.s
            radius: pager.px[2] * list.s
            iconPx: pager.px[3] * list.s
            framed: true
            lk: list.lk
            res: list.res
            button: list.fx.prev
            enabled: list.pg.at > 0
            onClicked: list.turn(list.pg.at - 1)
            onTip: (name, at) => list.tip(name, at)
        }
        Text {
            width: pager.px[0] * list.s
            text: list.pg.label
            textFormat: Text.PlainText
            font.family: list.fam
            font.pixelSize: list.sh.count_px * list.s
            font.weight: list.sh.count_weight
            color: list.pal.pop_muted
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.Wrap
            Accessible.role: Accessible.StaticText
            Accessible.name: text
        }
        PopButton {
            width: pager.px[0] * list.s
            height: pager.px[1] * list.s
            radius: pager.px[2] * list.s
            iconPx: pager.px[3] * list.s
            framed: true
            lk: list.lk
            res: list.res
            button: list.fx.next
            enabled: list.pg.at < list.pg.count - 1
            onClicked: list.turn(list.pg.at + 1)
            onTip: (name, at) => list.tip(name, at)
        }
    }
}

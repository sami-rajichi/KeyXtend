// The List design's language list (mock-up .langs and .lg-it): one page of layouts with the highlighted row ringed and
// the current one ticked, and a pager beside it when there is more than one page, never a scroll bar (ADR-0008).
// The board's arrows, Enter and Esc drive it through the keyboard; it never takes focus (ADR-0013).
pragma ComponentBehavior: Bound
import QtQuick

Window {
    id: box

    // The app's Ui object, the look, fonts, icons and texts, the keyboard size, the list's view (`{rows, hi, page_back,
    // page_on}`), and rows per page.
    required property var ui
    required property var lk
    required property var res
    required property real s
    required property var view
    required property int rows
    // The UI reads right to left, so the pager goes on the left and each row mirrors.
    required property bool rtl

    // Row `index` of the whole list was picked, or a page button sent board key `id`.
    signal pick(int index)
    signal page(string id)

    // The box whose fade, rise and scale the shadow follows.
    readonly property alias body: body
    readonly property var sh: lk.shape.list
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property real radius: lk.look.window_radius_px * pn.radius_share
    readonly property int per: Math.max(1, rows)
    readonly property int pages: Math.max(1, Math.ceil(view.rows.length / per))
    readonly property int at: Math.min(Math.floor(view.hi / per), pages - 1)
    // This page's rows; with more than one page the last is padded with nulls, so the box keeps its size.
    readonly property var shown: {
        const part = view.rows.slice(at * per, (at + 1) * per);
        const blanks = pages > 1 ? per - part.length : 0;
        return part.concat(Array(blanks).fill(null));
    }
    readonly property real pad: sh.pad_px * s
    // A row fits its badge, or its name over its layout, inside its padding.
    readonly property real rowH: Math.max(sh.badge_px[1] * s, nameLine.height + subLine.height) + 2 * sh.row_pad_px[0] * s
    readonly property real pagerW: pages > 1 ? (pn.body_gap_px + pn.page_px[0]) * s : 0

    // Shows the list, popping in (mock-up popIn).
    function open() {
        visible = true;
        popIn.restart();
    }

    FontMetrics {
        id: nameLine
        font.family: box.res.fam.latin
        font.pixelSize: box.sh.name_px * box.s
    }
    FontMetrics {
        id: subLine
        font.family: box.res.fam.latin
        font.pixelSize: box.sh.sub_px * box.s
    }

    flags: Qt.Tool | Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.WindowDoesNotAcceptFocus
    color: "transparent"
    visible: false
    // Whole units up, as a window's size is whole.
    width: Math.ceil(sh.width_px * s + pagerW)
    height: Math.ceil(Math.max(list.height, pages > 1 ? pager.height : 0) + 2 * pad)

    Rectangle {
        id: body
        width: box.width
        height: box.height
        radius: box.radius
        color: box.p.pop_bg
        // High contrast outlines it, as the P1 panel did.
        border.width: box.lk.contrast ? box.lk.shape.line.px : 0
        border.color: box.p.pop_line
        LayoutMirroring.enabled: box.rtl
        LayoutMirroring.childrenInherit: true

        Column {
            id: list
            anchors.left: parent.left
            anchors.leftMargin: box.pad
            anchors.verticalCenter: parent.verticalCenter
            width: box.sh.width_px * box.s - 2 * box.pad
            Accessible.role: Accessible.List
            Accessible.name: box.title

            Repeater {
                model: box.shown
                delegate: LangRow {
                    required property var modelData
                    required property int index
                    width: list.width
                    height: box.rowH
                    row: modelData
                    place: box.at * box.per + index
                    hi: place === box.view.hi
                    lk: box.lk
                    res: box.res
                    s: box.s
                    onPicked: box.pick(place)
                }
            }
        }
        Column {
            id: pager
            visible: box.pages > 1
            anchors.right: parent.right
            anchors.rightMargin: box.pad
            anchors.verticalCenter: parent.verticalCenter
            spacing: box.pn.pager_gap_px * box.s

            PageButton {
                icon: box.lk.icons.page.back
                name: box.res.tr("page.back")
                can: box.at > 0 && !!box.view.page_back
                onClicked: box.page(box.view.page_back)
            }
            Label {
                width: box.pn.page_px[0] * box.s
                text: box.pages > 1 && box.res ? box.ui.pageText(box.at + 1, box.pages) : ""
                font.family: box.res.fam.ui
                font.pixelSize: box.pn.count_px * box.s
                font.weight: box.pn.count_weight
                color: box.p.pop_muted
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.Wrap
                Accessible.ignored: false
                Accessible.role: Accessible.StaticText
                Accessible.name: text
            }
            PageButton {
                icon: box.lk.icons.page.on
                name: box.res.tr("page.on")
                can: box.at < box.pages - 1 && !!box.view.page_on
                onClicked: box.page(box.view.page_on)
            }
        }
    }

    // A page button (mock-up .pg), dimmed when there is no page that way.
    component PageButton: Rectangle {
        id: pb
        required property string icon
        required property string name
        required property bool can
        signal clicked
        width: box.pn.page_px[0] * box.s
        height: box.pn.page_px[1] * box.s
        radius: box.pn.page_px[2] * box.s
        color: box.p.pop_hover
        border.width: box.lk.shape.line.px
        border.color: box.p.pop_line
        enabled: can
        opacity: can ? 1 : box.pn.off_share
        Accessible.role: Accessible.Button
        Accessible.name: name
        Accessible.onPressAction: if (can) pb.clicked()
        Hover {
            lk: box.lk
            radius: pb.radius
            tint: box.p.pop_sel
            visible: area.containsMouse
        }
        Icon {
            anchors.centerIn: parent
            icon: pb.icon
            base: box.res.icons
            ext: box.res.ext
            px: box.pn.page_px[3] * box.s
            tint: box.lk.sel_ink
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: if (pb.can) pb.clicked()
        }
    }

    PopIn {
        id: popIn
        target: body
        move: box.lk.motion.moves.panel
        amount: box.lk.motion.amount
    }
}

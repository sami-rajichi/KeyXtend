// The email list over the keys and the pad (spec §6 Emails; mock-up .langs and .lg-it): one page of known emails, a
// pager beside them when there is more than one page, and Cancel, never a scroll bar (ADR-0008). Rows read left to
// right in every language; each tap goes to the hub, which pages and closes it, and it pops in once as it opens.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: box

    // The list's page (`{items, page, pages}`, the page from 0), the board whose keys and pad it covers, rows per page,
    // the app's Ui object for the page number, and the look, fonts, icons, texts and size.
    required property var list
    required property var board
    required property int rows
    required property var ui
    required property var lk
    required property var res
    required property real s

    // A row, page or Cancel tap: its `Ui.wordsAsk` kind and number.
    signal ask(string kind, int arg)

    readonly property var sh: lk.shape.list
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property var asks: names.asks
    readonly property int per: Math.max(1, rows)
    readonly property var shown: list ? list.items.slice(0, per) : []
    readonly property bool paged: !!list && list.pages > 1
    readonly property real pad: sh.pad_px * s
    // The box the keys and the pad take, which the list covers.
    readonly property var over: {
        const all = board.keys.concat([board.dpad]);
        const x = Math.min(...all.map(k => k.x)), y = Math.min(...all.map(k => k.y));
        const right = Math.max(...all.map(k => k.x + k.w)), bottom = Math.max(...all.map(k => k.y + k.h));
        return Qt.rect(x, y, right - x, bottom - y);
    }

    x: over.x
    y: over.y
    width: over.width
    height: over.height
    Accessible.role: Accessible.List
    Accessible.name: res.tr("chip.more")
    Accessible.id: res.uid(names.group, "list")
    Component.onCompleted: popIn.restart()

    WordNames {
        id: names
    }

    Rectangle {
        id: body
        width: box.width
        height: box.height
        radius: box.lk.look.window_radius_px * box.pn.radius_share
        color: box.p.pop_bg
        border.width: box.lk.shape.line.px
        border.color: box.p.pop_line

        // Takes every press and hover on the list, so the keys under it rest.
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
        }
        Column {
            id: rowsBox
            x: box.pad
            y: box.pad
            width: buttons.x - box.pn.body_gap_px * box.s - box.pad
            Repeater {
                model: box.shown
                delegate: EmailRow {
                    required property string modelData
                    required property int index
                    width: rowsBox.width
                    height: (box.height - 2 * box.pad) / box.per
                    email: modelData
                    place: index
                }
            }
        }
        Row {
            id: buttons
            anchors.right: parent.right
            anchors.rightMargin: box.pad
            anchors.verticalCenter: parent.verticalCenter
            spacing: box.pn.body_gap_px * box.s
            Pager {
                anchors.verticalCenter: parent.verticalCenter
                page: box.paged ? box.list.page : 0
                pages: box.paged ? box.list.pages : 0
                canBack: box.paged && box.list.page > 0
                canOn: box.paged && box.list.page < box.list.pages - 1
                ui: box.ui
                lk: box.lk
                res: box.res
                s: box.s
                backId: box.res.uid(names.group, "list-back")
                onId: box.res.uid(names.group, "list-on")
                onTurn: by => box.ask(box.asks.page, by)
            }
            PageButton {
                anchors.verticalCenter: parent.verticalCenter
                icon: box.lk.icons.words.cancel
                name: box.res.tr("words.cancel")
                can: true
                lk: box.lk
                res: box.res
                s: box.s
                uid: box.res.uid(names.group, "list-cancel")
                onClicked: box.ask(box.asks.cancel, 0)
            }
        }
    }

    // One email row (mock-up .lg-it): the at-sign and the email, left to right, ringed by the hover.
    component EmailRow: Item {
        id: r
        required property string email
        required property int place
        readonly property bool hot: area.containsMouse
        Accessible.role: Accessible.ListItem
        Accessible.name: email
        Accessible.id: box.res.uid(names.group, "list-row-" + place)
        Accessible.onPressAction: box.ask(box.asks.pick, r.place)
        Hover {
            lk: box.lk
            radius: box.sh.row_radius_px * box.s
            visible: r.hot
            tint: box.p.pop_hover
        }
        Icon {
            id: at
            anchors.left: parent.left
            anchors.leftMargin: box.sh.row_pad_px[1] * box.s
            anchors.verticalCenter: parent.verticalCenter
            icon: box.lk.icons.words.email
            base: box.res.icons
            ext: box.res.ext
            px: box.sh.icon_px * box.s
            // Hover text sits on pop_hover, which is Highlight in high contrast, so it takes sel_ink there.
            tint: r.hot ? box.lk.sel_ink : box.p.pop_muted
        }
        Label {
            anchors.left: at.right
            anchors.leftMargin: box.sh.gap_px * box.s
            anchors.right: parent.right
            anchors.rightMargin: box.sh.row_pad_px[1] * box.s
            anchors.verticalCenter: parent.verticalCenter
            text: r.email
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignLeft
            font.family: box.res.fam.latin
            font.pixelSize: box.sh.name_px * box.s
            color: r.hot ? box.lk.sel_ink : box.p.pop_ink
        }
        // It acts at let-go, wherever the pointer went (ADR-0024).
        Act {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onActed: box.ask(box.asks.pick, r.place)
        }
    }

    PopIn {
        id: popIn
        target: body
        move: box.lk.motion.moves.panel
        amount: box.lk.motion.amount
    }
}

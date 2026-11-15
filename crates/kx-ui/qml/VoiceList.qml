// A Voice panel page's list (mock-up .pl and .pager): five row slots on every page, a short page padded so nothing
// jumps, the column heads over Compare's table, and the pager beside them, never a scroll bar (ADR-0008); an empty
// History shows one line with its icon in place of the rows.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: listing

    // The panel window, whose view, look, names and row height it takes.
    required property var host

    readonly property var view: host.view
    readonly property var sheet: view.sheet
    readonly property var names: host.names
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property real rowH: host.rowH
    readonly property real rowW: frame.width
    readonly property real side: pn.body_pad_px[1] * s
    readonly property bool headed: sheet.is === names.sheets.results

    // How many slots a page body of kind `is` fills: every one for the shown kind, none for the rest or an empty listing.
    function slots(is) {
        return sheet.is === is && !sheet.empty ? host.per : 0;
    }
    // Slot `i`'s row, null past the page's end.
    function entry(i) {
        const all = sheet.rows || [];
        return i < all.length ? all[i] : null;
    }

    height: (pn.body_pad_px[0] + pn.body_pad_px[2]) * s + (headed ? head.height + pn.row_px[1] * s : 0) + host.pageH

    Item {
        id: frame
        x: listing.side
        y: listing.pn.body_pad_px[0] * listing.s
        // The pager's room stays, so rows keep their width as views change.
        width: listing.width - 2 * listing.side - (listing.pn.body_gap_px + listing.pn.page_px[0]) * listing.s
        height: col.y + listing.host.pageH

        VoiceHeads {
            id: head
            visible: listing.headed
            width: parent.width
            list: listing
        }
        Column {
            id: col
            y: listing.headed ? head.height + listing.pn.row_px[1] * listing.s : 0
            width: parent.width
            spacing: listing.pn.row_px[1] * listing.s
            Accessible.role: Accessible.List
            Accessible.name: listing.view.title
            Accessible.id: listing.host.res.uid(listing.names.group, listing.names.ids.list)
            Repeater {
                model: listing.slots(listing.names.sheets.home)
                delegate: VoiceHome { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.models)
                delegate: VoiceModels { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.device)
                delegate: VoiceDevice { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.running)
                delegate: VoiceRun { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.results)
                delegate: VoiceCompare { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.history)
                delegate: VoiceHistory { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.options)
                delegate: VoiceOptions { list: listing }
            }
            Repeater {
                model: listing.slots(listing.names.sheets.folders)
                delegate: VoiceFolder { list: listing }
            }
        }
    }
    Pager {
        anchors.right: parent.right
        anchors.rightMargin: listing.side
        anchors.verticalCenter: parent.verticalCenter
        page: listing.view.pages.at
        pages: listing.view.pages.count
        canBack: listing.view.pages.at > 0 && !listing.host.asking
        canOn: listing.view.pages.at < listing.view.pages.count - 1 && !listing.host.asking
        ui: listing.host.ui
        lk: listing.lk
        res: listing.host.res
        s: listing.s
        backId: listing.host.res.uid(listing.names.group, listing.names.ids.pageBack)
        onId: listing.host.res.uid(listing.names.group, listing.names.ids.pageOn)
        onTurn: by => listing.host.send(listing.names.acts.page, "", by)
    }
    Column {
        visible: !!listing.sheet.empty
        anchors.centerIn: parent
        spacing: listing.pn.body_gap_px * listing.s
        Icon {
            anchors.horizontalCenter: parent.horizontalCenter
            icon: listing.lk.icons.voice.panel.pages[listing.view.page] || ""
            base: listing.host.res.icons
            ext: listing.host.res.ext
            px: listing.pn.empty_px * listing.s
            tint: listing.lk.palette.pop_muted
        }
        Label {
            anchors.horizontalCenter: parent.horizontalCenter
            width: Math.min(implicitWidth, listing.width - 2 * listing.side)
            text: listing.sheet.empty || ""
            wrapMode: Text.Wrap
            horizontalAlignment: Text.AlignHCenter
            font.family: listing.host.res.fam.ui
            font.pixelSize: listing.pn.title_px * listing.s * (listing.host.ar ? listing.lk.look.arabic_scale : 1)
            color: listing.lk.palette.pop_muted
            Accessible.ignored: false
            Accessible.role: Accessible.StaticText
            Accessible.name: text
        }
    }
}

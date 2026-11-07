// Quick-fill's list and By label's labels (design "The panel"): the search line and the filters over five rows, each a
// kind's entry with its buttons, or a label with its count and Open. An empty notebook invites a first entry or an
// import, and a search or filter with no match says so.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.list, box.names.pages.labels]

    // Each row as a slot draws it: a label with its count, or an entry by its kind's icon.
    readonly property var rows: sheet.rows.map(r => r.label ? {
            key: r.label.text,
            icon: ic.label,
            title: r.label,
            hint: box.worded(r.values),
            buttons: r.buttons
        } : {
            key: r.id,
            icon: names.kindIcon(ic, r.kind),
            title: r.name,
            hint: r.hint,
            buttons: r.buttons
        })

    PanelLine {
        x: page.side
        y: page.lk.shape.fill.top_px[0] * page.s
        width: page.width - 2 * page.side
        said: page.view.search
        hint: page.view.search.hint
        icon: page.ic.search
        at: page.view.search.caret
        takes: page.box.aimed
        search: true
        rtl: page.box.rtl
        lk: page.lk
        res: page.res
        s: page.s
        uid: page.res.uid(page.names.group, page.names.ids.search)
        onPlaced: at => page.box.act(page.names.acts.caret, "", at)
    }
    QuickFillFilters {
        x: page.side
        y: page.box.bandY
        height: page.lk.shape.clip.button_px[0] * page.s
        filters: page.view.filters
        names: page.names
        rtl: page.box.rtl
        lk: page.lk
        res: page.res
        s: page.s
        onPicked: id => page.box.act(page.names.acts.filter, id, 0)
    }
    QuickFillRows {
        id: slots
        box: page.box
        rows: page.rows
        name: page.view.title
    }
    // The empty notebook's invitation, with New and Import where the rows would be.
    QuickFillInvite {
        visible: !!page.view.empty
        anchors.horizontalCenter: slots.horizontalCenter
        anchors.verticalCenter: slots.verticalCenter
        width: slots.width
        icon: page.ic.title
        text: page.view.empty || ""
        ink: page.lk.palette.pop_muted
        lk: page.lk
        res: page.res
        s: page.s
        QuickFillButtons {
            anchors.horizontalCenter: parent.horizontalCenter
            buttons: [{ id: page.ids.new, label: page.view.buttons[page.ids.new], icon: page.ic.new, tone: "enter", uid: page.res.uid(page.names.group, page.names.ids.invite) }, page.box.button(page.ids.keepass, page.ic.keepass)]
            lk: page.lk
            res: page.res
            s: page.s
            onClicked: id => page.box.press(id)
        }
    }
    Label {
        visible: !!page.sheet.none
        anchors.centerIn: slots
        width: slots.width
        text: page.sheet.none || ""
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        font.family: page.res.fam.ui
        font.pixelSize: page.box.pn.title_px * page.s
        color: page.lk.palette.pop_muted
        Accessible.ignored: !visible
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
}

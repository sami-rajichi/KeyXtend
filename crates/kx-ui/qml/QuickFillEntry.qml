// One entry's page (design "The panel"): its kind's tile and name, Back and Edit, then each field with its label, its
// value or the mask, Show for a hidden one and Insert, which types it into the app in front.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.entry]

    QuickFillTop {
        box: page.box
        icon: page.names.kindIcon(page.ic, page.sheet.kind)
        title: page.sheet.name
        starts: [page.box.button(page.ids.back, page.ic.back)]
        ends: [page.box.button(page.ids.edit, page.ic.edit)]
    }
    QuickFillSlots {
        id: slots
        x: page.side
        y: page.box.topH
        width: page.width - 2 * page.side
        per: page.box.per
        rowH: page.box.rowH
        count: page.sheet.fields.length
        hi: page.view.highlight
        name: page.sheet.name.text
        uid: page.res.uid(page.names.group, page.names.ids.list)
        lk: page.lk
        s: page.s
        row: QuickFillField {
            required property int index
            width: slots.width
            height: page.box.rowH
            slot: page.sheet.fields[index] || null
            place: index
            at: page.view.page.at
            hi: index === page.view.highlight
            edit: false
            names: page.names
            words: page.view.words
            rtl: page.box.rtl
            lk: page.lk
            res: page.res
            s: page.s
            onPressed: id => page.box.act(page.names.acts.field, id, index)
        }
    }
}

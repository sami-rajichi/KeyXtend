// Add a field (design "New and Edit"): the labels table, paged, each label with Add, and Your own label for a name of
// one's own; Back returns to the entry being edited.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.add]

    readonly property string title: view.buttons[ids.addField]

    QuickFillTop {
        box: page.box
        icon: page.ic.add_field
        title: page.box.worded(page.title)
        starts: [page.box.button(page.ids.back, page.ic.back)]
        ends: [page.box.button(page.ids.ownLabel, page.ic.own_label)]
    }
    // Each label as a slot draws it, its Add with the add icon.
    QuickFillRows {
        box: page.box
        rows: page.sheet.rows.map(r => ({
                    key: r.label.text,
                    icon: page.ic.label,
                    title: r.label,
                    hint: null,
                    buttons: r.buttons.map(b => ({ id: b.id, label: b.label, icon: page.ic.add_field }))
                }))
        name: page.title
    }
}

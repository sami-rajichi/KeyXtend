// By label's second page (design "The panel"): the label it opened, then every value carrying it across entries, each
// with its entry's name and Insert, a hidden one as the mask; Back returns to the labels.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.values]

    QuickFillTop {
        box: page.box
        icon: page.ic.label
        title: page.sheet.label
        starts: [page.box.button(page.ids.back, page.ic.back)]
    }
    // Each value as a slot draws it: the value, or null for the mask, over its entry's name.
    QuickFillRows {
        box: page.box
        rows: page.sheet.rows.map((r, i) => ({
                    key: i + r.name.text,
                    icon: page.ic.label,
                    title: r.value,
                    hint: r.name,
                    buttons: r.buttons
                }))
        name: page.sheet.label.text
    }
}

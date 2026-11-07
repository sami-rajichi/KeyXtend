// A Quick-fill page's rows in the frame's place (design "The panel"): each slot a QuickFillRow with the view's page and
// highlight, a padding slot blank, and a click on a row's button sent with its place.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillSlots {
    id: slots

    // The panel's window; each row as a slot draws it; and the place a question stands in, which draws no row, or -1.
    required property var box
    required property var rows
    property int asked: -1

    x: box.side
    y: box.topH
    width: box.width - 2 * box.side
    per: box.per
    rowH: box.rowH
    count: rows.length
    hi: box.view.highlight
    uid: box.res.uid(box.names.group, box.names.ids.list)
    lk: box.lk
    s: box.s
    row: QuickFillRow {
        required property int index
        width: slots.width
        height: slots.rowH
        slot: index === slots.asked ? null : slots.rows[index] || null
        place: index
        at: slots.box.view.page.at
        hi: index === slots.box.view.highlight
        names: slots.box.names
        mask: slots.box.view.words.mask
        lk: slots.lk
        res: slots.box.res
        s: slots.s
        onPressed: id => slots.box.act(slots.box.names.acts.row, id, index)
    }
}

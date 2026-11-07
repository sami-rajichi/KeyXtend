// A backup to bring in (design "New computer" and "A box this computer did not make"): the boxes found, newest first,
// each saying where and when with Bring back, and Another folder for one elsewhere; or one question about a box from
// another computer, whose Yes and No answer it.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.found, box.names.pages.foreign]

    readonly property string title: view.buttons[ids.bringBack]

    // Only the boxes found have rows; each binding asks the sheet itself, so found and foreign never mix.
    QuickFillTop {
        box: page.box
        icon: page.sheet.rows ? page.ic.bring_back : page.names.placeIcon(page.ic, page.sheet.kind)
        title: page.sheet.rows ? page.box.worded(page.title) : page.sheet.name
        starts: [page.box.button(page.ids.back, page.ic.back)]
        ends: page.sheet.rows ? [page.box.button(page.ids.otherFolder, page.ic.other_folder)] : []
    }
    QuickFillRows {
        visible: !!page.sheet.rows
        box: page.box
        rows: (page.sheet.rows || []).map(r => page.box.placeRow(r, page.box.worded(r.text), { icon: page.ic.bring_back }))
        name: page.title
    }
    // Yes loses nothing, as the box stays until its password opens it, so it wears Enter's colours.
    QuickFillAsk {
        x: page.side
        y: page.box.topH
        width: page.width - 2 * page.side
        height: page.box.slotsH
        box: page.box
        question: page.sheet.rows ? "" : page.sheet.text
        icon: page.ic.bring_back
        yesTone: "enter"
    }
}

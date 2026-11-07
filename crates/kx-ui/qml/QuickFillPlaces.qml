// The suggested places (design "The backup box"): what the page asks, then the cloud folders and the drives by their
// kind's icon, each with Choose; Another folder opens the paged folder list, Bring back finds a backup instead, and
// Import brings a KeePass file in.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.places]

    QuickFillTop {
        box: page.box
        icon: page.ic.backup
        title: page.box.worded(page.sheet.text)
        starts: [page.box.button(page.ids.back, page.ic.back)]
        ends: [page.box.button(page.ids.otherFolder, page.ic.other_folder), page.box.button(page.ids.bringBack, page.ic.bring_back), page.box.button(page.ids.keepass, page.ic.keepass)]
    }
    QuickFillRows {
        box: page.box
        rows: page.sheet.rows.map(r => page.box.placeRow(r, null, { icon: page.ic.pick }))
        name: page.sheet.text
    }
}

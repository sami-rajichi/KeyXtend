// The password page (design "The backup box" and "Import"): the line takes the letters clicked on our keyboard while
// aimed at (ADR-0040) and shows them (owner), while readers hear only the mask; below it Back and Done, then what the
// page asks in the rows' place: the fewest characters, the tries left, or whether the box is saved again under this
// computer's padlock.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.password]

    PanelLine {
        x: page.side
        y: page.box.fs.top_px[0] * page.s
        width: page.width - 2 * page.side
        said: page.sheet.typed
        hint: ""
        name: page.sheet.text
        at: page.sheet.caret
        takes: page.box.aimed
        secret: true
        mask: page.view.words.mask
        rtl: page.box.rtl
        lk: page.lk
        res: page.res
        s: page.s
        uid: page.res.uid(page.names.group, page.names.ids.password)
        onPlaced: at => page.box.act(page.names.acts.caret, "", at)
    }
    QuickFillTop {
        box: page.box
        starts: [page.box.button(page.ids.back, page.ic.back)]
        ends: [page.box.button(page.ids.done, page.ic.done, "enter")]
    }
    QuickFillInvite {
        x: page.side
        y: page.box.topH + (page.box.slotsH - height) / 2
        width: page.width - 2 * page.side
        icon: page.sheet.ask === page.names.asks.keepass ? page.ic.keepass : page.ic.lock
        text: page.sheet.text
        ink: page.lk.palette.pop_ink
        lk: page.lk
        res: page.res
        s: page.s
    }
}

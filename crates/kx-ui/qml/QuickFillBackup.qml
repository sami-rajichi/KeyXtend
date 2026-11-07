// The Backup page (design "The backup box"): each place's row with its kind, name and state, and below the rows Back,
// Back up now, Add place, Bring back, Change password, the memory checks switch and Import. Remove place asks once on
// its row and Replace in the rows' place, while the page's buttons wait; a page with no place invites one.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.backup]

    // The place Remove asks about, read with the question, so a new sheet never pairs a row with another's places.
    readonly property var asked: sheet.confirm ? sheet.places[sheet.confirm.row] || null : null
    readonly property bool asking: box.asking
    readonly property real buttonH: lk.shape.clip.button_px[0] * s
    readonly property string checksId: names.ids.checks
    // Each place as a slot draws it, with its state under its name.
    readonly property var rows: sheet.places.map(r => box.placeRow(r, box.worded(r.state), { can: !asking }))
    // A panel button `id` with its icon, waiting while a question shows.
    function button(id, icon, can) {
        return box.button(id, icon, "ghost", !asking && can !== false);
    }
    // The memory checks switch, which reads as on or off by its icon and its fill.
    readonly property var checks: ({
            id: checksId,
            label: view.words.checks,
            icon: sheet.checks ? ic.checks_on : ic.checks_off,
            tone: sheet.checks ? "shown" : "ghost",
            can: !asking,
            checked: sheet.checks,
            uid: res.uid(names.group, checksId)
        })

    // The rows take the frame's first place, and its two lines go below them, as the design has it.
    QuickFillRows {
        id: slots
        visible: !page.sheet.replace
        y: page.box.fs.top_px[0] * page.s
        box: page.box
        rows: page.rows
        asked: page.sheet.confirm ? page.sheet.confirm.row : -1
        name: page.view.buttons[page.ids.backup]
    }
    QuickFillAsk {
        x: slots.x
        y: slots.y + Math.max(slots.asked, 0) * (slots.rowH + slots.gap)
        width: slots.width
        height: slots.rowH
        box: page.box
        flat: true
        side: page.box.pn.row_px[2] * page.s
        question: page.sheet.confirm ? page.sheet.confirm.text : ""
        icon: page.asked ? page.names.placeIcon(page.ic, page.asked.kind) : ""
        yesIcon: page.ic.remove
    }
    QuickFillAsk {
        x: slots.x
        y: slots.y
        width: slots.width
        height: slots.height
        box: page.box
        question: page.sheet.replace ? page.sheet.replace.text : ""
        icon: page.ic.back_up_now
    }
    QuickFillInvite {
        visible: !!page.sheet.none
        x: slots.x
        y: slots.y + (slots.height - height) / 2
        width: slots.width
        icon: page.ic.backup
        text: page.sheet.none || ""
        ink: page.lk.palette.pop_muted
        lk: page.lk
        res: page.res
        s: page.s
    }
    QuickFillButtons {
        x: slots.x
        y: slots.y + slots.height + page.box.fs.top_px[1] * page.s
        buttons: [page.button(page.ids.back, page.ic.back), page.button(page.ids.backUpNow, page.ic.back_up_now, page.rows.length > 0), page.button(page.ids.addPlace, page.ic.add_place), page.button(page.ids.bringBack, page.ic.bring_back)]
        lk: page.lk
        res: page.res
        s: page.s
        onClicked: id => page.box.press(id)
    }
    QuickFillButtons {
        x: slots.x
        y: page.box.topH + page.box.slotsH - page.buttonH
        buttons: [page.button(page.ids.changePassword, page.ic.change_password), page.checks, page.button(page.ids.keepass, page.ic.keepass)]
        lk: page.lk
        res: page.res
        s: page.s
        onClicked: id => {
            if (id === page.checksId)
                page.box.act(page.names.acts.checks, "", page.sheet.checks ? 0 : 1);
            else
                page.box.press(id);
        }
    }
}

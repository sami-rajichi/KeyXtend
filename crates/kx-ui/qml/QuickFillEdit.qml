// New and Edit (design "New and Edit"): New first asks the kind with its chips; then the name line and the fields, each
// line taking the letters at a click while aimed at (ADR-0040), with Add a field, Delete entry on Edit, Cancel and Save.
// Delete entry's question takes the fields' place until Yes or No answers it, and the page's buttons wait meanwhile.
pragma ComponentBehavior: Bound
import QtQuick

QuickFillPage {
    id: page

    draws: [box.names.pages.edit]

    readonly property bool kinds: sheet.kind === null
    readonly property bool asking: box.asking
    // A panel button `id` with its icon and look, waiting while the question shows.
    function button(id, icon, tone) {
        return box.button(id, icon, tone, !asking);
    }
    // The buttons on the right: Cancel while New asks the kind; then Delete entry on Edit, Cancel and Save.
    readonly property var ends: {
        const cancel = button(ids.cancel, ic.cancel, "ghost");
        if (kinds)
            return [cancel];
        const drop = sheet.new ? [] : [button(ids.delete, ic.delete, "danger")];
        return drop.concat([cancel, button(ids.save, ic.save, "enter")]);
    }

    // New asks what to save until a kind is picked; then the name takes the first line.
    Label {
        visible: page.kinds
        x: page.side
        y: page.lk.shape.fill.top_px[0] * page.s
        width: page.width - 2 * page.side
        height: page.box.lineH
        verticalAlignment: Text.AlignVCenter
        text: page.sheet.ask || ""
        elide: Text.ElideRight
        font.family: page.res.fam.ui
        font.pixelSize: page.box.pn.title_px * page.s
        font.weight: page.box.pn.title_weight
        color: page.lk.palette.pop_ink
        Accessible.ignored: !visible
        Accessible.role: Accessible.Heading
        Accessible.name: text
    }
    PanelLine {
        visible: !page.kinds
        x: page.side
        y: page.lk.shape.fill.top_px[0] * page.s
        width: page.width - 2 * page.side
        said: page.sheet.name
        hint: ""
        caption: page.view.buttons[page.ids.name]
        share: page.lk.shape.fill.own_share
        at: page.sheet.caret
        takes: page.sheet.typing && page.box.aimed
        aims: !page.asking
        rtl: page.box.rtl
        lk: page.lk
        res: page.res
        s: page.s
        uid: page.res.uid(page.names.group, page.ids.name)
        onClicked: page.box.press(page.ids.name)
        onPlaced: at => page.box.act(page.names.acts.caret, "", at)
    }
    QuickFillButtons {
        visible: !page.kinds
        x: page.side
        y: page.box.bandY
        buttons: [page.button(page.ids.addField, page.ic.add_field, "ghost")]
        lk: page.lk
        res: page.res
        s: page.s
        onClicked: id => page.box.press(id)
    }
    QuickFillButtons {
        anchors.right: parent.right
        anchors.rightMargin: page.side
        y: page.box.bandY
        buttons: page.ends
        lk: page.lk
        res: page.res
        s: page.s
        onClicked: id => page.box.press(id)
    }
    QuickFillKinds {
        visible: page.kinds
        x: page.side
        y: page.box.topH
        width: page.width - 2 * page.side
        height: page.box.slotsH
        kinds: page.sheet.kinds
        names: page.names
        rtl: page.box.rtl
        lk: page.lk
        res: page.res
        s: page.s
        onPicked: id => page.box.act(page.names.acts.kind, id, 0)
    }
    QuickFillSlots {
        id: slots
        visible: !page.kinds && !page.asking
        x: page.side
        y: page.box.topH
        width: page.width - 2 * page.side
        per: page.box.per
        rowH: page.box.rowH
        count: page.sheet.fields.length
        hi: page.view.highlight
        ringed: false
        name: page.sheet.title
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
            edit: true
            aimed: page.box.aimed
            caret: page.sheet.caret
            names: page.names
            words: page.view.words
            rtl: page.box.rtl
            lk: page.lk
            res: page.res
            s: page.s
            onPressed: id => page.box.act(page.names.acts.field, id, index)
            onPlaced: at => page.box.act(page.names.acts.caret, "", at)
        }
    }
    // Delete entry's one question, in the fields' place.
    QuickFillAsk {
        x: page.side
        y: page.box.topH
        width: page.width - 2 * page.side
        height: page.box.slotsH
        box: page.box
        question: page.sheet.confirm || ""
        icon: page.ic.delete
    }
}

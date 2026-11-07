// One Quick-fill field (design "The panel"): on an entry's page its label over its value, with Insert and Show; on New
// and Edit its value line, and its own label's line, each taking the letters at a click while aimed at (ADR-0040), with
// Show or Hide, the hidden switch, Paste and Make one as the view gives them. A hidden value shows the mask until shown,
// then fades in; on New and Edit the mask is one dot per letter, and an empty value shows none.
import QtQuick

QuickFillSlot {
    id: f

    // The slot holds the view's field; whether the page is New or Edit, whether a line takes the letters, the page's one
    // caret, for the line that takes them, the click names, the view's words, whether the keyboard's language reads right
    // to left, the fonts and icons.
    required property bool edit
    property bool aimed: false
    property int caret: 0
    required property var names
    required property var words
    required property bool rtl
    required property var res

    // Field button or line `id` was clicked on this field; a line's click then says which place in what it draws.
    signal pressed(string id)
    signal placed(int at)

    readonly property var fs: lk.shape.fill
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    readonly property var ic: lk.icons.fill
    readonly property var label: filled ? slot.label : null
    readonly property var value: filled ? slot.value : null
    readonly property bool own: filled && slot.own
    readonly property bool revealed: filled && slot.shown
    readonly property real gap: pn.row_gap_px * s
    readonly property real side: lk.shape.panel.row_px[2] * s
    // What readers hear after the label: the value, or the fixed mask while hidden, never a hidden value or its count.
    readonly property string heard: !filled ? "" : value ? value.text : words.mask

    // Whether field button `id` is on: Show while the value shows, the hidden switch while it is on.
    function lit(id) {
        return id === names.field.show ? revealed : id === names.field.hidden && filled && slot.hidden;
    }
    // Field button `b` as the panels' button draws it: the hidden switch is a checkbox, filled while on.
    function look(b) {
        const on = lit(b.id), toggle = b.id === names.field.hidden;
        const about = label ? label.text : "";
        return { id: b.id, label: b.label, icon: names.fieldIcon(ic, b.id, on), tone: toggle && on ? "shown" : "ghost", checked: toggle ? on : null, uid: uidOf(b.id), about: about };
    }
    // The gate id of this field's button or line `id`.
    function uidOf(id) {
        return res.uid(names.group, names.ids.field + place + names.ids.part + id);
    }

    key: !filled ? "" : own ? names.lines.label : label.text
    onRevealedChanged: if (revealed) reveal.play()
    Accessible.role: Accessible.ListItem
    Accessible.name: label ? label.text : ""
    Accessible.description: heard
    Accessible.id: filled ? res.uid(names.group, names.ids.field + place) : ""

    Rise {
        id: reveal
        anchors.left: parent.left
        anchors.leftMargin: f.side
        anchors.right: trail.left
        anchors.rightMargin: f.gap
        height: parent.height
        move: f.lk.motion.fill.reveal
        from: 0

        // The read page: the label over its value.
        Column {
            visible: !f.edit
            width: parent.width
            anchors.verticalCenter: parent.verticalCenter
            Label {
                width: parent.width
                text: f.label ? f.label.text : ""
                elide: Text.ElideRight
                horizontalAlignment: f.label && f.label.rtl ? Text.AlignRight : Text.AlignLeft
                font.family: f.label && f.label.arabic ? f.res.fam.arabic : f.res.fam.latin
                font.pixelSize: f.pn.hint_px[0] * f.s
                color: f.p.pop_muted
            }
            Label {
                width: parent.width
                topPadding: f.pn.hint_px[1] * f.s
                text: f.value ? f.value.text : f.words.mask
                elide: Text.ElideRight
                horizontalAlignment: f.value && f.value.rtl ? Text.AlignRight : Text.AlignLeft
                font.family: f.value && f.value.arabic ? f.res.fam.arabic : f.res.fam.latin
                font.pixelSize: f.lk.shape.panel.row_text_px[f.value && f.value.arabic ? 1 : 0] * f.s
                font.letterSpacing: f.value ? 0 : f.fs.mask_em * f.lk.shape.panel.row_text_px[0] * f.s
                color: f.value ? f.p.pop_ink : f.p.pop_muted
            }
        }
        // New and Edit: an own label's line, then the value's, which names a table label before its text.
        Row {
            visible: f.edit
            anchors.verticalCenter: parent.verticalCenter
            spacing: f.gap
            PanelLine {
                visible: f.own
                width: reveal.width * f.fs.own_share
                said: f.own ? f.label : { text: "", rtl: false, arabic: false }
                hint: f.words.label
                at: f.caret
                takes: f.aimed && f.filled && f.slot.typing === f.names.lines.label
                aims: f.own
                rtl: f.rtl
                lk: f.lk
                res: f.res
                s: f.s
                uid: f.filled ? f.uidOf(f.names.lines.label) : ""
                onClicked: f.pressed(f.names.lines.label)
                onPlaced: at => f.placed(at)
            }
            PanelLine {
                width: reveal.width - (f.own ? reveal.width * f.fs.own_share + f.gap : 0)
                said: f.value
                hint: f.words.value
                caption: f.own || !f.label ? "" : f.label.text
                name: f.label ? f.label.text : f.words.value
                at: f.caret
                takes: f.aimed && f.filled && f.slot.typing === f.names.lines.value
                aims: f.edit && f.filled
                mask: f.words.mask
                count: f.filled && f.slot.count ? f.slot.count : 0
                maskEm: f.fs.mask_em
                share: f.fs.own_share
                rtl: f.rtl
                lk: f.lk
                res: f.res
                s: f.s
                uid: f.filled ? f.uidOf(f.names.lines.value) : ""
                onClicked: f.pressed(f.names.lines.value)
                onPlaced: at => f.placed(at)
            }
        }
    }
    QuickFillButtons {
        id: trail
        anchors.right: parent.right
        anchors.rightMargin: f.side
        anchors.verticalCenter: parent.verticalCenter
        buttons: !f.filled ? [] : f.slot.buttons.map(b => f.look(b))
        lk: f.lk
        res: f.res
        s: f.s
        onClicked: id => f.pressed(id)
    }
}

// One clipboard row (spec §7; mock-up .lrow): tick box, thumbnail or kind icon, two title lines over "time · size ·
// app", pin and expires chip; a click only ticks it, and a padding slot stays blank. An Arabic title reads from the
// right in its font while each part keeps its place (ADR-0020); new rows rise in, page turns slide and ticks pop.
import QtQuick

Item {
    id: r

    // The slot (`{r, at}`: the row or null for padding, and the page it shows on), its place on the page, whether it
    // is highlighted, the expires chip's text, the click names, the look, fonts, icons and size.
    required property var slot
    required property int place
    required property bool hi
    required property string expires
    required property var names
    required property var lk
    required property var res
    required property real s

    // The row was clicked or toggled by a screen reader.
    signal ticked

    readonly property var entry: slot ? slot.r : null
    readonly property bool filled: entry !== null
    readonly property bool on: filled && entry.tick !== null
    readonly property bool hot: filled && area.containsMouse
    readonly property bool lit: hot || on
    readonly property bool ar: filled && entry.arabic
    readonly property var pn: lk.shape.panel
    readonly property var c: lk.shape.clip
    readonly property var p: lk.palette
    // The ring's and the empty tick box's colour; lit rows sit on pop_hover or pop_sel, Highlight in high contrast.
    readonly property color mark: (hot || (on && lk.contrast)) ? lk.sel_ink : p.ring
    readonly property real gap: c.row_gap_px * s

    // What the slot showed last, so a new view knows what moved.
    property string lastId: ""
    property int lastAt: -1
    property var lastTick: null

    // Plays what changed since the last view: a page turn, a new row in this slot, or a new tick.
    function moved() {
        const was = { id: lastId, at: lastAt, tick: lastTick };
        lastId = filled ? entry.id : "";
        lastAt = slot ? slot.at : -1;
        lastTick = on ? entry.tick : null;
        if (!filled || was.at < 0)
            return;
        if (lastAt !== was.at)
            rise.slide(lk.motion.clip.turn, lastAt > was.at ? 1 : -1);
        else if (lastId !== was.id)
            rise.slide(lk.motion.clip.swap, 1);
        else if (on && was.tick === null)
            box.pop();
    }

    onSlotChanged: moved()
    Component.onCompleted: {
        lastId = filled ? entry.id : "";
        lastAt = slot ? slot.at : -1;
        lastTick = on ? entry.tick : null;
    }
    Accessible.role: Accessible.ListItem
    Accessible.name: filled ? entry.name : ""
    Accessible.ignored: !filled
    Accessible.id: filled ? res.uid(names.group, names.ids.row + place) : ""
    Accessible.selectable: true
    Accessible.selected: hi
    Accessible.checkable: true
    Accessible.checked: on
    Accessible.onPressAction: if (r.filled) r.ticked()
    // UI Automation's Toggle on a checkable row asks Qt for this action, not the press.
    Accessible.onToggleAction: if (r.filled) r.ticked()

    Rise {
        id: rise
        anchors.fill: parent
        visible: r.filled
        move: r.lk.motion.clip.swap
        from: 0

        // Slides in with `how` from `by` times the pop-in's rise below its place, or above when `by` is negative.
        function slide(how, by) {
            move = how;
            from = by * r.lk.motion.amount.pop_rise_px * r.s;
            play();
        }

        Rectangle {
            anchors.fill: parent
            visible: r.on
            radius: r.pn.row_px[3] * r.s
            color: r.p.pop_sel
        }
        Hover {
            lk: r.lk
            radius: r.pn.row_px[3] * r.s
            visible: r.hot
            tint: r.p.pop_hover
        }
        Row {
            id: lead
            anchors.left: parent.left
            anchors.leftMargin: r.pn.row_px[2] * r.s
            anchors.verticalCenter: parent.verticalCenter
            spacing: r.gap
            ClipTick {
                id: box
                anchors.verticalCenter: parent.verticalCenter
                number: r.filled ? r.entry.tick : null
                mark: r.mark
                lk: r.lk
                res: r.res
                s: r.s
            }
            ClipThumb {
                anchors.verticalCenter: parent.verticalCenter
                url: r.filled ? r.entry.thumb : null
                kind: r.filled ? r.entry.kind : ""
                lit: r.lit
                lk: r.lk
                res: r.res
                s: r.s
            }
        }
        Column {
            anchors.left: lead.right
            anchors.leftMargin: r.gap
            anchors.right: trail.left
            anchors.rightMargin: trail.width > 0 ? r.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            // The first two lines, with an ellipsis when longer; user text, so its direction is the hub's.
            Label {
                width: parent.width
                text: r.filled ? r.entry.title : ""
                wrapMode: Text.Wrap
                maximumLineCount: 2
                elide: Text.ElideRight
                horizontalAlignment: r.filled && r.entry.rtl ? Text.AlignRight : Text.AlignLeft
                font.family: r.ar ? r.res.fam.arabic : r.res.fam.latin
                font.pixelSize: r.pn.row_text_px[r.ar ? 1 : 0] * r.s
                color: r.lit ? r.lk.sel_ink : r.p.pop_ink
            }
            Label {
                width: parent.width
                topPadding: r.c.meta_px[1] * r.s
                text: r.filled ? r.entry.meta : ""
                elide: Text.ElideRight
                font.family: r.res.fam.ui
                font.pixelSize: r.c.meta_px[0] * r.s
                color: r.lit ? r.lk.sel_ink : r.p.pop_muted
            }
        }
        Row {
            id: trail
            anchors.right: parent.right
            anchors.rightMargin: r.pn.row_px[2] * r.s
            anchors.verticalCenter: parent.verticalCenter
            spacing: r.gap
            Icon {
                visible: r.filled && r.entry.pinned
                anchors.verticalCenter: parent.verticalCenter
                icon: r.lk.icons.clip.pin
                base: r.res.icons
                ext: r.res.ext
                px: r.c.pin_px * r.s
                tint: r.lit ? r.lk.sel_ink : r.p.pop_muted
            }
            // Media that goes tonight, on the danger keys' colours.
            Rectangle {
                visible: r.filled && r.entry.expires
                anchors.verticalCenter: parent.verticalCenter
                width: soon.width + 2 * r.c.expires_px[2] * r.s
                height: soon.height + 2 * r.c.expires_px[1] * r.s
                radius: height / 2
                color: r.p.danger
                Label {
                    id: soon
                    anchors.centerIn: parent
                    text: r.expires
                    font.family: r.res.fam.ui
                    font.pixelSize: r.c.expires_px[0] * r.s
                    font.weight: r.lk.shape.chip.weight
                    color: r.p.danger_ink
                }
            }
        }
    }
    // It acts at let-go, wherever the pointer went (ADR-0024).
    Act {
        id: area
        anchors.fill: parent
        enabled: r.filled
        hoverEnabled: true
        onActed: r.ticked()
    }
}

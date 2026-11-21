// One Voice panel row (mock-up .pr): its tick box, tile, name and tag over its line and the parts at its end, on the ticked
// fill or the hover tint. A padding slot stays blank; a click ticks the row or acts, at let-go (ADR-0024). Arabic reads
// from the right in its font while each part keeps its place (ADR-0020); a page turn slides the rows in, a tick pops.
import QtQuick

Item {
    id: r

    // The list, and the slot's place on the page, which the list's Repeater gives.
    required property var list
    required property int index

    // What the row shows: its tick (undefined for no box, null or false unticked, a number or true ticked), its tile's
    // icon, its name and line, whether the name reads from the right in the Arabic font, and whether a click acts.
    property var tick: undefined
    property string tile: ""
    property string name: ""
    property string line: ""
    property bool rtl: host.ar
    property bool arabic: host.ar
    property bool takes: true
    // A tag after its name, such as a Cloud row's Untested (empty for none).
    property string tag: ""
    // The share its tick, tile, name and line show at, below 1 on a dimmed row; the parts at its end set their own.
    property real fade: 1
    // A hint row (mock-up .pr.hint): its tile a bare muted icon, its name a muted line that wraps as a flat question does.
    property bool quiet: false
    // The parts at the row's end, and what shows under its line.
    default property alias ends: ends.data
    property alias under: below.data

    // The row was clicked, or pressed or toggled by a screen reader.
    signal clicked

    readonly property var host: list.host
    readonly property int place: index
    readonly property var entry: list.entry(index)
    readonly property bool filled: entry !== null
    readonly property bool on: filled && tick !== undefined && tick !== null && tick !== false
    readonly property bool hot: filled && takes && area.containsMouse
    readonly property bool lit: hot || on
    readonly property alias nameText: title
    readonly property alias lineText: meta
    readonly property var lk: host.lk
    readonly property var res: host.res
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property var p: lk.palette
    // The empty tick box's colour; lit rows sit on pop_hover or pop_sel, Highlight in high contrast.
    readonly property color outline: (hot || (on && lk.contrast)) ? lk.sel_ink : p.ring
    readonly property real gap: pn.row_gap_px * s
    readonly property real tagGap: lk.shape.chip.gap_px * s
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1

    // The tick the slot showed last, so a new view knows whether it is new.
    property var lastTick: null

    // What names the slot's row across views: its id, else its name or the folder's path.
    function keyOf(e) {
        if (e === null)
            return "";
        return typeof e === "string" ? e : String(e.id !== undefined ? e.id : (e.name || e.label || e.key));
    }
    // Plays what changed since the last view: a page turn or a new row in this slot (RowMove), or a new tick.
    function moved() {
        const was = lastTick;
        lastTick = on ? tick : null;
        if (move.moved(keyOf(entry), list.view.pages.at) && on && was === null)
            box.pop();
    }

    width: list.rowW
    height: list.rowH
    onEntryChanged: moved()
    Component.onCompleted: {
        move.start(keyOf(entry), list.view.pages.at);
        lastTick = on ? tick : null;
    }
    Accessible.role: Accessible.ListItem
    Accessible.name: name
    Accessible.description: line
    Accessible.ignored: !filled
    Accessible.id: filled ? res.uid(host.names.group, host.names.ids.row + place) : ""
    Accessible.checkable: tick !== undefined
    Accessible.checked: on
    Accessible.onPressAction: if (r.filled && r.takes) r.clicked()
    // UI Automation's Toggle on a checkable row asks Qt for this action, not the press.
    Accessible.onToggleAction: if (r.filled && r.takes) r.clicked()

    RowMove {
        id: move
        rise: rise
        turn: r.lk.motion.clip.turn
        swap: r.lk.motion.clip.swap
        lk: r.lk
        s: r.s
    }
    // Under the parts, so the buttons at the row's end take their own clicks.
    Act {
        id: area
        anchors.fill: parent
        enabled: r.filled && r.takes
        hoverEnabled: true
        onActed: r.clicked()
    }
    Rise {
        id: rise
        anchors.fill: parent
        visible: r.filled
        move: r.lk.motion.clip.swap
        from: 0

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
            opacity: r.fade
            anchors.left: parent.left
            anchors.leftMargin: r.pn.row_px[2] * r.s
            anchors.verticalCenter: parent.verticalCenter
            spacing: r.gap
            ClipTick {
                id: box
                visible: r.tick !== undefined
                anchors.verticalCenter: parent.verticalCenter
                number: r.on ? (r.tick === true ? 1 : r.tick) : null
                glyph: r.tick === true ? r.lk.icons.voice.panel.ticked : ""
                mark: r.outline
                lk: r.lk
                res: r.res
                s: r.s
            }
            PanelTile {
                visible: !!r.tile && !r.quiet
                anchors.verticalCenter: parent.verticalCenter
                icon: r.tile
                lk: r.lk
                res: r.res
                s: r.s
            }
            Icon {
                visible: !!r.tile && r.quiet
                anchors.verticalCenter: parent.verticalCenter
                icon: r.tile
                base: r.res.icons
                ext: r.res.ext
                px: r.lk.shape.clip.button_px[4] * r.s
                tint: r.p.pop_muted
            }
        }
        Column {
            opacity: r.fade
            anchors.left: lead.width > 0 ? lead.right : parent.left
            anchors.leftMargin: lead.width > 0 ? r.gap : r.pn.row_px[2] * r.s
            anchors.right: ends.left
            anchors.rightMargin: ends.width > 0 ? r.gap : 0
            anchors.verticalCenter: parent.verticalCenter
            Item {
                width: parent.width
                height: Math.max(title.height, chip.visible ? chip.height : 0)
                Label {
                    id: title
                    // Beside a tag it is as wide as the name reads, so the tag follows the name.
                    width: chip.visible ? Math.min(implicitWidth, parent.width - chip.width - r.tagGap) : parent.width
                    x: chip.visible && r.rtl ? parent.width - width : 0
                    anchors.verticalCenter: parent.verticalCenter
                    text: r.name
                    elide: Text.ElideRight
                    wrapMode: r.quiet ? Text.Wrap : Text.NoWrap
                    maximumLineCount: r.quiet ? r.pn.question_lines : undefined
                    horizontalAlignment: r.rtl ? Text.AlignRight : Text.AlignLeft
                    font.family: r.quiet ? r.res.fam.ui : (r.arabic ? r.res.fam.arabic : r.res.fam.latin)
                    font.pixelSize: (r.quiet ? r.pn.hint_px[0] * r.scale : r.pn.row_text_px[r.arabic ? 1 : 0]) * r.s
                    color: r.lit ? r.lk.sel_ink : (r.quiet ? r.p.pop_muted : r.p.pop_ink)
                }
                // The tag after the name (mock-up .tag): a small outlined chip in the muted ink, which readers hear.
                Rectangle {
                    id: chip
                    visible: !!r.tag
                    x: r.rtl ? title.x - width - r.tagGap : title.width + r.tagGap
                    anchors.verticalCenter: parent.verticalCenter
                    width: word.implicitWidth + 2 * r.tagGap
                    height: word.implicitHeight
                    radius: height / 2
                    color: "transparent"
                    border.width: r.lk.shape.line.px
                    border.color: r.p.pop_line
                    Accessible.role: Accessible.StaticText
                    Accessible.name: r.tag
                    Label {
                        id: word
                        anchors.centerIn: parent
                        text: r.tag
                        font.family: r.host.ar ? r.res.fam.arabic : r.res.fam.latin
                        font.pixelSize: r.pn.hint_px[0] * r.s * r.scale
                        font.weight: r.lk.shape.chip.weight
                        color: r.lit ? r.lk.sel_ink : r.p.pop_muted
                    }
                }
            }
            Label {
                id: meta
                visible: !!r.line
                width: parent.width
                topPadding: r.pn.hint_px[1] * r.s
                text: r.line
                elide: Text.ElideRight
                horizontalAlignment: r.host.ar ? Text.AlignRight : Text.AlignLeft
                font.family: r.res.fam.ui
                font.pixelSize: r.pn.hint_px[0] * r.s * r.scale
                color: r.lit ? r.lk.sel_ink : r.p.pop_muted
            }
            Column {
                id: below
                width: parent.width
                visible: visibleChildren.length > 0
                topPadding: r.pn.row_pad_px * r.s
            }
        }
        Row {
            id: ends
            anchors.right: parent.right
            anchors.rightMargin: r.pn.row_px[2] * r.s
            anchors.verticalCenter: parent.verticalCenter
            spacing: r.gap
        }
    }
}

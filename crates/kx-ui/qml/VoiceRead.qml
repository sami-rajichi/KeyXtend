// Compare's set text (mock-up compare()): what to do, the text to read aloud in a box, in its language's direction and
// font, then how many models it runs on which device, or the recording's line while the user reads, then its cloud
// models to tick, a page of them at a time with the pager beside; it takes a page of rows' height, so the panel keeps
// its size, or its body's or pager's when too few rows to a page cannot hold them.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: read

    // The panel window, whose view, look and fonts it takes.
    required property var host

    readonly property var sheet: host.view.sheet
    readonly property var lang: host.view.lang
    readonly property var lk: host.lk
    readonly property real s: host.s
    readonly property var pn: lk.shape.panel
    readonly property real side: pn.head_pad_px[3] * s
    readonly property real scale: host.ar ? lk.look.arabic_scale : 1
    readonly property var all: sheet.picks || []
    // Start shows only while the microphone is closed; once it opens the run's plan is fixed, so the picks only show.
    readonly property bool idle: host.view.foot.some(b => b.kind === host.names.acts.compare)
    // The pager's room beside the body while the picks take more than one page.
    readonly property real pager: host.view.pages.count > 1 ? (pn.body_gap_px + pn.page_px[0]) * s : 0
    readonly property alias body: col
    // Picks pair, `across` to a row and each at that share of the body, when at their own widths they take more rows than
    // a full page's picks would paired.
    readonly property int across: lk.shape.voice.pick_across
    readonly property bool pair: rows(flow.children) > Math.ceil(host.per / across)
    // The widest a pick may be: the body, or its share less the gaps while picks pair.
    readonly property real most: pair ? Math.floor((col.width - (across - 1) * flow.spacing) / across) : col.width

    // The rows the picks among `items` take in the body at their own widths, as the Flow lays them.
    function rows(items) {
        let n = 0, used = 0;
        for (let i = 0; i < items.length; i++) {
            if (items[i].natural === undefined)
                continue;
            const w = Math.min(items[i].natural, col.width);
            if (n === 0 || used + flow.spacing + w > col.width) {
                n++;
                used = w;
            } else {
                used += flow.spacing + w;
            }
        }
        return n;
    }

    height: (pn.body_pad_px[0] + pn.body_pad_px[2]) * s + Math.max(host.pageH, col.height, pg.least)

    Column {
        id: col
        x: read.side
        width: read.width - 2 * read.side - read.pager
        anchors.verticalCenter: parent.verticalCenter
        spacing: read.pn.body_gap_px * read.s
        Line {
            text: read.sheet.intro || ""
        }
        Rectangle {
            width: parent.width
            height: big.height + 2 * read.side
            radius: read.lk.shape.clip.note_px[1] * read.s
            color: read.lk.palette.pop_hover
            border.width: read.lk.shape.line.px
            border.color: read.lk.palette.pop_line
            // On pop_hover, which is Highlight in high contrast, so the text takes sel_ink.
            Label {
                id: big
                x: read.side
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 2 * read.side
                text: read.sheet.text || ""
                wrapMode: Text.Wrap
                horizontalAlignment: read.lang.rtl ? Text.AlignRight : Text.AlignLeft
                font.family: read.lang.arabic ? read.host.res.fam.arabic : read.host.res.fam.latin
                font.pixelSize: read.lk.shape.caption.words_px[read.lang.arabic ? 1 : 0] * read.s
                color: read.lk.sel_ink
                Accessible.ignored: false
                Accessible.role: Accessible.StaticText
                Accessible.name: text
            }
        }
        Line {
            text: read.sheet.recording || read.sheet.line || ""
        }
        // The cloud models it may send the recording to, each a tick target; slots stay while the count does, so a
        // tick pops and fades in place.
        Flow {
            id: flow
            visible: picks.count > 0
            width: parent.width
            spacing: read.pn.filter_gap_px * read.s
            layoutDirection: read.host.ar ? Qt.RightToLeft : Qt.LeftToRight
            Repeater {
                id: picks
                model: read.all.length
                delegate: Pick {}
            }
        }
    }

    VoicePager {
        id: pg
        anchors.right: parent.right
        anchors.rightMargin: read.pn.body_pad_px[1] * read.s
        anchors.verticalCenter: parent.verticalCenter
        host: read.host
    }

    // A line about the text, in the panel's muted ink.
    component Line: Label {
        width: parent ? parent.width : 0
        wrapMode: Text.Wrap
        horizontalAlignment: read.host.ar ? Text.AlignRight : Text.AlignLeft
        font.family: read.host.res.fam.ui
        font.pixelSize: read.pn.hint_px[0] * read.s * read.scale
        color: read.lk.palette.pop_muted
        Accessible.ignored: false
        Accessible.role: Accessible.StaticText
        Accessible.name: text
    }
    // A cloud model to tick (no mock-up of its own, so a list row's tick box and ticked fill, VoiceRow, on a filter
    // chip's big target, WCAG 2.5.8): its tick box, name and service's mark; a click ticks it at let-go (ADR-0024). A
    // name too wide for its room loses its middle, then its mark is shortened too, and readers still hear both whole.
    component Pick: Rectangle {
        id: pick
        required property int index
        readonly property var entry: read.all[index] || ({ name: "", service: "", said: "", tick: false })
        readonly property bool on: entry.tick
        readonly property bool hot: read.idle && area.containsMouse
        readonly property bool lit: on || hot
        readonly property var sh: read.pn.filter_px
        // Its width with the whole name and mark, and the room its name and mark share within the widest a pick may be.
        readonly property real natural: label.implicitWidth + box.width + mark.natural + 2 * parts.spacing + 2 * sh[1] * read.s
        readonly property real room: read.most - 2 * sh[1] * read.s - box.width - 2 * parts.spacing

        // Sends the tick click, which the module takes only while the microphone is closed.
        function act() {
            if (read.idle)
                read.host.send(read.host.names.acts.row, "", index);
        }

        width: parts.width + 2 * sh[1] * read.s
        height: sh[0] * read.s
        radius: read.pn.row_px[3] * read.s
        color: on ? read.lk.palette.pop_sel : "transparent"
        border.width: on ? 0 : read.lk.shape.line.px
        border.color: read.lk.palette.pop_line
        onOnChanged: if (on) box.pop()
        Accessible.role: Accessible.CheckBox
        Accessible.name: entry.said
        Accessible.checkable: true
        Accessible.checked: on
        Accessible.id: read.host.res.uid(read.host.names.group, read.host.names.ids.row + index)
        Accessible.onPressAction: pick.act()
        // UI Automation's Toggle on a checkable item asks Qt for this action, not the press.
        Accessible.onToggleAction: pick.act()
        Behavior on color {
            ColourTween {
                move: read.lk.motion.clip.highlight
            }
        }

        Hover {
            lk: read.lk
            radius: pick.radius
            visible: pick.hot
            tint: read.lk.palette.pop_hover
        }
        Row {
            id: parts
            anchors.centerIn: parent
            spacing: pick.sh[3] * read.s
            ClipTick {
                id: box
                anchors.verticalCenter: parent.verticalCenter
                number: pick.on ? 1 : null
                glyph: read.lk.icons.voice.panel.ticked
                mark: pick.hot || (pick.on && read.lk.contrast) ? read.lk.sel_ink : read.lk.palette.ring
                lk: read.lk
                res: read.host.res
                s: read.s
            }
            Label {
                id: label
                // The name keeps all the room the whole mark leaves, but never less than the least a name keeps.
                readonly property real least: Math.min(implicitWidth, read.lk.shape.voice.pick_name_px * read.s)
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(implicitWidth, Math.max(least, pick.room - mark.natural))
                elide: Text.ElideMiddle
                text: pick.entry.name
                font.family: read.host.res.fam.latin
                font.pixelSize: read.lk.shape.chip.font_px[0] * read.s
                font.weight: read.lk.shape.chip.weight
                color: pick.lit ? read.lk.sel_ink : read.lk.palette.pop_ink
            }
            VoiceMark {
                id: mark
                anchors.verticalCenter: parent.verticalCenter
                host: read.host
                mark: ({ id: read.host.names.cloud, text: pick.entry.service })
                lit: pick.lit
                // Capped only once the name is at its least, so a whole mark never loses a letter to rounding.
                most: label.width > label.least ? Infinity : pick.room - label.width
            }
        }
        Act {
            id: area
            anchors.fill: parent
            enabled: read.idle
            hoverEnabled: true
            onActed: pick.act()
        }
    }
}

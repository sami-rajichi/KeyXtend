// The Snip panel's page (snip-editor.js panel()): Capture's keyboard during a capture, with what it does, and the folder
// the Save dialog opens at; or Video's countdown, frame rate, longest video, and the microphone and computer sound at
// start. A key at an end says why it waits.
pragma ComponentBehavior: Bound
import QtQuick

Column {
    id: rows

    // The panel.
    required property var panel

    readonly property var lk: panel.lk
    readonly property var res: panel.res
    readonly property var n: panel.n
    readonly property var p: panel.page
    readonly property real s: panel.s
    readonly property bool capture: !!p && p.tab === n.tab.capture
    readonly property bool video: !!p && p.tab === n.tab.video

    // A click on key `id` of `kind`, with a step's sign in `arg`.
    function act(kind, id, arg) {
        panel.snips.act(kind, id, arg);
    }
    // Why the stepper of `ends` waits at its low end (`least`) or high end.
    function why(ends, least) {
        const at = ends && (least ? ends.low : ends.high);
        return at ? res.tr("snip.why-" + (least ? n.why.least : n.why.most)) : "";
    }

    // The keyboard during a capture: Bubble, Hidden or Shown, with what the one picked does.
    SnipRow {
        visible: rows.capture
        panel: rows.panel
        label: rows.res.tr("snip.panel-row-" + rows.n.page.keyboard)
        say: rows.capture ? rows.res.tr("snip.panel-say-" + rows.p.keyboard) : ""
        Repeater {
            model: rows.n.ids.keyboards
            delegate: SnipChoice {
                required property string modelData
                panel: rows.panel
                name: rows.res.tr("snip.panel-keyboard-" + modelData)
                on: rows.capture && rows.p.keyboard === modelData
                uid: "snip.keyboard." + modelData
                onClicked: rows.act(rows.n.acts.keyboard, modelData)
            }
        }
    }
    // The folder the Save dialog opens at, then Change and Open folder.
    SnipRow {
        visible: rows.capture
        panel: rows.panel
        label: rows.res.tr("snip.panel-row-" + rows.n.page.folder)
        say: !rows.capture ? "" : rows.p.folder || rows.res.tr("snip.panel-folder-none")
        Repeater {
            model: rows.n.ids.folders
            delegate: SnipButton {
                required property string modelData
                lk: rows.lk
                res: rows.res
                s: rows.s
                name: rows.res.tr("snip.panel-folder-" + modelData)
                icon: rows.lk.icons.snip.panel[modelData]
                worded: true
                tone: "ghost"
                uid: "snip.folder." + modelData
                tip: rows.panel.hint
                onClicked: rows.act(rows.n.acts.folder, modelData)
            }
        }
    }
    // The Video page's rows in the hub's order: the countdown and the longest video by − and +, the others by their
    // first or second choice (the frame rates the hub offers, or Off and On).
    Repeater {
        model: rows.n.ids.rows
        delegate: SnipRow {
            id: field
            required property string modelData
            readonly property bool counts: modelData === rows.n.row.countdown
            readonly property bool steps: counts || modelData === rows.n.row.longest
            readonly property bool rate: modelData === rows.n.row.fps
            readonly property var ends: !rows.video || !steps ? null : counts ? rows.p.countdown : rows.p.longest
            readonly property var names: !rows.video || steps ? [] : rate ? rows.p.rates.map(String) : [rows.res.tr("snip.panel-off"), rows.res.tr("snip.panel-on")]
            readonly property bool on: rows.video && (modelData === rows.n.row.mic ? rows.p.mic_on : rows.p.sound_on)
            readonly property int at: !rows.video ? -1 : rate ? rows.p.rates.indexOf(rows.p.fps) : on ? 1 : 0
            visible: rows.video
            panel: rows.panel
            label: rows.res.tr("snip.panel-row-" + modelData)
            SnipStepper {
                visible: field.steps
                lk: rows.lk
                res: rows.res
                s: rows.s
                what: field.modelData
                value: !field.ends ? "" : String(field.counts ? rows.p.countdown_s : rows.p.max_min)
                least: rows.why(field.ends, true)
                most: rows.why(field.ends, false)
                tip: rows.panel.hint
                onStepped: by => rows.act(rows.n.acts.step, field.modelData, by)
            }
            Repeater {
                model: field.names
                delegate: SnipChoice {
                    required property string modelData
                    required property int index
                    panel: rows.panel
                    name: modelData
                    on: field.at === index
                    uid: "snip." + field.modelData + "." + index
                    onClicked: rows.act(rows.n.acts.step, field.modelData, index > 0 ? 1 : -1)
                }
            }
        }
    }
}

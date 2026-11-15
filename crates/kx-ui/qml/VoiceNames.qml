// The names the dictation bar's and the Voice panel's clicks and gate ids use, held once in QML: tst_voicebar checks
// the clicks against the hub's (`kx_ui::voice_acts`); gates find a part by `group` and id.
import QtQuick

QtObject {
    // Each `Voice.act` kind.
    readonly property var acts: ({
            bar: "bar",
            button: "button",
            compare: "compare",
            confirm: "confirm",
            fetch: "fetch",
            flip: "flip",
            gear: "gear",
            minus: "minus",
            open: "open",
            over: "over",
            page: "page",
            plus: "plus",
            row: "row",
            step: "step",
            wheel: "wheel"
        })
    // Each bar state, as the hub's JSON names it.
    readonly property var states: ({
            hidden: "hidden",
            listening: "listening",
            transcribing: "transcribing",
            text: "text",
            note: "note"
        })
    // A button's plain tone, which the gear takes.
    readonly property string plain: "plain"
    // Copy's id, which turns to Copied once clicked.
    readonly property string copy: "copy"
    // The gate-id group of the bar and the panel; a bar button's id is the hub's, so its gate id is its word's text key.
    readonly property string group: "voice"
    // The gear's id, which the hub's buttons never take.
    readonly property string gear: "gear"
}

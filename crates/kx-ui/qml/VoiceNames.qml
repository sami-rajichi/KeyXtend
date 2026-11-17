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
    // Copy's id, which turns to Copied once clicked, and History's Pin, which shows Unpin's icon while it unpins.
    readonly property string copy: "copy"
    readonly property string pin: "pin"
    // The gate-id group of the bar and the panel; a bar button's id is the hub's, so its gate id is its word's text key.
    readonly property string group: "voice"
    // The gear's id, which the hub's buttons never take.
    readonly property string gear: "gear"
    // Each panel page's body, as the hub's JSON names its kind.
    readonly property var sheets: ({
            home: "home",
            models: "models",
            device: "device",
            ready: "ready",
            running: "running",
            results: "results",
            history: "history",
            options: "options",
            folders: "folders"
        })
    // Each option row's kind.
    readonly property var options: ({
            switch: "switch",
            number: "number",
            choice: "choice",
            pair: "pair",
            hint: "hint"
        })
    // The panel button look (ClipButton) each of the hub's tones takes.
    readonly property var tones: ({
            plain: "ghost",
            primary: "enter",
            danger: "danger"
        })
    // The marks in the badge colours and the outlined ones, by id; the rest sit on a chip, and Writing draws its line.
    readonly property var lit: ["used"]
    readonly property var outlined: ["download", "not-here", "other", "failed", "waiting", "run-failed"]
    readonly property string writing: "writing"
    // Each panel part's gate id in `group`; a row's, a stepper's and a download button's take their place after it, and a
    // pair's chips take its minus's and plus's.
    readonly property var ids: ({
            list: "list",
            row: "row-",
            pageBack: "page-back",
            pageOn: "page-on",
            start: "start",
            yes: "yes",
            no: "no",
            minus: "minus-",
            plus: "plus-"
        })
}

// The names Snip's views and clicks use, held once in QML: tst_snip checks the clicks and capture windows against the
// hub's (`kx_ui::snip_acts`); the rest are the view's JSON names.
import QtQuick

QtObject {
    // Each `Snip.act` kind.
    readonly property var acts: ({
            bar: "bar",
            strip: "strip",
            rec: "rec",
            tool: "tool",
            ink: "ink",
            size: "size",
            edit: "edit",
            ask: "ask",
            tab: "tab",
            keyboard: "keyboard",
            step: "step",
            folder: "folder",
            close: "close",
            caret: "caret",
            handle: "handle",
            crop: "crop"
        })
    // The capture windows' ids, each shielded on its own.
    readonly property var capture: ({
            bar: "bar",
            strip: "strip",
            hint: "hint",
            edge: "edge",
            count: "count",
            record: "record"
        })
    // Each stage, as the view names it.
    readonly property var stages: ({
            idle: "idle",
            pick: "pick",
            strip: "strip",
            count: "count",
            record: "record",
            edit: "edit"
        })
    // The editor's, question's and panel's ids by name: the hub's keys, then the ones the views add (`i18n::keys`), the
    // editor's own keys, the Capture page's rows, the steppers and why a key waits.
    readonly property var edit: ({
            undo: "undo",
            redo: "redo",
            close: "close"
        })
    readonly property var keep: ({
            copy: "copy",
            save: "save"
        })
    readonly property var answer: ({
            save: "save",
            drop: "drop",
            back: "back"
        })
    readonly property var tab: ({
            capture: "capture",
            video: "video"
        })
    readonly property var row: ({
            countdown: "countdown",
            fps: "fps",
            longest: "longest",
            mic: "mic",
            sound: "sound"
        })
    readonly property var own: ({
            full: "full",
            back: "back",
            move: "move",
            size: "size"
        })
    readonly property var page: ({
            keyboard: "keyboard",
            folder: "folder"
        })
    readonly property var step: ({
            thick: "thick",
            text: "text",
            countdown: "countdown",
            longest: "longest"
        })
    readonly property var why: ({
            undo: "undo",
            redo: "redo",
            least: "least",
            most: "most"
        })
    // A frame's handles, clockwise from the top-left corner, as the view names them.
    readonly property var handle: ({
            topLeft: "top-left",
            top: "top",
            topRight: "top-right",
            right: "right",
            bottomRight: "bottom-right",
            bottom: "bottom",
            bottomLeft: "bottom-left",
            left: "left"
        })
    // Crop's ✓ and ✗, shown while its frame waits.
    readonly property var crops: ({
            ok: "ok",
            cancel: "cancel"
        })
    // Each list in its row's order, as tst_snip_editor checks them against the hub's.
    readonly property var ids: ({
            tools: ["crop", "pen", "highlighter", "box", "circle", "arrow", "line", "text", "cover", "steps", "eraser"],
            inks: ["red", "yellow", "green", "blue", "black", "white"],
            edits: Object.values(edit),
            keeps: Object.values(keep),
            answers: Object.values(answer),
            tabs: Object.values(tab),
            rows: Object.values(row),
            keyboards: ["bubble", "hidden", "shown"],
            folders: ["change", "open"],
            own: Object.values(own),
            capture: Object.values(page),
            stepped: Object.values(step),
            whys: Object.values(why),
            handles: Object.values(handle),
            crops: Object.values(crops)
        })
    // What each drawing is (`mark.kind`) and each shape, named as the tool that draws it; the tools that draw a path,
    // each with the mark it makes; and the tools that type and crop.
    readonly property var marks: ({
            pen: "pen",
            highlight: "highlight",
            shape: "shape",
            text: "text",
            step: "step"
        })
    readonly property var shapes: ({
            box: "box",
            circle: "circle",
            arrow: "arrow",
            line: "line",
            cover: "cover"
        })
    readonly property var pens: ({
            pen: "pen",
            highlighter: "highlight"
        })
    readonly property string typing: "text"
    readonly property string crop: "crop"
    // The face state's name for Snip's windows while they take the letters (`st.aim`), each ink's colour token, and the
    // colour token of a step's number on it.
    readonly property string aim: "snip"
    readonly property string ink: "snip_ink_"
    readonly property string number: "snip_num_"
    // Where the strip or record bar sits by its part.
    readonly property var spots: ({
            under: "under",
            above: "above",
            bottom: "bottom",
            top: "top"
        })
    // The part edge's sides.
    readonly property var sides: ({
            top: "top",
            bottom: "bottom",
            left: "left",
            right: "right"
        })
    // The capture bar's groups in the spec's order, gaps between: the picture ways and the video ways with their words,
    // one of them picked, the video ways with their red light, then Settings and Close as icons alone.
    readonly property var bar: [
        {
            keys: ["region", "window", "full"],
            worded: true,
            picks: true,
            lit: false
        },
        {
            keys: ["video-region", "video-full"],
            worded: true,
            picks: true,
            lit: true
        },
        {
            keys: ["settings", "close"],
            worded: false,
            picks: false,
            lit: false
        }
    ]
    // The ways a click takes a whole window or screen, the switches, and the keys with a look, a value or a line for
    // readers of their own.
    readonly property string window: "window"
    readonly property var whole: ["full", "video-full"]
    readonly property var switches: ["mic", "sound"]
    readonly property string mic: "mic"
    readonly property string record: "record"
    readonly property string play: "play"
    readonly property string speed: "speed"
    readonly property string stop: "stop"
    readonly property string cancel: "cancel"
    // How the keyboard steps aside for a capture (`Ui.modeJson().aside`).
    readonly property var aside: ({
            bubble: "bubble",
            hidden: "hidden"
        })
}

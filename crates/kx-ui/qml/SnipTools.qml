// A picture's two rows in the editor (snip-editor.js editor(), options()): the eleven tools, Undo and Redo, then Copy
// and Save; below, the picked tool's name, the inks, and its thickness or text size, or Apply ✓ and Cancel ✗ while
// Crop's frame waits with no handle held. A key that waits says why.
pragma ComponentBehavior: Bound
import QtQuick

Column {
    id: tools

    // The editor.
    required property var editor

    readonly property var lk: editor.lk
    readonly property var res: editor.res
    readonly property var sh: lk.shape.snip
    readonly property var n: editor.n
    readonly property var pic: editor.pic
    readonly property var ic: lk.icons.snip
    readonly property var p: lk.palette
    readonly property var pad: sh.editor_pad_px
    readonly property real gap: lk.shape.clip.button_px[5]
    // Text steps its size, any other tool its thickness; the sizes sent, the one picked, and the stepper's id.
    readonly property bool typed: !!pic && pic.tool === n.typing
    readonly property var sizes: editor.snips.sent ? (typed ? editor.snips.sent.text : editor.snips.sent.thick) : []
    readonly property var size: sizes.length > 0 && pic ? sizes[Math.min(pic.size, sizes.length - 1)] : null
    readonly property string stepped: typed ? n.step.text : n.step.thick
    // The picked size in window pixels: the sizes sent are the picture's, at the scale it was taken at.
    readonly property real shown: !size ? 0 : (typed ? size.height : size.pen) / editor.snips.taken
    // Crop's frame waits, so its ✓ and ✗ take the inks' and size's place, as Crop draws with neither (Greenshot's
    // crop). They hide while a handle is held, so a set click that overshoots onto them never cuts, as the strip hides.
    readonly property bool framing: !!pic && !!pic.frame
    readonly property bool held: framing && !!pic.frame.grip
    // The first row's keys with Copy and Save, the second row and its stepper, for the tests: their widths say whether
    // they fit.
    readonly property alias keys: keys
    readonly property alias keeps: keeps
    readonly property alias options: options
    readonly property alias stepper: stepper

    // The tools, Undo and Redo, then Copy and Save at the end.
    Item {
        width: tools.width
        height: tools.sh.editor_rows_px[1]
        Row {
            id: keys
            x: tools.pad[0]
            anchors.verticalCenter: parent.verticalCenter
            spacing: tools.gap
            layoutDirection: Qt.LeftToRight
            Repeater {
                model: tools.n.ids.tools
                delegate: SnipButton {
                    required property string modelData
                    readonly property bool on: !!tools.pic && tools.pic.tool === modelData
                    lk: tools.lk
                    res: tools.res
                    name: tools.res.tr("snip.tool-" + modelData)
                    icon: tools.ic.tools[modelData]
                    tone: on ? "shown" : "flat"
                    checked: on
                    uid: "snip.tool." + modelData
                    tip: tools.editor.hint
                    onClicked: tools.editor.snips.act(tools.n.acts.tool, modelData)
                }
            }
            SnipSep {
                lk: tools.lk
            }
            Repeater {
                model: [tools.n.edit.undo, tools.n.edit.redo]
                delegate: SnipButton {
                    required property string modelData
                    readonly property bool acts: !!tools.pic && tools.pic[modelData]
                    lk: tools.lk
                    res: tools.res
                    name: tools.res.tr("snip.edit-" + modelData)
                    icon: tools.ic.edit[modelData]
                    why: acts ? "" : tools.res.tr("snip.why-" + modelData)
                    uid: "snip.edit." + modelData
                    tip: tools.editor.hint
                    onClicked: tools.editor.snips.act(tools.n.acts.edit, modelData)
                }
            }
        }
        SnipKeeps {
            id: keeps
            anchors.right: parent.right
            anchors.rightMargin: tools.pad[0]
            anchors.verticalCenter: parent.verticalCenter
            editor: tools.editor
        }
    }
    // The tool's name, the inks, and its size (.opts).
    Item {
        width: tools.width
        height: tools.sh.editor_rows_px[2]
        Row {
            id: options
            x: tools.pad[0]
            anchors.verticalCenter: parent.verticalCenter
            spacing: tools.gap
            layoutDirection: Qt.LeftToRight
            Icon {
                anchors.verticalCenter: parent.verticalCenter
                icon: tools.pic ? tools.ic.tools[tools.pic.tool] : ""
                base: tools.res.icons
                ext: tools.res.ext
                px: tools.sh.icon_px
                tint: tools.p.pop_ink
            }
            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: tools.pic ? tools.res.tr("snip.tool-" + tools.pic.tool) : ""
                font.family: tools.res.fam.ui
                font.pixelSize: tools.lk.shape.clip.button_px[3]
                font.weight: tools.lk.shape.clip.button_weight
                color: tools.p.pop_ink
            }
            SnipSep {
                visible: !tools.held
                anchors.verticalCenter: parent.verticalCenter
                lk: tools.lk
            }
            Repeater {
                model: tools.framing && !tools.held ? tools.n.ids.crops : []
                delegate: SnipButton {
                    required property string modelData
                    // The keys go as a handle is picked up or the frame ends, which leaves them without a parent.
                    anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                    lk: tools.lk
                    res: tools.res
                    name: tools.res.tr("snip.crop-" + modelData)
                    icon: tools.ic.crop[modelData]
                    worded: true
                    tone: modelData === tools.n.crops.ok ? "enter" : "ghost"
                    uid: "snip.crop." + modelData
                    tip: tools.editor.hint
                    onClicked: tools.editor.snips.act(tools.n.acts.crop, modelData)
                }
            }
            Label {
                visible: !tools.framing
                anchors.verticalCenter: parent.verticalCenter
                text: tools.res.tr("snip.colour")
                font.family: tools.res.fam.ui
                font.pixelSize: tools.lk.shape.clip.button_px[3]
                color: tools.p.pop_muted
            }
            Row {
                visible: !tools.framing
                anchors.verticalCenter: parent.verticalCenter
                spacing: tools.sh.ink_px[2]
                layoutDirection: Qt.LeftToRight
                Repeater {
                    model: tools.n.ids.inks
                    delegate: SnipInk {
                        required property string modelData
                        editor: tools.editor
                        ink: modelData
                        on: !!tools.pic && tools.pic.ink === modelData
                    }
                }
            }
            SnipSep {
                visible: !tools.framing
                anchors.verticalCenter: parent.verticalCenter
                lk: tools.lk
            }
            Label {
                visible: !tools.framing
                anchors.verticalCenter: parent.verticalCenter
                text: tools.res.tr(tools.typed ? "snip.text-size" : "snip.thick")
                font.family: tools.res.fam.ui
                font.pixelSize: tools.lk.shape.clip.button_px[3]
                color: tools.p.pop_muted
            }
            SnipStepper {
                id: stepper
                visible: !tools.framing
                anchors.verticalCenter: parent.verticalCenter
                lk: tools.lk
                res: tools.res
                what: tools.stepped
                value: tools.size ? String(Math.round(tools.shown)) : ""
                dot: tools.size && !tools.typed ? tools.shown : 0
                least: tools.pic && tools.pic.size <= 0 ? tools.res.tr("snip.why-" + tools.n.why.least) : ""
                most: tools.pic && tools.pic.size >= tools.sizes.length - 1 ? tools.res.tr("snip.why-" + tools.n.why.most) : ""
                tip: tools.editor.hint
                onStepped: by => tools.editor.snips.act(tools.n.acts.size, "", by)
            }
        }
    }
}

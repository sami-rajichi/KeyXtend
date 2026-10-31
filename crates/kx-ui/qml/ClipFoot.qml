// The clipboard panel's footer (spec §7; mock-up .lfoot): Preview, Pin or Unpin and Delete on the ticked rows or the
// previewed entry, and Insert (n) as the strongest button, in Enter's colours. A button with nothing to act on shows
// dimmed, never hidden, so nothing moves; all wait while the Delete-all question shows.
import QtQuick

Item {
    id: foot

    // The panel's view, the click names, the look, fonts, icons and size.
    required property var view
    required property var names
    required property var lk
    required property var res
    required property real s

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    readonly property var pn: lk.shape.panel
    readonly property var icons: lk.icons.clip
    readonly property var words: view.words
    readonly property bool asking: !!view.confirm
    readonly property bool previewing: !!view.preview
    // Pin, Delete and Insert act on the ticked rows, or on the entry the preview page shows.
    readonly property bool targets: !asking && (previewing || view.ticked > 0)
    // Preview opens the last ticked row, else the highlighted one.
    readonly property bool aimed: view.ticked > 0 || view.highlight !== null
    readonly property bool previewable: !asking && !previewing && view.rows.length > 0 && aimed

    height: lk.shape.clip.button_px[0] * s + 2 * pn.foot_pad_px[0] * s

    function uid(id) {
        return res.uid(names.group, id);
    }

    Rectangle {
        width: parent.width
        height: foot.lk.shape.line.px
        color: foot.lk.palette.pop_line
    }
    Row {
        anchors.left: parent.left
        anchors.leftMargin: foot.pn.foot_pad_px[1] * foot.s
        anchors.verticalCenter: parent.verticalCenter
        spacing: foot.lk.shape.clip.foot_gap_px * foot.s
        ClipButton {
            label: foot.words.preview
            icon: foot.icons.preview
            tone: "ghost"
            can: foot.previewable
            lk: foot.lk
            res: foot.res
            s: foot.s
            uid: foot.uid(foot.names.ids.preview)
            onClicked: foot.act(foot.names.acts.preview, 0)
        }
        ClipButton {
            label: foot.words.pin
            icon: foot.view.unpin ? foot.icons.unpin : foot.icons.pin
            tone: "ghost"
            can: foot.targets
            lk: foot.lk
            res: foot.res
            s: foot.s
            uid: foot.uid(foot.names.ids.pin)
            onClicked: foot.act(foot.names.acts.pin, 0)
        }
        ClipButton {
            label: foot.words.delete
            icon: foot.icons.delete
            tone: "danger"
            can: foot.targets
            lk: foot.lk
            res: foot.res
            s: foot.s
            uid: foot.uid(foot.names.ids.delete)
            onClicked: foot.act(foot.names.acts.delete, 0)
        }
    }
    ClipButton {
        anchors.right: parent.right
        anchors.rightMargin: foot.pn.foot_pad_px[1] * foot.s
        anchors.verticalCenter: parent.verticalCenter
        label: foot.words.insert
        icon: foot.icons.insert
        tone: "enter"
        can: foot.targets
        lk: foot.lk
        res: foot.res
        s: foot.s
        uid: foot.uid(foot.names.ids.insert)
        onClicked: foot.act(foot.names.acts.insert, 0)
    }
}

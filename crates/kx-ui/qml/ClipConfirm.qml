// Delete all's question (spec §7), the panels' one question (PanelAsk): it names how many unpinned entries go, and only
// a click on Yes deletes them; No keeps them.
import QtQuick

PanelAsk {
    id: ask

    // The question (`{text, count}`, or null) and the click names.
    required property var confirm
    required property var names

    // A click: its `Ui.clipAct` kind and number.
    signal act(string kind, int arg)

    question: confirm ? confirm.text : ""
    yesId: res.uid(names.group, names.ids.yes)
    noId: res.uid(names.group, names.ids.no)
    icon: lk.icons.clip.delete
    noIcon: lk.icons.clip.cancel
    move: lk.motion.clip.swap
    side: lk.shape.panel.foot_pad_px[1] * s
    onAnswered: y => act(y ? names.acts.yes : names.acts.no, 0)
}

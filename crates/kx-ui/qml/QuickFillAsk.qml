// Quick-fill's one question: the panels' question with the view's Yes and No, its gate ids and move; an answer goes to
// the QuickFill object with the view's round. Delete entry, Remove place, Replace and a foreign box ask with it.
import QtQuick

PanelAsk {
    // The panel's window, whose view, sizes and clicks the question takes.
    required property var box

    yes: box.view.words.yes
    no: box.view.words.no
    yesId: box.res.uid(box.names.group, box.names.ids.yes)
    noId: box.res.uid(box.names.group, box.names.ids.no)
    noIcon: box.lk.icons.fill.cancel
    move: box.lk.motion.fill.sheet
    rtl: box.rtl
    lk: box.lk
    res: box.res
    s: box.s
    onAnswered: agreed => box.act(box.names.acts.answer, "", agreed ? 1 : 0)
}

// One choice of a Snip panel row (snip-editor.js .segs .sp): its word on a key, shown as picked and checked while on.
import QtQuick

SnipButton {
    // The panel, and whether this choice is the one picked.
    required property var panel
    required property bool on

    lk: panel.lk
    res: panel.res
    s: panel.s
    icon: ""
    worded: true
    tone: on ? "shown" : "ghost"
    checked: on
    tip: panel.hint
}

// The keyboard window as Snip's editor and panel read it, for the tests: the pointer its bridge reads in physical
// pixels, the poll time, the panel the letters go to (`st.aim`), the keyboard's size, where a pop-up goes, and its place
// and size, which the Snip panel follows.
import QtQuick

QtObject {
    property point pointer: Qt.point(0, 0)
    property string aim: ""
    property real s: 1
    property point spot: Qt.point(300, 200)
    property point at: Qt.point(0, 0)
    property size drawn: Qt.size(0, 0)

    readonly property var cfg: ({ pollMs: 10 })
    readonly property var st: ({ aim: aim || null })
    readonly property var keyboard: ({ cursor: () => JSON.stringify({ x: pointer.x, y: pointer.y }) })
    readonly property var spots: ({ popSpot: (w, h, left) => ({ x: spot.x, y: spot.y }) })
}

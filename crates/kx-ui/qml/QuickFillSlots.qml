// A Quick-fill page's rows (design "The panel"; mock-up .list): the page size's slots on every page, a short page
// padded so nothing jumps, and the highlight ring gliding to the highlighted row; never a scroll bar (ADR-0008). The
// page gives the row each slot draws.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: list

    // Rows per page, a row's height, how many hold a row, the highlighted place or null, what readers call the list,
    // its gate id, the look and size.
    required property int per
    required property real rowH
    required property int count
    required property var hi
    required property string name
    required property string uid
    required property var lk
    required property real s
    // The row each slot draws, a QuickFillSlot that reads its row by its `index`.
    property alias row: rep.delegate
    // The ring marks the highlighted row; New and Edit's lines each mark themselves.
    property bool ringed: true

    // The highlight ring, for the tests.
    readonly property alias ring: ring
    readonly property real gap: lk.shape.panel.row_px[1] * s
    readonly property bool lit: hi !== null && hi !== undefined && hi < count

    height: per * rowH + (per - 1) * gap

    Column {
        id: col
        width: parent.width
        spacing: list.gap
        Accessible.role: Accessible.List
        Accessible.name: list.name
        Accessible.id: list.uid
        Repeater {
            id: rep
            model: list.per
        }
    }
    // One ring over the rows, so a new highlight glides to its row (motion.toml [fill] highlight).
    Rectangle {
        id: ring
        visible: list.ringed && list.lit
        y: (list.lit ? list.hi : 0) * (list.rowH + list.gap)
        width: col.width
        height: list.rowH
        radius: list.lk.shape.panel.row_px[3] * list.s
        color: "transparent"
        border.width: list.lk.shape.press.ring_px * list.s
        border.color: list.lk.palette.ring
        Behavior on y {
            Tween {
                move: list.lk.motion.fill.highlight
            }
        }
    }
}

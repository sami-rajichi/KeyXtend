// One of a page's row slots (design "The panel"), the base of Quick-fill's rows and fields: what it holds, or null for
// padding, its place, and the page it shows on. A turned page slides it in, and a new row in its place swaps in.
import QtQuick

Item {
    id: cell

    // What the slot holds or null, its place on the page, the page shown, whether it is highlighted, the look and size.
    required property var slot
    required property int place
    required property int at
    required property bool hi
    required property var lk
    required property real s
    // Who the slot holds, so a new view knows whether it moved.
    property string key: ""
    // The slot's parts, which rise in with it.
    default property alias parts: rise.data

    readonly property bool filled: slot !== null
    readonly property string seen: at + "/" + key

    onSeenChanged: move.moved(filled ? key : "", at)
    Component.onCompleted: move.start(filled ? key : "", at)
    Accessible.ignored: !filled
    Accessible.selectable: filled
    Accessible.selected: filled && hi

    RowMove {
        id: move
        rise: rise
        turn: cell.lk.motion.fill.turn
        swap: cell.lk.motion.fill.swap
        lk: cell.lk
        s: cell.s
    }
    Rise {
        id: rise
        anchors.fill: parent
        visible: cell.filled
        move: cell.lk.motion.fill.swap
        from: 0
    }
}

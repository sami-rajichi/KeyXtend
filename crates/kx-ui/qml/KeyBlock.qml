// One block of the board's keys, the main keys or the side block, so a flip of the shortcuts layer makes only the side
// block's keys again; its keys keep the board's order, which readers follow.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: block

    // The plate that draws it (Plate), and its keys in the board's order.
    required property Item plate
    required property var keys

    // A count, not the list: a new size then moves the keys instead of making them all again.
    Repeater {
        model: block.keys.length
        delegate: Key {
            id: cell
            required property int index
            // It is swapping in now.
            readonly property bool swapping: swap.running
            readonly property var body: block.plate
            kv: block.keys[index]
            ks: kv ? body.st.keys[kv.id] : null
            lk: body.lk
            lang: body.st.lang
            paging: body.st.layer
            design: body.board.lang_key
            res: body.res
            s: body.s
            transform: Translate {
                id: rise
            }
            Component.onCompleted: if (body.fresh(kv)) swap.play()
            onKvChanged: if (body.fresh(kv)) swap.play()
            onHit: back => body.win.presses.tap(kv.id, back)
            onHeld: body.win.presses.hold(kv.id)
            onLetGo: body.win.presses.letGo(kv.id)
            onLost: body.win.presses.leave()
            onTip: (name, at) => body.win.hint(name, at)

            Swap {
                id: swap
                target: cell
                shift: rise
                lk: cell.body.lk
                s: cell.body.s
            }
        }
    }
}

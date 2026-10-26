// The words menu in the chips' place on the top bar's row (spec §6 Controls): Forget, Pause or Resume, suggestions off
// or on, and Cancel, each a chip with its icon and short label that sends its ask, closes and pops in once as it opens.
// Its full name is its tip and what readers hear, and short of room, labels drop from the row's end first.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: menu

    // The menu's switches (learning, suggestions), the look, fonts, icons and texts, size, the chips' corner radius, and
    // whether the keyboard's language reads right to left.
    required property bool learn
    required property bool on
    required property var lk
    required property var res
    required property real s
    required property real radius
    required property bool rtl
    // It shows in place of the chips.
    property bool open: false

    // An item's `Ui.wordsAsk` kind and number; the pointer came onto an item (its name) or left it (empty).
    signal ask(string kind, int arg)
    signal tip(string name, Item at)

    readonly property var icons: lk.icons.words
    readonly property var asks: names.asks
    // Each item: its gate id, name and label keys, icon and ask, the switches' with the value a click gives; Cancel only
    // closes, and its label is its name.
    readonly property var items: [
        { id: "forget", key: "words.forget", short: "words.forget-short", icon: icons.forget, kind: asks.forget, arg: 0 },
        learn ? { id: "pause", key: "words.pause", short: "words.pause-short", icon: icons.pause, kind: asks.pause, arg: 1 }
              : { id: "pause", key: "words.resume", short: "words.resume-short", icon: icons.resume, kind: asks.pause, arg: 0 },
        on ? { id: "power", key: "words.off", short: "words.off-short", icon: icons.power, kind: asks.power, arg: 0 }
           : { id: "power", key: "words.on", short: "words.on-short", icon: icons.power, kind: asks.power, arg: 1 },
        { id: "cancel", key: "words.cancel", short: "words.cancel", icon: icons.cancel, kind: "", arg: 0 }
    ]
    // How many items, from the first, show their label; `made` reruns it as items come and go.
    readonly property int labelled: fits(width, rep.count, made)
    property int made: 0

    // The most labels that fit within `room`, the first items' labelled and the rest's icons alone.
    function fits(room) {
        for (let k = rep.count; k > 0; k--) {
            let used = -row.spacing;
            for (let i = 0; i < rep.count; i++) {
                const c = rep.itemAt(i);
                used += row.spacing + (c ? (i < k ? c.full : c.slim) : Infinity);
            }
            if (used <= room)
                return k;
        }
        return 0;
    }

    visible: open
    onOpenChanged: if (open) popIn.restart()

    WordNames {
        id: names
    }

    // What pops in, resting at y 0, scale 1 and opacity 1 as PopIn wants.
    Item {
        id: shelf
        width: menu.width
        height: menu.height
        Row {
            id: row
            anchors.verticalCenter: parent.verticalCenter
            spacing: menu.lk.shape.chip.gap_px * menu.s
            Repeater {
                id: rep
                model: menu.items
                onItemAdded: menu.made++
                onItemRemoved: menu.made++
                delegate: Chip {
                    required property var modelData
                    required property int index
                    chip: ({
                            text: menu.res.tr(modelData.short),
                            name: menu.res.tr(modelData.key),
                            icon: modelData.icon,
                            ar: menu.rtl,
                            uid: menu.res.uid(names.group, names.ids.item + modelData.id)
                        })
                    bare: index >= menu.labelled
                    tipped: true
                    radius: menu.radius
                    lk: menu.lk
                    res: menu.res
                    s: menu.s
                    onTip: (name, at) => menu.tip(name, at)
                    onClicked: {
                        menu.open = false;
                        if (modelData.kind)
                            menu.ask(modelData.kind, modelData.arg);
                    }
                }
            }
        }
    }
    PopIn {
        id: popIn
        target: shelf
        move: menu.lk.motion.moves.panel
        amount: menu.lk.motion.amount
    }
}

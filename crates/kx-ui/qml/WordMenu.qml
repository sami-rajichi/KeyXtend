// The words menu in the chips' place on the top bar's row (spec §6 Controls): Forget a word, Pause or Resume learning,
// suggestions off or on, and Cancel, each a chip that sends its ask and closes the menu. It pops in once as it opens;
// when the texts do not all fit, each item shows its icon alone and its name as a tip.
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

    // An item's `Ui.wordsAsk` kind and number; the pointer came onto an item showing its icon alone (its name) or left.
    signal ask(string kind, int arg)
    signal tip(string name, Item at)

    readonly property var icons: lk.icons.words
    readonly property var asks: names.asks
    // Each item: its gate id, text key, icon and ask, the switches' with the value a click gives; Cancel only closes.
    readonly property var items: [
        { id: "forget", key: "words.forget", icon: icons.forget, kind: asks.forget, arg: 0 },
        learn ? { id: "pause", key: "words.pause", icon: icons.pause, kind: asks.pause, arg: 1 }
              : { id: "pause", key: "words.resume", icon: icons.resume, kind: asks.pause, arg: 0 },
        on ? { id: "power", key: "words.off", icon: icons.power, kind: asks.power, arg: 0 }
           : { id: "power", key: "words.on", icon: icons.power, kind: asks.power, arg: 1 },
        { id: "cancel", key: "words.cancel", icon: icons.cancel, kind: "", arg: 0 }
    ]
    // Every item fits with its text; `made` reruns it as items come and go.
    readonly property bool wide: needs(rep.count, made) <= width
    property int made: 0

    // The room every item takes with its text.
    function needs() {
        let used = -row.spacing;
        for (let i = 0; i < rep.count; i++) {
            const c = rep.itemAt(i);
            used += row.spacing + (c ? c.full : Infinity);
        }
        return used;
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
                    chip: ({
                            text: menu.res.tr(modelData.key),
                            icon: modelData.icon,
                            ar: menu.rtl,
                            uid: menu.res.uid("words", "menu-" + modelData.id)
                        })
                    bare: !menu.wide
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

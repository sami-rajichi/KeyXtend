// The chip lane on the top bar (spec §6 Chips): best first from the left, each chip whole or not at all, and nothing
// animates per key. A password box shows the lock and its note, forget mode ends the row with Cancel, the words menu
// takes the chips' place while open, and readers hear when it has nothing to offer.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: lane

    // The lane as `Ui.words` gives it (null until read), the look, size and corner radius, and whether the lane's own
    // texts read right to left.
    required property var view
    required property var lk
    required property var res
    required property real s
    required property real radius
    required property bool rtl
    // The words menu shows in the chips' place.
    property alias open: menu.open

    // A chip click (its round and place), a menu, list or control tap (its `Ui.wordsAsk` kind and number), a menu tap
    // it handles alone, and a menu item's tip (its name, or empty once the pointer left).
    signal pick(real round, int index)
    signal ask(string kind, int arg)
    signal tapped
    signal tip(string name, Item at)

    // What the lane shows, its chips, forget mode, and whether no module runs, which closes the menu.
    readonly property string kind: view ? view.kind : ""
    readonly property var chips: view ? view.chips : []
    readonly property bool forgetting: !!view && view.forgetting
    readonly property bool stopped: kind === names.stopped
    onStoppedChanged: if (stopped) menu.open = false
    // How many chips fit whole, best first, beside Cancel in forget mode; `made` reruns it as chips come and go.
    readonly property int fit: fits(row.width - (forgetting ? cancel.width + row.spacing : 0), rep.count, made)
    property int made: 0

    // The first chip that would not fit within `room` drops with every chip after it, never cut or squeezed.
    function fits(room) {
        let used = -row.spacing;
        for (let i = 0; i < rep.count; i++) {
            const c = rep.itemAt(i);
            used += row.spacing + (c ? c.width : Infinity);
            if (used > room)
                return i;
        }
        return rep.count;
    }

    Accessible.role: Accessible.Grouping
    Accessible.name: kind === names.empty ? res.tr("chip.none") : ""
    Accessible.id: res.uid(names.group, names.ids.lane)

    WordNames {
        id: names
    }

    Row {
        id: row
        visible: !menu.open
        anchors.fill: parent
        clip: true
        spacing: lane.lk.shape.chip.gap_px * lane.s
        Repeater {
            id: rep
            model: lane.chips
            onItemAdded: lane.made++
            onItemRemoved: lane.made++
            delegate: Chip {
                required property var modelData
                required property int index
                // More and Undo carry no text, so they show their name, in the keyboard's language; a word takes the
                // Arabic font by its script, as the hub tells it.
                readonly property bool named: modelData.text.length === 0
                visible: index < lane.fit
                chip: ({
                        text: named ? modelData.name : modelData.text,
                        icon: lane.lk.icons.words[modelData.kind] || "",
                        ar: named ? lane.rtl : modelData.arabic,
                        name: modelData.name,
                        uid: lane.res.uid(names.group, names.ids.chip + index)
                    })
                forget: lane.forgetting
                radius: lane.radius
                lk: lane.lk
                res: lane.res
                s: lane.s
                // More opens the email list through the hub; every other chip is picked on the lane drawn.
                onClicked: modelData.kind === names.more ? lane.ask(names.asks.more, 0) : lane.pick(lane.view.round, index)
            }
        }
        // Forget mode's Cancel, which ends the row.
        Chip {
            id: cancel
            visible: lane.forgetting
            chip: ({
                    text: lane.res.tr("words.cancel"),
                    icon: lane.lk.icons.words.cancel,
                    ar: lane.rtl,
                    uid: lane.res.uid(names.group, names.ids.cancel)
                })
            radius: lane.radius
            lk: lane.lk
            res: lane.res
            s: lane.s
            onClicked: lane.ask(names.asks.cancel, 0)
        }
        // In a password box, the lock and its note in plate text, not a button (spec §6 Password boxes).
        Item {
            visible: lane.kind === names.locked
            width: row.width
            height: row.height
            Accessible.role: Accessible.StaticText
            Accessible.name: note.text
            Accessible.id: lane.res.uid(names.group, names.ids.note)
            Icon {
                id: padlock
                anchors.verticalCenter: parent.verticalCenter
                icon: lane.lk.icons.words.locked
                base: lane.res.icons
                ext: lane.res.ext
                px: lane.lk.shape.chip.icon_px * lane.s
                tint: lane.lk.palette.legend_2_plate
            }
            Label {
                id: note
                anchors.left: padlock.right
                anchors.leftMargin: row.spacing
                anchors.verticalCenter: parent.verticalCenter
                width: Math.min(implicitWidth, row.width - padlock.width - row.spacing)
                elide: Text.ElideRight
                text: lane.res.tr("chip.locked")
                font.family: lane.res.fam.ui
                font.pixelSize: lane.lk.shape.chip.font_px[lane.rtl ? 1 : 0] * lane.s
                color: lane.lk.palette.legend_2_plate
            }
        }
    }
    WordMenu {
        id: menu
        anchors.fill: parent
        learn: !lane.view || lane.view.learn
        on: !lane.view || lane.view.on
        lk: lane.lk
        res: lane.res
        s: lane.s
        radius: lane.radius
        rtl: lane.rtl
        onAsk: (kind, arg) => lane.ask(kind, arg)
        onTapped: lane.tapped()
        onTip: (name, at) => lane.tip(name, at)
    }
}

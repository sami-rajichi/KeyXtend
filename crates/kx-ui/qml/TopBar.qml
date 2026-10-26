// The top bar: move grip, the chip lane and its menu button, light or dark, the theme, the size buttons, minimise and
// close (mock-up topBar plus the owner's sizes and theme button). It never mirrors: in Arabic only its texts change
// (ADR-0020).
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: bar

    // Its box, the look, fonts, icons and texts, size, the preset shown (null between presets), how many presets there are,
    // the size buttons' texts in the keyboard's language, and the mode.
    required property var box
    required property var lk
    required property var res
    required property real s
    required property var preset
    required property int presets
    required property var sizes
    required property bool dark
    // The chip lane as `Ui.words` gives it (null until read), and whether the keyboard's language reads right to left,
    // as the bar's own texts on chips do.
    property var lane: null
    property bool rtl: false

    // One signal per button; sizes carry their index, a chip click its lane's round and its place, and a menu, list or
    // control chip tap its `Ui.wordsAsk` kind and number. `tapped` is a lane tap the bar handles alone, such as the menu
    // button, which sounds as a key does.
    signal grip
    signal pick(real round, int index)
    signal ask(string kind, int arg)
    signal tapped
    signal flip
    signal theme
    signal smaller
    signal bigger
    signal pickSize(int i)
    signal minimise
    signal close
    // The pointer came onto a button (its name) or left it (empty).
    signal tip(string name, Item at)

    readonly property var sh: lk.shape.bar
    readonly property var p: lk.palette
    readonly property real radius: lk.look.key_radius_px * sh.radius_share
    // The light or dark switch shows what a click gives.
    readonly property string mode: dark ? "mode_on" : "mode"

    x: box.x
    y: box.y
    width: box.w
    height: box.h

    // One top-bar button: `part` names its icon (`icons.toml [bar]`) and its name (`bar.*`); a size shows a letter.
    component Button: Item {
        id: b
        property string part: ""
        property string text: ""
        property string icon: part ? bar.lk.icons.bar[part] : ""
        property string name: bar.res.bar(part)
        // The gates find each button by this id, whatever the language.
        property string uid: part ? bar.res.uid("bar", part) : ""
        property bool lit: false
        readonly property bool hot: area.containsMouse
        // Hover text sits on pop_hover, which is Highlight in high contrast, so it takes sel_ink there; the lit size
        // letter is text in pop_ink, and its mark carries the ring colour.
        readonly property color ink: hot ? bar.lk.sel_ink : lit ? bar.p.pop_ink : bar.p.legend_2_plate
        signal clicked
        width: bar.sh.button_px[0] * bar.s
        height: bar.sh.button_px[1] * bar.s
        Accessible.role: Accessible.Button
        Accessible.name: b.name
        Accessible.id: b.uid
        Accessible.onPressAction: b.clicked()
        Hover {
            lk: bar.lk
            radius: bar.radius
            visible: b.hot
            tint: bar.p.pop_hover
        }
        Icon {
            visible: !b.text
            anchors.centerIn: parent
            icon: b.text ? "" : b.icon
            base: bar.res.icons
            ext: bar.res.ext
            px: bar.sh.icon_px * bar.s
            tint: b.ink
        }
        Label {
            visible: !!b.text
            anchors.centerIn: parent
            text: b.text
            font.family: bar.res.fam.ui
            font.pixelSize: bar.sh.label_px * bar.s
            font.weight: b.lit ? bar.sh.lit_weight : bar.sh.weight
            color: b.ink
        }
        // The lit size's mark: a pill under its letter, in sel_ink on a hovered button in high contrast.
        Rectangle {
            readonly property bool mark: true
            visible: b.lit
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: bar.sh.mark_px[2] * bar.s
            width: bar.sh.mark_px[0] * bar.s
            height: bar.sh.mark_px[1] * bar.s
            radius: height / 2
            color: b.hot && bar.lk.contrast ? bar.lk.sel_ink : bar.p.ring
        }
        // It acts at let-go, wherever the pointer went (ADR-0024).
        Act {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onContainsMouseChanged: bar.tip(containsMouse ? b.name : "", b)
            onActed: {
                bar.tip("", b);
                b.clicked();
            }
        }
    }

    Row {
        id: leftRow
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: bar.sh.gap_px * bar.s
        Button {
            part: "grip"
            onClicked: bar.grip()
        }
    }

    // The chip lane between the grip and the menu button, where the words menu opens in place.
    WordLane {
        id: words
        anchors.left: leftRow.right
        anchors.right: menuButton.left
        anchors.leftMargin: bar.sh.gap_px * bar.s
        anchors.rightMargin: bar.sh.gap_px * bar.s
        height: parent.height
        view: bar.lane
        lk: bar.lk
        res: bar.res
        s: bar.s
        radius: bar.radius
        rtl: bar.rtl
        onPick: (round, index) => bar.pick(round, index)
        onAsk: (kind, arg) => bar.ask(kind, arg)
        onTapped: bar.tapped()
        onTip: (name, at) => bar.tip(name, at)
    }

    // The words menu's button at the lane's right end, gone while no module runs; the menu opens on this row, in the
    // chips' place.
    Button {
        id: menuButton
        visible: !words.stopped
        anchors.right: rightRow.left
        anchors.rightMargin: bar.sh.gap_px * bar.s
        anchors.verticalCenter: parent.verticalCenter
        icon: bar.lk.icons.words.menu
        name: bar.res.tr("words.menu")
        uid: bar.res.uid(names.group, names.ids.menu)
        lit: words.open
        onClicked: {
            words.open = !words.open;
            bar.tapped();
        }
    }
    WordNames {
        id: names
    }

    Row {
        id: rightRow
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: bar.sh.gap_px * bar.s
        Button {
            part: bar.mode
            onClicked: bar.flip()
        }
        Button {
            part: "theme"
            onClicked: bar.theme()
        }
        Button {
            part: "smaller"
            onClicked: bar.smaller()
        }
        // One button per preset; the keyboard's settings allow no more presets than the sizes named here.
        Repeater {
            model: bar.presets
            delegate: Button {
                required property int index
                readonly property var says: bar.sizes[index]
                text: says.label
                name: says.name
                uid: bar.res.uid("size", says.id)
                lit: bar.preset === index
                onClicked: bar.pickSize(index)
            }
        }
        Button {
            part: "bigger"
            onClicked: bar.bigger()
        }
        Button {
            part: "minimise"
            onClicked: bar.minimise()
        }
        Button {
            part: "close"
            onClicked: bar.close()
        }
    }
}

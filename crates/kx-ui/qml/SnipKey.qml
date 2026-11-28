// One keycap of Snip's capture windows (design/snip-parts.css .cb-keys .cw, .strip .key): the keyboard's own cap on the
// plate with its icon, its word and any value after it, a ring while picked or recording a sound, a red light on a
// video way, a sliding line while it waits for a video's copy, and the tick and done word once it acted. A dimmed one
// never acts and says why in its tip; it acts at the press, as keys do.
pragma ComponentBehavior: Bound
import QtQuick

Item {
    id: sk

    // The look and texts; the key's group and id (`bar`, `region`), its word, icon file and whether its word shows.
    required property var lk
    required property var res
    required property string group
    required property string key
    required property string word
    required property string icon
    property bool worded: true
    // What the key is set to, after its word left to right, as a speed's `2×`; empty shows none.
    property string value: ""
    // Picked or recording (a ring), a switch (readers hear it checked), one of many picks (readers hear a radio button),
    // why it is dimmed (a text, empty while it acts), the Enter key's colours, and a video way's red light.
    property bool on: false
    property bool toggle: false
    property bool pick: false
    property string why: ""
    // What readers hear after its name while it acts, as Play's own player; a dimmed key says why instead.
    property string say: ""
    // It waits for the video's copy at its speed: a line slides along its foot, and readers and its tip say so.
    property bool waits: false
    property bool enter: false
    property bool lit: false
    // Shows `text` as a tip over `item`, or hides it with an empty text; tests leave it quiet.
    property var tip: (text, item) => {}

    // A press on it while it acts.
    signal hit

    readonly property var sh: lk.shape.snip
    readonly property var p: lk.palette
    readonly property bool dim: why.length > 0
    readonly property bool down: area.pressed && !dim
    readonly property real pad: worded ? sh.key_pad_px[0] : sh.key_pad_px[1]
    readonly property real need: glyph.width + (worded ? sh.icon_gap_px + text.implicitWidth : 0) + (valued ? sh.icon_gap_px + set.implicitWidth : 0)
    readonly property bool valued: worded && value.length > 0
    // The cap's paint: a dimmed key's, the Enter key's, or an action key's (mock-up --key-act, --legend-act).
    readonly property color fill: dim ? p.dim : enter ? p.enter : p.key_act
    readonly property color ink: down && p.press_ink ? p.press_ink : dim ? p.dim_ink : enter ? p.enter_ink : p.legend_act
    readonly property color skirt: !p.skirt ? fill : dim ? p.skirt.dim : enter ? p.skirt.enter : p.skirt.key_mod
    // What its tip says: the reason a dimmed key waits, that it waits for its copy, or the name of a key that shows its
    // icon alone.
    readonly property string says: dim ? res.noteTip(word, why) : waiting ? res.tr("snip.strip-wait") : named
    readonly property string named: worded ? "" : word
    readonly property bool waiting: waits && !dim
    // The waiting line, for the tests.
    readonly property alias cue: cue
    // The ring's and the red light's colours; pressed, the press text when the look has one, as a legend takes it.
    readonly property color ringInk: down && p.press_ink ? p.press_ink : p.ring_act
    readonly property color lightInk: down && p.press_ink ? p.press_ink : p.rec_light

    width: Math.ceil(need + 2 * pad)
    height: sh.key_px

    Accessible.role: sk.pick ? Accessible.RadioButton : Accessible.Button
    Accessible.name: sk.valued ? sk.word + sk.res.tr("snip.join") + sk.value : sk.word
    Accessible.description: sk.dim ? sk.why : sk.waiting ? sk.says : sk.say
    Accessible.id: res.uid("snip." + sk.group, sk.key)
    Accessible.checkable: sk.toggle || sk.pick
    Accessible.checked: sk.on
    Accessible.onPressAction: sk.act()
    Accessible.onToggleAction: sk.act()

    function act() {
        if (!dim)
            hit();
    }
    // A tip showing follows what it says, as a key under the pointer starts or stops waiting.
    onSaysChanged: if (area.containsMouse) tip(says, sk)

    KeyCap {
        anchors.fill: parent
        lk: sk.lk
        s: 1
        topColour: sk.fill
        skirt: sk.skirt
        pressed: sk.down
        hovered: !sk.dim && area.containsMouse

        Row {
            anchors.centerIn: parent
            spacing: sk.sh.icon_gap_px
            // Left to right in every language: Arabic flips the text only (ADR-0020).
            layoutDirection: Qt.LeftToRight
            Icon {
                id: glyph
                anchors.verticalCenter: parent.verticalCenter
                icon: sk.icon
                base: sk.res.icons
                ext: sk.res.ext
                px: sk.sh.icon_px
                tint: sk.ink
            }
            Label {
                id: text
                visible: sk.worded
                anchors.verticalCenter: parent.verticalCenter
                text: sk.word
                font.family: sk.res.fam.ui
                font.pixelSize: sk.sh.word_px
                font.weight: sk.lk.look.legend_weight
                color: sk.ink
            }
            // Its own Latin label, so `2×` reads left to right in Arabic too.
            Label {
                id: set
                visible: sk.valued
                anchors.verticalCenter: parent.verticalCenter
                text: sk.value
                font.family: sk.res.fam.latin
                font.pixelSize: sk.sh.word_px
                font.weight: sk.lk.look.legend_weight
                color: sk.ink
            }
        }
        // The ring inside a picked way or a sound that records (mock-up [aria-pressed] inset 2px ring).
        Rectangle {
            anchors.fill: parent
            visible: sk.on && !sk.dim
            radius: sk.lk.look.key_radius_px
            color: "transparent"
            border.width: sk.sh.ring_px
            border.color: sk.ringInk
        }
        // A video way's red light, where a key's light sits; its icon and word carry the meaning.
        Rectangle {
            visible: sk.lit && !sk.dim
            readonly property var led: sk.lk.shape.led
            x: parent.width - led.inset_px - width
            y: led.inset_px
            width: led.size_px
            height: width
            radius: width / 2
            color: sk.lightInk
        }
        // While it waits, the voice panel's writing line (`RunLine`) slides along its foot in its ring's colour.
        Item {
            id: cue
            visible: sk.waiting
            x: sk.lk.look.key_radius_px
            y: parent.height - height
            width: parent.width - 2 * x
            height: sk.sh.ring_px
            RunLine {
                lk: sk.lk
                ink: sk.ringInk
            }
        }
    }

    Act {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        atPress: true
        onActed: sk.act()
        onContainsMouseChanged: sk.tip(containsMouse ? sk.says : "", sk)
    }
}

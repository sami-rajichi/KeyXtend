// One key: its shadows, cap, inner shading, legends or icon, LED and pointer feedback. Readers hear its name, and a key
// that comes in a later version is drawn dimmed, never presses and says so.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects

Item {
    id: key

    // The key's cell and kind (board JSON), its changing state, the look, the language key and its design, fonts, icons
    // and texts, and size.
    required property var kv
    required property var ks
    required property var lk
    required property var lang
    required property string design
    required property var res
    required property real s

    // A click; `back` is the language key's left part.
    signal hit(bool back)
    // The pointer came onto a key with a tip (the tip) or left it (empty).
    signal tip(string name, Item at)

    readonly property var p: lk.palette
    readonly property var look: lk.look
    readonly property var press: lk.shape.press
    readonly property var mv: lk.motion.moves
    readonly property bool dim: !!kv.dim
    readonly property bool on: ks ? ks.on : false
    // The recording mic takes the rec paint.
    readonly property bool rec: ks ? ks.paint.fill === "rec" : false
    readonly property bool down: area.pressed && !dim
    // The palette tokens for the cap, legend and skirt, which the face picks from the kind and state.
    readonly property var paint: ks ? ks.paint : ({})
    // Colours ease to each new state or theme (mock-up .cap transition).
    property color fillColour: p[paint.fill]
    property color ink: p[paint.ink]
    property color skirt: p.skirt ? p.skirt[paint.skirt] : p[paint.fill]
    // How far the cap has sunk, 0 to 1, and how far a recording cap has brightened, 0 to 1.
    property real sink: down ? 1 : 0
    property real lift: 0
    readonly property real glowBy: 1 + (lk.motion.amount.glow_lift - 1) * lift
    readonly property color capColour: lift > 0 ? Qt.lighter(fillColour, glowBy) : fillColour
    readonly property color capSkirt: lift > 0 ? Qt.lighter(skirt, glowBy) : skirt
    readonly property real radius: look.key_radius_px
    readonly property color onColour: p.on
    // The icon file the theme gives this key; without one the key shows its label.
    readonly property string icon: iconOf(kv.icon)
    readonly property string label: !icon && kv.label ? res.tr(kv.label) : ""
    readonly property string name: kv.name ? res.tr(kv.name) : (ks ? ks.name : "")
    readonly property string later: dim && kv.tip ? res.tr(kv.tip) : ""
    // The cap sits half a gap inside its cell.
    readonly property real inset: look.gap_px * s / 2
    // The recording glow runs.
    readonly property bool glowing: glow.running

    x: kv.x + inset
    y: kv.y + inset
    width: kv.w - 2 * inset
    height: kv.h - 2 * inset

    Accessible.role: Accessible.Button
    Accessible.name: key.name
    Accessible.description: key.later
    Accessible.checkable: !!kv.checkable
    Accessible.checked: key.on
    Accessible.onPressAction: if (!key.dim) key.hit(false)

    // The file name `icons.toml` gives for `path`, such as `keys.bksp`, or empty.
    function iconOf(path) {
        const [group, id] = path ? path.split(".") : [];
        const set = group ? lk.icons[group] : null;
        return set && set[id] ? set[id] : "";
    }

    Behavior on fillColour {
        ColourTween {
            move: key.mv.colour
        }
    }
    Behavior on ink {
        ColourTween {
            move: key.mv.colour
        }
    }
    Behavior on skirt {
        ColourTween {
            move: key.mv.colour
        }
    }
    Behavior on sink {
        Tween {
            move: key.mv.press
        }
    }
    // A recording cap glows (mock-up .key.rec .cap); loops never run at 0 ms.
    SequentialAnimation on lift {
        id: glow
        running: key.rec && key.mv.glow.ms > 0
        loops: Animation.Infinite
        onStopped: key.lift = 0

        Tween {
            move: key.mv.glow
            share: 0.5
            to: 1
        }
        Tween {
            move: key.mv.glow
            share: 0.5
            to: 0
        }
    }

    // A locked key's glow (Dolch); the recording key has its own.
    RectangularShadow {
        opacity: key.on && !key.rec ? 1 : 0
        visible: opacity > 0 && !key.lk.contrast && key.look.lock_glow_px > 0
        anchors.fill: cap
        blur: key.look.lock_glow_px * key.s
        radius: key.radius
        color: Qt.rgba(key.onColour.r, key.onColour.g, key.onColour.b, key.look.lock_glow_mix)
        Behavior on opacity {
            Tween {
                move: key.mv.colour
            }
        }
    }

    Item {
        id: cap
        width: key.width
        height: key.height
        transform: [
            Scale {
                id: shrink
                origin.x: cap.width / 2
                origin.y: cap.height / 2
                xScale: 1 - (1 - key.press.scale) * key.sink
                yScale: shrink.xScale
            },
            Translate {
                y: key.press.sink_px * key.s * key.sink
            }
        ]

        Repeater {
            model: key.p.key_shadow.filter(l => !l.inset)
            delegate: Shade {
                required property var modelData
                anchors.fill: parent
                shadow: modelData
                radius: key.radius
            }
        }
        CapFace {
            anchors.fill: parent
            cap: key.look.cap
            topColour: key.capColour
            skirt: key.capSkirt
            radius: key.radius
            s: key.s
            pressed: key.down
            sink: key.sink
        }
        Inset {
            anchors.fill: parent
            key: key
        }
        Hover {
            lk: key.lk
            radius: key.radius
            visible: !key.dim && (area.containsMouse || key.down)
            tint: key.down ? key.lk.common.press : key.p.hover
        }
        KeyText {
            anchors.fill: parent
            kv: key.kv
            ks: key.ks
            lk: key.lk
            lang: key.lang
            design: key.design
            res: key.res
            s: key.s
            ink: key.ink
            icon: key.icon
            label: key.label
        }
        Led {
            key: key
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: {
            const says = key.dim ? key.res.dimTip(key.name, key.later) : key.icon ? key.name : "";
            if (says)
                key.tip(containsMouse ? says : "", key);
        }
        onClicked: mouse => {
            if (!key.dim)
                key.hit(key.kv.kind === "lang" && mouse.x < width * key.lk.shape.lang.back_share);
        }
    }
}

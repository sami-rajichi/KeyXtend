// One key: its shadows, cap, inner shading, legends or icon, LED and pointer feedback. Readers hear its name and a note,
// also in its tip: when a dimmed key, which never presses, comes, or that the language key has only one language.
pragma ComponentBehavior: Bound
import QtQuick

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
    // A key that repeats while held (spec §4.6) went down, came up over itself, lost the pointer, or got it back while
    // still down.
    signal held
    signal letGo
    signal slid
    signal reentered
    // The pointer came onto a key with a tip (the tip) or left it (empty).
    signal tip(string name, Item at)

    readonly property var p: lk.palette
    readonly property var look: lk.look
    readonly property var press: lk.shape.press
    readonly property var mv: lk.motion.moves
    readonly property bool dim: !!kv.dim
    // It repeats while held, so the pointer's press and release drive it instead of a click.
    readonly property bool repeats: !!kv.repeat && !dim
    // This press holds the key instead of clicking it, as `repeats` said when the pointer went down.
    property bool holds: false
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
    readonly property string name: kv.name ? res.tr(kv.name) : (ks ? said(ks) : "")
    readonly property string later: dim && kv.tip ? res.tr(kv.tip) : ""
    // What readers hear after its name, also in its tip: when a dimmed key comes, else the state's tip, such as a lone
    // language key's.
    readonly property string note: later || (ks && ks.tip ? res.tr(ks.tip) : "")
    // The cap sits half a gap inside its cell.
    readonly property real inset: look.gap_px * s / 2
    // The recording glow runs.
    readonly property bool glowing: glow.running
    // This key showed a tip, which it takes back on leave even when its note went meanwhile.
    property bool tipped: false
    // How lit the lock glow and the light's glow are, 0 to 1, and the light's colour. They ease here on the key, so a
    // glow or light made only while it shows starts where the key is.
    property real lockLit: on && !rec ? 1 : 0
    property real ledLit: on ? 1 : 0
    property color ledColour: on ? p.led_on : p.led_off

    x: kv.x + inset
    y: kv.y + inset
    width: kv.w - 2 * inset
    height: kv.h - 2 * inset

    Accessible.role: Accessible.Button
    Accessible.name: key.name
    // UI Automation's id is the board id, which never changes with the language, the size or the top bar.
    Accessible.id: key.kv.id
    Accessible.description: key.note
    Accessible.checkable: !!kv.checkable
    Accessible.checked: key.on
    Accessible.onPressAction: if (!key.dim) key.hit(false)
    // UI Automation's Toggle on a latch key asks Qt for this action, not the press.
    Accessible.onToggleAction: if (!key.dim) key.hit(false)

    // What readers hear for a character key in state `st`: its empty name while it types nothing, the names of the marks
    // it types, which they cannot say bare, or else what it types.
    function said(st) {
        if (st.blank)
            return res.tr(st.blank);
        return st.marks.length > 0 ? st.marks.map(m => res.tr(m)).join(res.tr("mark.join")) : st.name;
    }
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
    // A press sinks the cap at once, so the next frame shows it (spec §10); only the rise back eases. It reads the
    // target, since `down` and `sink` change in no set order.
    Behavior on sink {
        enabled: targetValue < key.sink
        Tween {
            move: key.mv.press
        }
    }
    Behavior on lockLit {
        Tween {
            move: key.mv.colour
        }
    }
    Behavior on ledLit {
        Tween {
            move: key.mv.colour
        }
    }
    Behavior on ledColour {
        ColourTween {
            move: key.mv.colour
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

    // A locked key's glow (Dolch), made only while it shows; the recording key has its own.
    Loader {
        anchors.fill: cap
        active: key.lockLit > 0 && !key.lk.contrast && key.look.lock_glow_px > 0
        sourceComponent: Soft {
            opacity: key.lockLit
            blur: key.look.lock_glow_px * key.s
            radius: key.radius
            color: Qt.rgba(key.onColour.r, key.onColour.g, key.onColour.b, key.look.lock_glow_mix)
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
        // The light, made only where it shows: on a lit or recording key, or on every key of a theme with off lights.
        Loader {
            active: key.rec || key.ledLit > 0 || key.ledColour.a > 0
            sourceComponent: Led {
                key: key
            }
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: {
            const says = key.note ? key.res.noteTip(key.name, key.note) : key.icon ? key.name : "";
            if (containsMouse && says)
                key.tip(says, key);
            else if (!containsMouse && key.tipped)
                key.tip("", key);
            key.tipped = containsMouse && !!says;
            if (pressed && key.holds) {
                if (containsMouse)
                    key.reentered();
                else
                    key.slid();
            }
        }
        onPressed: {
            key.holds = key.repeats;
            if (key.holds)
                key.held();
        }
        onReleased: if (key.holds && containsMouse) key.letGo()
        onCanceled: if (key.holds) key.slid()
        onClicked: mouse => {
            if (!key.dim && !key.holds)
                key.hit(key.kv.kind === "lang" && mouse.x < width * key.lk.shape.lang.back_share);
        }
    }
}

// What a key shows, made for its kind alone: character legends, the space bar text, a named key's icon or label, or the
// language key in its design (Carousel, Cycle or List). A new kind or design makes the new part.
pragma ComponentBehavior: Bound
import QtQuick

Loader {
    id: host

    // The key's cell and kind, its changing state, the look, the language key and its design (`board.lang_key`), fonts,
    // icons and texts, size and legend colour.
    required property var kv
    required property var ks
    required property var lk
    required property var lang
    required property string design
    required property var res
    required property real s
    required property color ink
    // A named key's icon file, else its translated label.
    required property string icon
    required property string label

    readonly property var look: lk.look
    readonly property var lg: lk.shape.legend
    readonly property var mv: lk.motion.moves
    readonly property var amt: lk.motion.amount
    readonly property color ink2: lk.palette.legend_2
    // Dolch's sculpted caps lift centred legends to their lighter top's middle (mock-up line 217), and labels fit it.
    readonly property bool sculpted: look.cap.kind === "skirt"
    readonly property real lift: sculpted ? look.cap.legend_lift_px * s : 0
    // The face the theme draws, as insets from the cap (top, right, bottom, left): a sculpted lit top, else the cap.
    readonly property var face: sculpted ? look.cap.top_inset_px : [0, 0, 0, 0]
    readonly property bool isChar: kv.kind === "char"
    readonly property bool isSpace: kv.kind === "space"
    readonly property bool isLang: kv.kind === "lang"

    // The cap hides what slides past its edge (mock-up .cap overflow), as the part asks.
    clip: !!item && item.clips
    sourceComponent: isChar ? charC : isSpace ? spaceC : isLang ? langC : namedC

    Component {
        id: charC
        CharLegends {
            host: host
        }
    }
    Component {
        id: spaceC
        SpaceText {
            host: host
        }
    }
    Component {
        id: namedC
        NamedFace {
            host: host
        }
    }
    Component {
        id: langC
        LangFace {
            host: host
        }
    }
}

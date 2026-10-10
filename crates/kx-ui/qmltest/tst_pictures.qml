// The board as each theme draws it in light mode, in English, French and Arabic plain, with Shift and with AltGr, in the
// bundled fonts as the app picks them; saved as PNGs only where KX_PICTURES points (README), to see faces before install.
pragma ComponentBehavior: Bound
import QtQuick
import QtTest
import "../qml"
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-dolch-light.js" as D

Item {
    id: root

    // The largest size the keyboard allows, so small legends show their details.
    readonly property real s: Face.sizeMax
    // The states pictured, by golden name, each with its layout's language: its texts and direction (ADR-0020).
    readonly property var states: [
        { name: "en", ui: En },
        { name: "fr", ui: Fr },
        { name: "ar", ui: Ar },
        { name: "ar-shift", ui: Ar },
        { name: "ar-altgr", ui: Ar }
    ]
    // The bundled fonts loaded so far, from where the texts test lays them out.
    property int fontsIn: 0

    width: Face.board.plate.w * s
    height: Face.board.plate.h * s

    Instantiator {
        model: Assets.assets.fonts
        delegate: FontLoader {
            required property string modelData
            source: Qt.resolvedUrl(Assets.root + "/" + modelData)
            onStatusChanged: if (status === FontLoader.Ready) root.fontsIn++
        }
    }

    // The plate and every key of the board at size `root.s`, in look `lk` and state `st`.
    Component {
        id: boardC
        Rectangle {
            id: plate
            required property var lk
            required property var st
            required property var res
            width: root.width
            height: root.height
            radius: lk.look.window_radius_px * root.s
            color: lk.palette.plate

            Repeater {
                model: Face.board.keys
                delegate: Key {
                    required property var modelData
                    kv: Object.assign({}, modelData, { x: modelData.x * root.s, y: modelData.y * root.s, w: modelData.w * root.s, h: modelData.h * root.s })
                    ks: plate.st.keys[modelData.id]
                    lk: plate.lk
                    lang: plate.st.lang
                    design: Face.board.lang_key
                    res: plate.res
                    s: root.s
                }
            }
        }
    }

    TestCase {
        name: "pictures"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // Every item under `item` for which `ok` holds.
        function all(item, ok, found) {
            found = found || [];
            if (ok(item))
                found.push(item);
            for (let i = 0; i < item.children.length; i++)
                all(item.children[i], ok, found);
            return found;
        }
        // The folder the texts test handed over from KX_PICTURES, or empty.
        function folder() {
            const c = Qt.createComponent(Qt.resolvedUrl(Assets.root + "/" + Assets.pictures));
            verify(c.status === Component.Ready, "run cargo test -p kx-ui --test texts first");
            const o = c.createObject(root);
            const f = o.folder;
            o.destroy();
            return f;
        }
        // The space bar's text as the hub joins `parts`: each layout after the language through `lang.space`, whose
        // variables the texts test leaves as Fluent shows them, each between bidi isolation marks.
        function spaceOf(texts) {
            const at = name => "{$" + name + "}";
            const join = (lang, layout) => texts["lang.space"].replace(at("lang"), lang).replace(at("layout"), layout);
            return parts => parts.slice(1).reduce(join, parts[0]);
        }
        // Fonts, icons and texts for look `lk` in language `ui`, each family the first Qt has, else the last, and the
        // UI font by the language's direction, as the keyboard window picks.
        function resOf(lk, ui) {
            const have = Qt.fontFamilies();
            const pick = list => list.find(f => have.indexOf(f) >= 0) || list[list.length - 1];
            const [latin, arabic] = [pick(lk.look.latin_fonts), pick(lk.look.arabic_fonts)];
            return {
                fam: { latin: latin, arabic: arabic, mark: pick(lk.look.mark_fonts), ui: ui.lang.rtl ? arabic : latin },
                icons: Qt.resolvedUrl(Assets.root + "/" + Assets.assets.icons),
                ext: Assets.assets.ext,
                tr: k => ui.texts[k] !== undefined ? ui.texts[k] : k,
                space: spaceOf(ui.texts),
                noteTip: (name, note) => name + "\n" + note
            };
        }

        function test_each_theme_pictures_the_board_in_each_language_and_latch() {
            const out = folder();
            if (out === "")
                skip("KX_PICTURES names no folder (crates/kx-ui/README.md)");
            tryCompare(root, "fontsIn", Assets.assets.fonts.length, 5000, "the bundle is laid out by the texts test");
            for (const t of [N, E, D])
                for (const st of root.states) {
                    const props = { lk: t.lk, st: Face.states[st.name], res: resOf(t.lk, st.ui) };
                    const board = boardC.createObject(root, props);
                    const texts = all(board, o => o.isChar === true);
                    tryVerify(() => texts.every(o => !o.clip), 2000, "the legends rose in");
                    waitForRendering(board);
                    grabImage(board).save(out + "/" + t.lk.look.id + "-" + st.name + ".png");
                    board.destroy();
                }
        }
    }
}

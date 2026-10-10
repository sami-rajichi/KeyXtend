// The board in each theme, light and dark, and in Native's high contrast: English, French and Arabic plain, with Shift,
// AltGr and Caps Lock, and with a key pressed, the dim keys in each. Saved where KX_PICTURES points and compared with the
// baseline where KX_PICTURES_BASE points (README), so the key work keeps every pixel.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Window
import QtTest
import "../qml"
import "pointer.js" as Pointer
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as N
import "../../kx-theme/fixtures/look-native-dark.js" as Nd
import "../../kx-theme/fixtures/look-native-light-contrast.js" as Nc
import "../../kx-theme/fixtures/look-et66-light.js" as E
import "../../kx-theme/fixtures/look-et66-dark.js" as Ed
import "../../kx-theme/fixtures/look-dolch-light.js" as D
import "../../kx-theme/fixtures/look-dolch-dark.js" as Dd

Item {
    id: root

    // The largest size the keyboard allows, so small legends show their details.
    readonly property real s: Face.sizeMax
    // The looks pictured: each theme light and dark, and Native's high contrast.
    readonly property var looks: [N, Nd, Nc, E, Ed, D, Dd].map(t => t.lk)
    // The states pictured, by golden name, each with its layout's language: its texts and direction (ADR-0020).
    readonly property var states: [
        { name: "en", ui: En },
        { name: "fr", ui: Fr },
        { name: "ar", ui: Ar },
        { name: "ar-shift", ui: Ar },
        { name: "ar-altgr", ui: Ar },
        { name: "en-caps", ui: En }
    ]
    // The key pictured pressed: the board's first character key, in the English state.
    readonly property string pressed: Face.board.keys.find(k => k.kind === "char").id
    // The bundled fonts, from where the texts test lays them out.
    property var bundled: []

    width: Face.board.plate.w * s
    height: Face.board.plate.h * s

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
    // A saved picture shown pixel for pixel, at the screen's device pixel ratio.
    Component {
        id: savedC
        Image {
            cache: false
            smooth: false
            width: sourceSize.width / Screen.devicePixelRatio
            height: sourceSize.height / Screen.devicePixelRatio
        }
    }
    // A plain box in colour `color`, for the comparison's own check.
    Component {
        id: boxC
        Rectangle {
            width: root.width / 4
            height: root.height / 4
        }
    }

    TestCase {
        name: "pictures"
        when: windowShown

        // The board sits in the test's own window, which the real pointer passes through; the bundled fonts load once.
        function initTestCase() {
            Pointer.through([Window.window]);
            root.bundled = Board.loadFonts(root);
        }
        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The folders the texts test handed over: `folder` (KX_PICTURES), `base` (KX_PICTURES_BASE) and `scratch`.
        function folders() {
            const c = Qt.createComponent(Qt.resolvedUrl(Assets.root + "/" + Assets.pictures));
            verify(c.status === Component.Ready, "run cargo test -p kx-ui --test texts first");
            const o = c.createObject(root);
            const f = { folder: o.folder, base: o.base, scratch: o.scratch };
            o.destroy();
            return f;
        }
        // A file URL for local path `path`.
        function url(path) {
            return "file:///" + path.replace(/^\/+/, "");
        }
        // How pictures `a` and `b` differ: their sizes, or how many pixels, where, and the largest step in any channel.
        function differ(a, b) {
            if (a.width !== b.width || a.height !== b.height)
                return a.width + "×" + a.height + " against " + b.width + "×" + b.height;
            let [n, most, x0, y0, x1, y1] = [0, 0, a.width, a.height, -1, -1];
            for (let y = 0; y < a.height; y++)
                for (let x = 0; x < a.width; x++) {
                    const [p, q] = [a.pixel(x, y), b.pixel(x, y)];
                    const step = Math.max(Math.abs(p.r - q.r), Math.abs(p.g - q.g), Math.abs(p.b - q.b), Math.abs(p.a - q.a));
                    if (step === 0)
                        continue;
                    n++;
                    most = Math.max(most, step);
                    [x0, y0, x1, y1] = [Math.min(x0, x), Math.min(y0, y), Math.max(x1, x), Math.max(y1, y)];
                }
            return n + " pixels differ in " + [x0, y0, x1, y1].join(",") + ", up to " + Math.round(most * 255) + " steps";
        }
        // Empty when picture `shot` matches file `name` in folder `base`, else a note naming the picture.
        function unlike(base, name, shot) {
            const saved = createTemporaryObject(savedC, root, { source: url(base + "/" + name) });
            tryVerify(() => saved.status !== Image.Loading, 5000, name + " loads");
            if (saved.status !== Image.Ready)
                return name + ": no baseline";
            waitForRendering(saved);
            const was = grabImage(saved);
            saved.destroy();
            return was.equals(shot) ? "" : name + ": " + differ(shot, was);
        }
        // The board in look `lk` and state `st` read in language `ui`, with key `down` pressed when given, as a picture.
        function shoot(lk, st, ui, down) {
            // The test's pointer rests on the bare plate, so no key is hovered wherever the real pointer stood.
            mouseMove(root, 1, 1);
            const board = boardC.createObject(root, { lk: lk, st: Face.states[st], res: Board.resOf(lk, ui) });
            const texts = Board.all(board, o => o.isChar === true);
            tryVerify(() => texts.every(o => !o.clip), 2000, "the legends rose in");
            const key = down ? Board.all(board, o => o.kv !== undefined && o.kv.id === down)[0] : null;
            if (key) {
                mousePress(key, key.width / 2, key.height / 2);
                tryCompare(key, "sink", 1, 2000, "the key went down");
            }
            waitForRendering(board);
            const shot = grabImage(board);
            if (key)
                mouseRelease(key, key.width / 2, key.height / 2);
            board.destroy();
            return shot;
        }
        // Each picture's file name, look and state: every state in every look, then the pressed key in each look last,
        // so its pointer never rests over another picture's keys.
        function pictures() {
            const mode = lk => lk.contrast ? "contrast" : lk.dark ? "dark" : "light";
            const name = (lk, what) => [lk.look.id, mode(lk), what].join("-") + ".png";
            const each = root.looks.map(lk => root.states.map(st => ({ file: name(lk, st.name), lk: lk, st: st.name, ui: st.ui })));
            const down = root.looks.map(lk => ({ file: name(lk, "pressed"), lk: lk, st: "en", ui: En, down: root.pressed }));
            return [].concat(...each, down);
        }

        function test_a_picture_unlike_its_baseline_is_named() {
            const out = folders().scratch;
            const box = createTemporaryObject(boxC, root, { color: N.lk.palette.plate });
            waitForRendering(box);
            grabImage(box).save(out + "/box.png");
            compare(unlike(out, "box.png", grabImage(box)), "", "a picture matches itself");
            box.color = N.lk.palette.ring;
            waitForRendering(box);
            const note = unlike(out, "box.png", grabImage(box));
            verify(note.startsWith("box.png: ") && note.indexOf(" pixels differ") > 0, note);
            // Qt warns of a missing file, which fails the picture test at that picture.
            ignoreWarning(/Cannot open: .*\/none\.png$/);
            compare(unlike(out, "none.png", grabImage(box)), "none.png: no baseline");
        }

        function test_each_look_pictures_the_board_in_each_state() {
            const f = folders();
            if (f.folder === "" && f.base === "")
                skip("neither KX_PICTURES nor KX_PICTURES_BASE names a folder (crates/kx-ui/README.md)");
            tryVerify(() => Board.loaded(root.bundled), 5000, "the bundle is laid out by the texts test");
            const notes = [];
            for (const p of pictures()) {
                const shot = shoot(p.lk, p.st, p.ui, p.down);
                // Compared before it is saved, so even a run saving over its baseline meets the old picture.
                const note = f.base === "" ? "" : unlike(f.base, p.file, shot);
                if (f.folder !== "")
                    shot.save(f.folder + "/" + p.file);
                if (note !== "") {
                    // QtTest cuts a long failure message, so each note is also logged whole.
                    console.log(note);
                    notes.push(note);
                }
            }
            verify(notes.length === 0, notes.length + " pictures differ from the baseline: " + notes.join("; "));
        }
    }
}

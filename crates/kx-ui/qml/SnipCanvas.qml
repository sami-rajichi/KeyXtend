// The editor's picture (design "The editor"): the part Crop keeps, with Crop's frame over it while one waits, and its
// drawings in the picture's own pixels, scaled to fit, at most at its own size. A click goes to the hub as a press,
// ends a pen's path, or moves the caret in the text being typed; Copy and Save flatten the kept part at its own pixels.
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes

Item {
    id: canvas

    // The editor.
    required property var editor

    readonly property var snips: editor.snips
    readonly property var lk: editor.lk
    readonly property var res: editor.res
    readonly property var sh: lk.shape.snip
    readonly property var n: editor.n
    readonly property var ed: editor.ed
    readonly property var pic: editor.pic
    readonly property var sent: snips.sent
    readonly property var whole: ed ? { x: 0, y: 0, w: ed.width, h: ed.height } : { x: 0, y: 0, w: 1, h: 1 }
    // Flattening grabs the kept part at its own pixels.
    property bool flattening: false
    // The picture is in, or failed. The hub's synchronous image provider serves it in the turn its view comes, so this
    // never makes the editor wait, and a picture that does load later keeps the editor hidden until it is in.
    readonly property bool loaded: picture.status !== Image.Loading
    readonly property var shown: pic ? pic.crop : whole
    // Window pixels a picture pixel: it fits, at most at the picture's own size on the screen it was taken on.
    readonly property real k: Math.max(0, Math.min(width / shown.w, height / shown.h, 1 / snips.taken))
    // One window pixel in the picture's pixels, for outlines that stay thin at any scale.
    readonly property real line: k > 0 ? lk.shape.line.px / k : lk.shape.line.px
    // The pointer over the picture in its pixels (null while elsewhere), and a pen's path since its first click.
    property var pointer: null
    property var path: []
    // The size in picture pixels a flatten grabs, one to an image pixel.
    property size grabbing: Qt.size(0, 0)
    // Tells a listener each flattened picture; nothing here keeps it, so its pixels go once the hub has them.
    signal flattened(var grab)
    // The tool draws a path, and the first click it awaits a second for, by value so a new view keeps the path.
    readonly property bool pens: !!pic && n.pens[pic.tool] !== undefined
    readonly property string pending: pic && pic.pending ? pic.pending.x + "," + pic.pending.y : ""
    // What a second click would draw: a pen's path or a shape to the pointer, with the picked ink and size.
    readonly property var preview: {
        if (!pending || !pointer)
            return null;
        const [t, from] = [pic.tool, pic.pending];
        const mark = pens ? { kind: n.pens[t], points: [from].concat(path, [pointer]) } : { kind: n.marks.shape, shape: t, from: from, to: pointer };
        return pens || Object.values(n.shapes).indexOf(t) >= 0 ? { mark: mark, ink: pic.ink, size: pic.size } : null;
    }
    // The text being typed as a drawing.
    readonly property var typing: pic && pic.typing ? { mark: Object.assign({ kind: n.marks.text }, pic.typing), ink: pic.ink, size: pic.size } : null
    // A drawing of nothing, for a hidden mark.
    readonly property var none: ({ mark: { kind: "" }, ink: n.ids.inks[0], size: 0 })
    // The drawings, the text being typed, the picture shown and its image, for the tests.
    readonly property alias typer: typer
    readonly property alias marks: marks
    readonly property alias view: view
    readonly property alias picture: picture

    onPendingChanged: path = []

    // Item `i` of size list `list`, its last past its end; null for an empty list.
    function step(list, i) {
        return list.length > 0 ? list[Math.min(i, list.length - 1)] : null;
    }
    // The fewest whole Qt units whose grab at scale `d`, rounded as Qt rounds it, holds `len` picture pixels.
    function pixels(len, d) {
        let t = Math.max(1, Math.floor(len / d));
        while (Math.round(t * d) < len)
            t++;
        return t;
    }
    // The pointer moved to (`x`, `y`) over the picture shown; a pen's path takes each new point.
    function moved(x, y) {
        pointer = { x: shown.x + x / k, y: shown.y + y / k };
        if (pens && pending)
            path = path.concat([pointer]);
    }
    // A click at `at` on the picture shown, sent in window pixels from the whole picture's corner.
    function click(at) {
        const [x, y] = [at.x + shown.x * k, at.y + shown.y * k];
        if (typing && typer.words.contains(view.mapToItem(typer.words, at.x, at.y))) {
            const p = view.mapToItem(typer.words, at.x, at.y);
            snips.act(n.acts.caret, "", typer.words.positionAt(p.x, p.y));
            return;
        }
        if (pens && pending) {
            const points = path.concat([{ x: x / k, y: y / k }]).map(p => [p.x * k, p.y * k]);
            path = [];
            snips.snip.stroke(JSON.stringify(points), k, snips.round);
            return;
        }
        snips.snip.press(x, y, k, snips.round);
    }
    // Flattens the kept part with its drawings, one picture pixel to an image pixel, and hands it on for Copy or Save
    // `keep`. A grab comes in whole pixels of the window's scale, so it may hold extra pixels, which the hub cuts off;
    // one that cannot start, as on a hidden window, leaves the picture as it was.
    function flatten(keep) {
        const d = canvas.Window.window ? canvas.Window.window.devicePixelRatio : 1;
        const t = Qt.size(pixels(pic.crop.w, d), pixels(pic.crop.h, d));
        grabbing = Qt.size(Math.round(t.width * d), Math.round(t.height * d));
        flattening = true;
        const started = sheet.grabToImage(result => {
            flattening = false;
            snips.snip.flat(keep, result.image, snips.round);
            canvas.flattened(result);
        }, t);
        if (!started)
            flattening = false;
    }

    Rectangle {
        anchors.fill: parent
        color: canvas.lk.palette.plate
    }
    Item {
        id: view
        anchors.centerIn: parent
        width: canvas.shown.w * canvas.k
        height: canvas.shown.h * canvas.k
        clip: true

        Item {
            id: sheet
            width: canvas.flattening ? canvas.grabbing.width : canvas.shown.w
            height: canvas.flattening ? canvas.grabbing.height : canvas.shown.h
            clip: true
            transform: Scale {
                xScale: canvas.k
                yScale: canvas.k
            }
            Item {
                x: -canvas.shown.x
                y: -canvas.shown.y
                Image {
                    id: picture
                    width: canvas.whole.w
                    height: canvas.whole.h
                    source: canvas.ed ? canvas.ed.src : ""
                    cache: false
                    smooth: true
                }
                Repeater {
                    id: marks
                    model: canvas.pic ? canvas.pic.drawings : []
                    delegate: SnipMark {
                        required property var modelData
                        canvas: canvas
                        drawn: modelData
                    }
                }
                SnipMark {
                    objectName: "next"
                    visible: !!canvas.preview && !canvas.flattening
                    canvas: canvas
                    drawn: canvas.preview || canvas.none
                }
                SnipMark {
                    id: typer
                    visible: !!canvas.typing
                    canvas: canvas
                    drawn: canvas.typing || canvas.none
                    caret: canvas.typing ? canvas.typing.mark.caret : -1
                }
            }
        }
    }
    SnipCrop {
        id: crop
        objectName: "crop"
        anchors.fill: parent
        canvas: canvas
    }
    // Named for readers by the tool a click uses. While Crop's frame waits it reaches half a handle past the picture,
    // within the padding, and while a handle is held it covers the canvas and that band, so a set click always lands.
    Act {
        id: act
        objectName: "press"
        readonly property real out: crop.waiting ? Math.min(canvas.sh.handle_hit_px / 2, canvas.sh.editor_pad_px[1]) : 0
        anchors.fill: crop.grip ? canvas : view
        anchors.margins: -act.out
        enabled: !!canvas.pic
        hoverEnabled: true
        cursorShape: crop.cursorAt(act.x + act.mouseX, act.y + act.mouseY, Qt.ArrowCursor)
        Accessible.role: Accessible.Graphic
        Accessible.name: canvas.pic ? canvas.res.tr("snip.tool-" + canvas.pic.tool) : ""
        onActed: at => canvas.click(act.mapToItem(view, at.x, at.y))
        onPositionChanged: mouse => {
            const p = act.mapToItem(view, mouse.x, mouse.y);
            canvas.moved(p.x, p.y);
        }
        onExited: canvas.pointer = null
    }
}

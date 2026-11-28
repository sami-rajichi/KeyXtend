// The hub's Snip object for the tests: the view a test sets as `page` (null for none), which it tells as the hub does,
// every call it heard in order, and the editor's places in physical pixels: where it opens, the box under a maximised
// one, the least it shrinks to and the work area it fills.
import QtQuick

QtObject {
    property var page: null
    property var calls: []
    // Where the editor opens (null for no screen), the box it comes back to when it opens maximised (null when it opens
    // unmaximised), the least size it keeps, and its screen's work area (null once gone). Plain test data: the editor
    // never reads the theme's least, only the hub's answers, so these need not match it.
    property var place: ({ x: 100, y: 80, w: 980, h: 540 })
    property var back: null
    property var least: ({ w: 740, h: 420 })
    property var work: ({ x: 0, y: 0, w: 1920, h: 1040 })

    // A new view: read `view` again.
    signal viewChanged

    onPageChanged: viewChanged()

    function note(call) {
        calls = calls.concat([call]);
    }
    // The calls of kind `what`: `act`, `spot`, `press`, `stroke`, `fit`, `flat`, `shield`, `make`, `open`, `order`,
    // `opened`, `draw`, `counted`, `placed` or `full`.
    function heard(what) {
        return calls.filter(c => c[what] !== undefined);
    }
    function view() {
        return JSON.stringify(page);
    }
    function act(kind, id, arg, round) {
        note({ act: kind, id: id, arg: arg, round: round });
    }
    function spot(screen, x, y, round) {
        note({ spot: screen, x: x, y: y, round: round });
    }
    function press(x, y, scale, round) {
        note({ press: [x, y], scale: scale, round: round });
    }
    function stroke(points, scale, round) {
        note({ stroke: JSON.parse(points), scale: scale, round: round });
    }
    function fit(json) {
        note({ fit: JSON.parse(json) });
        return true;
    }
    function flat(keep, image, round) {
        note({ flat: keep, image: image, round: round });
    }
    function shield(window, title) {
        note({ shield: window, title: title });
        return true;
    }
    function make(title) {
        note({ make: title });
    }
    function open() {
        note({ open: true });
    }
    function order(title) {
        note({ order: title });
    }
    function opened(stage, click, shown, drawn, ready) {
        note({ opened: stage, click: click, shown: shown, drawn: drawn, ready: ready });
    }
    function draw(title) {
        note({ draw: title });
        return true;
    }
    function counted(left, draws, held, screens) {
        note({ counted: left, draws: draws, held: held, screens: screens });
    }
    // The editor's places, as the hub keeps it on its screen and no smaller than its least; with no screen, none.
    function kept(x, y, w, h) {
        if (!work)
            return null;
        const [ww, hh] = [Math.min(Math.max(w, least.w), work.w), Math.min(Math.max(h, least.h), work.h)];
        const at = (v, len, from, room) => Math.max(from, Math.min(v, from + room - len));
        return { x: at(x, ww, work.x, work.w), y: at(y, hh, work.y, work.h), w: ww, h: hh };
    }
    function editorAt() {
        return JSON.stringify(place ? { place: place, back: back } : null);
    }
    function editorFit(x, y, w, h) {
        return JSON.stringify(kept(x, y, w, h));
    }
    // A move, a resize or a restore, kept unmaximised for next time.
    function editorPlaced(x, y, w, h) {
        const r = kept(x, y, w, h);
        if (r) {
            note({ placed: r });
            [place, back] = [r, null];
        }
        return JSON.stringify(r);
    }
    // A maximise, kept with the box under it for next time.
    function editorFull(x, y, w, h) {
        const r = kept(x, y, w, h);
        if (r) {
            note({ full: work, from: r });
            [place, back] = [work, r];
        }
        return JSON.stringify(r ? work : null);
    }
}

// A stand-in for the hub's Voice object: the view and level as the hub writes them, where the hub places the bar for
// each place and the Voice panel, and every click, sound, open and place ask QML sends.
import QtQuick

QtObject {
    // The view as the hub writes it, null before any; setting it sends `viewChanged`.
    property var page: null
    // The sound level, 0 to 1; setting it sends `levelChanged`.
    property real heard: 0
    // Where `barAt` puts the bar's top-left for each place, in physical pixels; with the keyboard hidden it goes to the
    // bottom spot, as the hub puts it at the bottom centre of the main screen.
    property var spots: ({ above: { x: 300, y: 500 }, bottom: { x: 300, y: 900 }, top: { x: 300, y: 10 } })
    // Each `barAt` ask as `{over, w, h}`, and each call as `act kind id arg round`, `chime true|false` or `open`.
    property var asks: []
    // Where `panelAt` puts the Voice panel, `{x, y, fit}` in physical pixels, and each ask as `{over, bar, w, h}`.
    property var panelSpot: ({ x: 300, y: 100, fit: 1 })
    property var panelAsks: []
    property var calls: []
    // How often QML read the level.
    property int reads: 0

    // A new view: read `view` again.
    signal viewChanged
    // A new level: read `level` again.
    signal levelChanged

    onPageChanged: viewChanged()
    onHeardChanged: levelChanged()

    function view() {
        return JSON.stringify(page);
    }
    function level() {
        reads++;
        return heard;
    }
    function act(kind, id, arg, round) {
        calls = calls.concat(["act " + kind + " " + id + " " + arg + " " + round]);
    }
    function chime(start) {
        calls = calls.concat(["chime " + start]);
    }
    function open() {
        calls = calls.concat(["open"]);
    }
    function barAt(over, w, h) {
        const box = JSON.parse(over);
        asks = asks.concat([{ over: box, w: w, h: h }]);
        const place = box && page ? page.place : "bottom";
        return JSON.stringify(spots[place]);
    }
    function panelAt(over, bar, w, h) {
        panelAsks = panelAsks.concat([{ over: JSON.parse(over), bar: JSON.parse(bar), w: w, h: h }]);
        return JSON.stringify(panelSpot);
    }
}

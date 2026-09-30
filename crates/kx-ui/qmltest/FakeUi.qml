// Stands in for the hub's Ui object: it answers from the hub's fixtures and records the calls QML makes.
// qmltestrunner cannot load KeyXtend.Ui, which is linked into Rust binaries.
import QtQuick
import "fixtures/ui-en.js" as En
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as Look

QtObject {
    // What the hub gives: settings and texts in one language, and the look; setting one sends its change signal.
    property var settings: En.settings
    property var texts: En.texts
    property var look: Look.lk
    // The backdrop answer, and every call so far.
    property bool frosted: false
    property var calls: []
    // Notices the hub holds for the banner, the banner's counts by how many wait, and the tray picture's side.
    property var notices: []
    property var more: En.more
    property int trayPixels: 16
    // Where `popAt` puts every pop-up, in physical pixels.
    property point popSpot: Qt.point(40, 30)

    // A second copy asks to show the keyboard; the app ended the UI.
    signal showAsked
    signal quitAsked
    // Notices wait; the tray icon was clicked at (x, y), or another window came to the front.
    signal noticesWaiting
    signal trayClicked(int x, int y)
    signal trayAway

    function note(call) {
        calls = calls.concat([call]);
    }
    function settingsJson() {
        return JSON.stringify(settings);
    }
    function lookJson() {
        return JSON.stringify(look);
    }
    // The bundle as the app holds it, laid out by the texts test.
    function assets() {
        const at = rel => Qt.resolvedUrl(Assets.root + "/" + rel).toString();
        const a = Assets.assets;
        return JSON.stringify({ icons: at(a.icons), ext: a.ext, fonts: a.fonts.map(at) });
    }
    function text(key) {
        return texts[key] !== undefined ? texts[key] : key;
    }
    function nameOf(group, id) {
        const key = group + "." + id;
        return texts[key] !== undefined ? texts[key] : id;
    }
    function space(parts) {
        return JSON.parse(parts).join(" · ");
    }
    function flipMode() {
        note("flipMode");
        return true;
    }
    function stepTheme() {
        note("stepTheme");
        return true;
    }
    function quit() {
        note("quit");
        return true;
    }
    // Hands over the notices waiting, as the hub does once.
    function takeNotices() {
        const taken = notices;
        notices = [];
        return JSON.stringify(taken);
    }
    function retry(id) {
        note("retry " + id);
        return true;
    }
    function dismiss(id) {
        note("dismiss " + id);
    }
    function moreText(n) {
        return more[n] !== undefined ? more[n] : String(n);
    }
    // Fluent fills the page and the count into `page.of`, each between isolation marks the fixture keeps.
    function pageText(at, count) {
        return text("page.of").replace("{$at}", at).replace("{$count}", count);
    }
    function trayPx() {
        return trayPixels;
    }
    function trayShow(picture) {
        note("trayShow");
        return true;
    }
    function trayHide() {
        note("trayHide");
    }
    function popAt(over, left, centred, w, h) {
        note("popAt " + (JSON.parse(over) ? "over" : "home") + " " + (centred ? "centred" : "left"));
        return JSON.stringify({ x: popSpot.x, y: popSpot.y });
    }
    // A menu goes up from the click, as the hub puts it.
    function menuAt(x, y, w, h) {
        note("menuAt " + x + " " + y);
        return JSON.stringify({ x: x, y: y - h });
    }
    function backdrop(title) {
        note("backdrop " + title);
        return frosted;
    }
}

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

    // A second copy asks to show the keyboard; the app ended the UI.
    signal showAsked
    signal quitAsked

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
    function backdrop(title) {
        note("backdrop " + title);
        return frosted;
    }
}

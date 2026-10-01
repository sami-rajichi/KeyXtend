// Stands in for the hub's Ui object: it answers from the hub's fixtures and records the calls QML makes.
// qmltestrunner cannot load KeyXtend.Ui, which is linked into Rust binaries.
import QtQuick
import "fixtures/ui-en.js" as En
import "fixtures/assets.js" as Assets
import "../../kx-theme/fixtures/look-native-light.js" as Look

QtObject {
    // What the hub gives: settings, the keyboard's language and texts, the tray's texts, and the look; setting one sends
    // its change signal. Setting `texts` last switches the keyboard's language as the hub does, by `textsChanged`.
    property var settings: En.settings
    property var lang: En.lang
    property var texts: En.texts
    property var trayTexts: En.texts
    property var look: Look.lk
    // The backdrop answer, and every call so far.
    property bool frosted: false
    property var calls: []
    // Notices the hub holds for the banner, those it shows told again in a new language, the banner's counts by how
    // many wait, and the tray picture's side.
    property var notices: []
    property var retold: []
    property var more: En.more
    property int trayPixels: 16
    // Where `popAt` puts every pop-up, in physical pixels.
    property point popSpot: Qt.point(40, 30)
    // The keyboard's language and texts as the calls read them, copied on each change where no binding sees it, as the
    // real object's calls are not tracked: QML must read them again on `textsChanged`.
    readonly property var said: ({ lang: En.lang, texts: En.texts })

    onLangChanged: said.lang = lang
    onTextsChanged: said.texts = texts
    Component.onCompleted: {
        said.lang = lang;
        said.texts = texts;
    }

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
    function langJson() {
        return JSON.stringify(said.lang);
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
        return said.texts[key] !== undefined ? said.texts[key] : key;
    }
    function trayText(key) {
        return trayTexts[key] !== undefined ? trayTexts[key] : key;
    }
    function nameOf(group, id) {
        const key = group + "." + id;
        return said.texts[key] !== undefined ? said.texts[key] : id;
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
    function shownNotices() {
        return JSON.stringify(retold);
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

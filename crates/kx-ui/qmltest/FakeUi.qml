// Stands in for the hub's Ui object: it answers from the hub's fixtures and records the calls QML makes.
// qmltestrunner cannot load KeyXtend.Ui, which is linked into Rust binaries.
import QtQuick
import QtQml.Models
import "fixtures/ui-en.js" as En
import "fixtures/assets.js" as Assets
import "fixtures/words.js" as Words
import "../../kx-theme/fixtures/look-native-light.js" as Look

QtObject {
    id: stub

    // What the hub gives: settings, the keyboard's language and texts, the tray's texts, the look and the pointer
    // overlay's state (the tests set its `mode` as the hub names it); setting one sends its change signal. Setting
    // `texts` last switches the keyboard's language as the hub does (`textsChanged`).
    property var settings: En.settings
    property var lang: En.lang
    property var texts: En.texts
    property var trayTexts: En.texts
    property var look: Look.lk
    property var assist: En.assist
    // How often QML read the overlay's state, the newest state a read returns, and whether a `change` wake waits for a
    // read, as the hub keeps it.
    property int assistReads: 0
    property var newest: En.assist
    property bool waiting: false
    // The scroll pad's state and its box to outline (setting it sends `scrollChanged`), how often QML read it, and how
    // many more entry watch or page mark calls answer that the keyboard does not show or the list is not guarded yet.
    property var scroll: En.scroll
    property int scrollReads: 0
    property int notYet: 0
    property int unpaged: 0
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
    // The room above the keyboard, in Qt units, that `popFit` shrinks a pop-up panel into; endless by default.
    property real fitRoom: Infinity
    // The keyboard's centre that `popFit` centres a panel on, in physical pixels; NaN keeps it at `popSpot`. The sizes
    // `popFit` was asked for, `{w, h}` in Qt units.
    property real fitCentre: NaN
    property var fitAsks: []
    // The keyboard's mode the hub keeps, as `keepMode` last set it.
    property var mode: ({ away: false, minimised: false })
    // The chip lane as the hub writes it, the start's until a test sets one; setting it sends `wordsChanged`. How many
    // times QML read it, counted where no binding sees it.
    property var lane: Words.lanes.start
    readonly property var reads: ({ words: 0 })
    // The keyboard's language and texts as the calls read them, copied on each change where no binding sees it, as the
    // real object's calls are not tracked: QML must read them again on `textsChanged`.
    readonly property var said: ({ lang: En.lang, texts: En.texts })
    // The clipboard panel's view as the hub writes it, null while closed; setting it sends `clipChanged`.
    property var clipView: null
    // The round each clipboard click named, oldest first.
    property var clipRounds: []

    onLangChanged: said.lang = lang
    onTextsChanged: said.texts = texts
    onAssistChanged: newest = assist
    onLaneChanged: wordsChanged()
    onClipViewChanged: clipChanged()
    Component.onCompleted: {
        said.lang = lang;
        said.texts = texts;
    }

    // The hub loads the theme's bundled fonts before QML picks its families; the stand-in loads every bundled file and
    // says so as each comes in.
    property Instantiator loaders: Instantiator {
        model: Assets.assets.fonts
        delegate: FontLoader {
            required property string modelData
            source: Qt.resolvedUrl(Assets.root + "/" + modelData)
            onStatusChanged: if (status === FontLoader.Ready) stub.fontsChanged()
        }
    }
    signal fontsChanged

    // A second copy asks to show the keyboard; the app ended the UI.
    signal showAsked
    signal quitAsked
    // Notices wait; the tray icon was clicked at (x, y), or another window came to the front.
    signal noticesWaiting
    signal trayClicked(int x, int y)
    signal trayAway
    // A mouse mode changed or a latch let go, so the keyboard reads its state again.
    signal keysChanged
    // A new chip lane: read `words` again.
    signal wordsChanged
    // The clipboard panel shows a new view or closes: read `clip` again.
    signal clipChanged

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
    // Changes the overlay's state as the hub does: it wakes QML only when no wake waits for a read, unlike setting
    // `assist`, which wakes each time.
    function change(state) {
        newest = state;
        if (waiting)
            return;
        waiting = true;
        assist = state;
    }
    function assistJson() {
        assistReads++;
        waiting = false;
        return JSON.stringify(newest);
    }
    function scrollJson() {
        scrollReads++;
        return JSON.stringify(scroll);
    }
    function watchEntries(title) {
        note("watchEntries " + title);
        if (notYet <= 0)
            return true;
        notYet--;
        return false;
    }
    function unwatchEntries(title) {
        note("unwatchEntries " + title);
    }
    function markPages(title) {
        note("markPages " + title);
        if (unpaged <= 0)
            return true;
        unpaged--;
        return false;
    }
    function layUnder(title, under) {
        note("layUnder " + title + " " + under);
        return true;
    }
    // The bundle as the app holds it: fonts the texts test lays out, and icons the tests' icon plugin serves.
    function assets() {
        const at = rel => Qt.resolvedUrl(Assets.root + "/" + rel).toString();
        const a = Assets.assets;
        return JSON.stringify({ icons: a.icons, ext: a.ext, fonts: a.fonts.map(at) });
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
    // Fluent fills the page and the count into `page.of`, whose variables the fixture shows as `{$name}`.
    function pageText(at, count) {
        return text("page.of").replace("{$at}", at).replace("{$count}", count);
    }
    // The same for the shortcuts layer's page, which readers hear after the Next key's name.
    function layerPageText(page, pages) {
        return text("key.layer-page").replace("{$page}", page).replace("{$pages}", pages);
    }
    // Names a shortcut's keys as the hub does: a key's label, else its `keyname` text, else its name in upper case.
    function keysText(steps) {
        const named = n => ["label.", "keyname."].map(g => said.texts[g + n]).find(t => t !== undefined);
        const shown = n => named(n) !== undefined ? named(n) : n.toUpperCase();
        const each = JSON.parse(steps).filter(s => s.length > 0).map(s => s.map(shown).join(text("combo.join")));
        return each.join(text("combo.then"));
    }
    // A shortcut's tip as the hub makes it: its name and keys in `tip.keys`, or the name alone when it has no keys.
    function keysTip(nameKey, steps) {
        const [name, keys] = [text(nameKey), keysText(steps)];
        if (keys.length === 0)
            return name;
        return text("tip.keys").replace("{$name}", () => name).replace("{$keys}", () => keys);
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
    // A pop-up panel goes where `popAt` puts pop-ups, or centred on `fitCentre` at its fit, shrunk to the room as the
    // hub does, never below the least fit. Each size asked is kept in `fitAsks`.
    function popFit(over, left, centred, w, h) {
        note("popFit " + (JSON.parse(over) ? "over" : "home") + " " + (centred ? "centred" : "left"));
        fitAsks.push({ w: w, h: h });
        const fit = Math.min(1, Math.max(look.shape.panel.min_fit, fitRoom / h));
        const x = isNaN(fitCentre) ? popSpot.x : fitCentre - w * fit / 2;
        return JSON.stringify({ x: x, y: popSpot.y, fit: fit });
    }
    // A menu goes up from the click, as the hub puts it.
    function menuAt(x, y, w, h) {
        note("menuAt " + x + " " + y);
        return JSON.stringify({ x: x, y: y - h });
    }
    function modeJson() {
        return JSON.stringify(mode);
    }
    function keepMode(away, minimised) {
        mode = { away: away, minimised: minimised };
    }
    function backdrop(title) {
        note("backdrop " + title);
        return frosted;
    }
    // The chip lane, counted in `reads`, a chip click, a menu, list or control tap, and a tap the top bar handles alone,
    // noted as `pickChip round index`, `wordsAsk kind arg` and `tapped`.
    function words() {
        reads.words++;
        return JSON.stringify(lane);
    }
    function pickChip(round, index) {
        note("pickChip " + round + " " + index);
    }
    function wordsAsk(kind, arg) {
        note("wordsAsk " + kind + " " + arg);
    }
    function tapped() {
        note("tapped");
    }
    // The clipboard panel's view, a click on it, noted as `clipAct kind arg` with its round kept, and Open folder.
    function clip() {
        return JSON.stringify(clipView);
    }
    function clipAct(kind, arg, round) {
        clipRounds = clipRounds.concat([round]);
        note("clipAct " + kind + " " + arg);
    }
    function openFolder() {
        note("openFolder");
    }
}

// The keyboard steps aside while Snip captures (spec "Keyboard during a capture"): Bubble turns it into its bubble as
// Minimise does; Hidden hides it, its bubble and every other window of ours but Snip's, with their shadows, at once and
// with no fade, and keeps them hidden while it lasts. The end brings back the mode the hub kept and what still shows.
pragma ComponentBehavior: Bound
import QtQuick

QtObject {
    id: aside

    // The app, whose Ui object, windows and mode it steps aside.
    required property var app

    // How the keyboard is aside now, as `Ui.modeJson().aside` names it, or empty.
    property string way: ""
    readonly property SnipNames names: SnipNames {}
    // Hidden is on: no shadow window shows, so nor the window it owns, and the overlay and the outline hide too.
    readonly property bool hidden: way === names.aside.hidden

    property Connections woke: Connections {
        target: aside.app.ui
        function onAsideAsked() {
            aside.read();
        }
    }
    // Every shadow window of ours and the outline's edges hide while Hidden lasts; the overlay reads `hidden` itself.
    Component.onCompleted: {
        const a = app;
        for (const w of [a.kbGlow, a.bubbleGlow, a.menuGlow, a.noticeGlow, a.strip, a.dictation, a.outline])
            w.hush = Qt.binding(() => aside.hidden);
    }

    // Reads how the keyboard is aside: Hidden hides ours until it ends; the end brings back the kept mode.
    function read() {
        const m = JSON.parse(app.ui.modeJson()) || {};
        const was = way;
        way = m.aside || "";
        if (hidden && was !== way)
            hush();
        else if (was === names.aside.hidden && !hidden)
            unhush();
        if (way === names.aside.bubble && was !== way && !app.away)
            app.minimise();
        if (!way && was)
            back(m);
    }
    // Hides the keyboard, its bubble, the tray menu, the tips, a move under way and every other window of ours.
    function hush() {
        app.menu.shut();
        app.bubble.hideTip();
        app.face.hint("", null);
        app.face.finish();
        for (const w of [app.face, app.kbGlow, app.bubble, app.bubbleGlow])
            w.visible = false;
        for (const [glow, w] of kept()) {
            w.visible = false;
            glow.visible = false;
        }
    }
    // Each kept window reads its state again and shows only if it still has something to show; a tip or the tray
    // menu answered a pointer that has moved on, so they stay shut.
    function unhush() {
        app.bubble.hideTip();
        app.face.hint("", null);
        const [n, p] = [app.notice, app.dictation.panel];
        if (n.shown)
            n.open();
        app.strip.chips.read();
        app.dictation.bar.read();
        if (p)
            p.start();
    }
    // The keyboard comes back in the mode the hub kept: shown, as its bubble, or still closed.
    function back(m) {
        app.away = !!m.away;
        app.minimised = !!m.minimised;
        app.follow();
    }
    // The windows that may show besides the keyboard and its bubble, each after the shadow that owns it; the keyboard's
    // panels go with it, and the outline's edges and the overlay hide by `hidden`.
    function kept() {
        const [a, p] = [app, app.dictation.panel];
        const all = [[a.noticeGlow, a.notice], [a.strip, a.strip.chips], [a.dictation, a.dictation.bar]];
        if (p)
            all.push([p, p.pane]);
        return all;
    }
}

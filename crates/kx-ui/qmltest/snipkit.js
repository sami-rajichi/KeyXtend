.pragma library
.import "fixtures/snip.js" as Snip
.import "fixtures/ui-en.js" as En
.import "fixtures/ui-fr.js" as Fr
.import "fixtures/ui-ar.js" as Ar
.import "board.js" as Board
// What Snip's tests share: the hub's views with their frozen screens and the editor's picture drawn from a data URL,
// since qmltestrunner has no `snip` provider, each language's texts with Snip's own, and the texts by what they say.

// A 2 by 2 picture for each frozen screen and the editor.
var picture = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEUlEQVR4nGO4o6HxH4QZYAwATUwIraFLCzAAAAAASUVORK5CYII=";
// The languages, and each one's UI fixture.
var langs = ["en", "fr", "ar"];
var ui = { en: En, fr: Fr, ar: Ar };

// A copy of view `name`, each frozen screen and the editor's picture drawn as `picture`, with `more` laid over it.
function view(name, more) {
    const v = JSON.parse(JSON.stringify(Snip.views[name]));
    for (const f of v.frozen)
        f.src = picture;
    if (v.edit)
        v.edit.src = picture;
    return Object.assign(v, more || {});
}

// Language `tag`'s UI fixture with Snip's own texts in, as the hub loads them in one bundle.
function lang(tag) {
    const u = ui[tag];
    return { lang: u.lang, texts: Object.assign({}, u.texts, Snip.texts[tag]) };
}

// The fonts, icons and texts for look `lk` in language `tag`.
function resOf(lk, tag) {
    return Board.resOf(lk, lang(tag));
}

// Snip's text `key` in language `tag`.
function text(tag, key) {
    return lang(tag).texts[key];
}

// The shown texts under `item`, by what they say.
function texts(item, said) {
    return Board.all(item, o => o.visible && o.text === said && o.font !== undefined);
}

// The shown controls under `item` whose UI Automation id (`uid`) starts with `prefix`, left to right on screen.
function keys(item, prefix) {
    const all = Board.all(item, o => o.visible && typeof o.uid === "string" && o.uid.startsWith(prefix));
    return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
}

// The shown control under `item` with UI Automation id `uid`, read from its `uid`.
function key(item, uid) {
    return Board.all(item, o => o.visible && o.uid === uid)[0];
}

// The shown keys of capture window `win` (SnipKey, SnipPill), ways and switches too, left to right on screen.
function buttons(win) {
    const all = Board.all(win.contentItem, o => o.visible && typeof o.key === "string" && typeof o.hit === "function");
    return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
}

// The shown key `id` of capture window `win`.
function button(win, id) {
    return buttons(win).find(o => o.key === id);
}

// Physical box `r` as Snip's windows of app `app` show it, in whole Qt units.
function shown(app, r) {
    const b = app.box(r);
    return [Math.round(b.x), Math.round(b.y), Math.max(1, Math.round(b.width)), Math.max(1, Math.round(b.height))];
}

// Window `w`'s box in Qt units.
function box(w) {
    return [w.x, w.y, w.width, w.height];
}

// The first item under `item` named `name` (`objectName`), shown or not.
function named(item, name) {
    return Board.all(item, o => o.objectName === name)[0];
}

// The shown dims under `item` in look `lk`'s Snip dim, as the close question lays over the editor.
function dims(item, lk) {
    return Board.all(item, o => o.visible && typeof o.color === "object" && Qt.colorEqual(o.color, lk.common.snip_dim));
}

// Each of Snip's shadows of app `app` with the window it owns: editor, question, panel, capture bar, strip, record bar.
function owned(app) {
    return [[app.editor, app.editor.editor], [app.ask, app.ask.question], [app.panel, app.panel.pane],
            [app.bar, app.bar.bar], [app.strip, app.strip.strip], [app.record, app.record.bar]];
}

// Windows `wins` stacked as Qt 6.11 on Windows would. Each enters in front as Qt first makes it: at a `make` or
// `shield` the fake `snip` heard for its title, after its owner, as `make_titled` does; else at its first show, inside
// its owner's. Each flags change lifts that window alone to the front (SWP_NOOWNERZORDER), and Qt keeps new flags
// before telling, so a window told later whose flags already changed moved first.
function stack(wins, snip) {
    const st = { wins: wins, z: [], seen: wins.map(w => w.flags) };
    seed(st, snip);
    for (const w of wins) {
        w.flagsChanged.connect(() => told(st, w));
        w.visibleChanged.connect(() => made(st, w));
    }
    const at = w => st.z.indexOf(w);
    return { above: (a, b) => at(a) >= 0 && at(b) >= 0 && at(a) < at(b) };
}

// Enters each window of stack `st` that the fake `snip` heard a `make` or `shield` for, in the order heard.
function seed(st, snip) {
    const asked = snip ? snip.calls.filter(c => c.make !== undefined || c.shield !== undefined) : [];
    for (const c of asked) {
        const title = c.make !== undefined ? c.make : c.title;
        st.wins.filter(w => w.title === title).forEach(w => ahead(st, w));
    }
}

// Enters window `w` in front of stack `st`, after its owner, as `make_titled` makes a pair.
function ahead(st, w) {
    for (const o of [w.transientParent, w]) {
        if (st.wins.indexOf(o) >= 0 && st.z.indexOf(o) < 0)
            st.z.unshift(o);
    }
}

// Enters window `w` in front of stack `st` at its first show, unless a make entered it already.
function made(st, w) {
    if (w.visible && st.z.indexOf(w) < 0)
        st.z.unshift(w);
}

// Window `w` of stack `st` told of new flags: each other window whose flags already changed lifts first, then `w`.
function told(st, w) {
    st.wins.forEach((o, i) => {
        if (o !== w && moved(st, i))
            lift(st, o);
    });
    if (moved(st, st.wins.indexOf(w)))
        lift(st, w);
}

// True once for each new flags of window `i` of stack `st`.
function moved(st, i) {
    if (st.wins[i].flags === st.seen[i])
        return false;
    st.seen[i] = st.wins[i].flags;
    return true;
}

// Lifts window `w` alone to the front of stack `st`, once it is in.
function lift(st, w) {
    const at = st.z.indexOf(w);
    if (at >= 0)
        st.z.unshift(st.z.splice(at, 1)[0]);
}

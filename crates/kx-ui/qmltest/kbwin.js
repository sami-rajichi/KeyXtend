.pragma library
.import "fixtures/face.js" as Face
.import "pointer.js" as Pointer
// The keyboard window helpers the scale and place tests share, over stand-in bridges.

// Every item under `item` for which `ok` holds.
function all(item, ok, found) {
    found = found || [];
    if (ok(item))
        found.push(item);
    for (let i = 0; i < item.children.length; i++)
        all(item.children[i], ok, found);
    return found;
}

// How often the keyboard `kb` heard `call`.
function heard(kb, call) {
    return kb.calls.filter(c => c === call).length;
}

// A shown keyboard window that the keyboard put at physical `at`, its pointer there too, made by test case `t` under
// `root` from the `ui`, `kb` and `win` components in `parts`; the real pointer passes through it and the catchers.
function make(t, parts, root, at) {
    const state = Object.assign({}, Face.states.en, { at: at });
    const kb = t.createTemporaryObject(parts.kb, root, { pointer: at, state: state });
    const win = t.createTemporaryObject(parts.win, root, { ui: t.createTemporaryObject(parts.ui, root), keyboard: kb });
    const mover = all(win.contentItem, o => o.covers !== undefined)[0];
    const covers = [];
    for (let i = 0; i < mover.covers.count; i++)
        covers.push(mover.covers.objectAt(i));
    Pointer.through([win].concat(covers));
    win.place();
    win.visible = true;
    t.waitForRendering(win.contentItem);
    const rescale = all(win.contentItem, o => o.keep !== undefined)[0];
    const placing = all(win.contentItem, o => o.owed !== undefined)[0];
    return { win: win, kb: kb, mover: mover, rescale: rescale, placing: placing };
}

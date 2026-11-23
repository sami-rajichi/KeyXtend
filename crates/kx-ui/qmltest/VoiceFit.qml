// What the Voice panel's fit tests share (ADR-0008): the page under the bands, a list's height, and checks that a page
// keeps that height, lays its body inside it, and cuts no text or draws it past the panel.
import QtQuick
import "board.js" as Board
import "voicekit.js" as Kit

VoiceCase {
    // The shown children of `item`.
    function kids(item) {
        const out = [];
        for (let i = 0; i < item.children.length; i++)
            if (item.children[i].visible)
                out.push(item.children[i]);
        return out;
    }
    // The panel's page under the bands: its list, Compare's set text, a card or a key page.
    function page(box) {
        const hit = Kit.only(kids(box.foot.parent).filter(o => o.sheet !== undefined));
        verify(hit, "one page shows");
        return hit;
    }
    // A list's height: one page of rows and the body's padding.
    function listH(box) {
        return (box.pn.body_pad_px[0] + box.pn.body_pad_px[2]) * box.s + box.pageH;
    }
    // Whether `o` lies in a typed line, which keeps its text's end in sight as a text box does.
    function typed(o) {
        for (let at = o.parent; at; at = at.parent)
            if (at.drawn !== undefined && at.masked !== undefined)
                return true;
        return false;
    }
    // `o`'s box in the panel's coordinates.
    function rect(box, o) {
        const at = o.mapToItem(box.contentItem, 0, 0);
        return { x: at.x, y: at.y, w: o.width, h: o.height };
    }
    // Checks that `box`'s page keeps a list's height and lays its body inside it.
    function keeps(box) {
        const p = page(box);
        verify(p.height <= listH(box) + 0.5, "the page keeps a list's height: " + p.height + " of " + listH(box));
        if (p.rowW === undefined)
            holds(box);
    }
    // Checks that `box`'s page lays its body inside it.
    function holds(box) {
        const p = page(box);
        const body = p.body !== undefined ? p.body : Kit.only(kids(p).filter(o => o.spacing !== undefined));
        verify(body, "one body");
        within(box, body, "it");
    }
    // Checks that `o`, named `what` in a failure, lies inside `box`'s page from top to bottom.
    function within(box, o, what) {
        const inside = rect(box, o), room = rect(box, page(box));
        verify(inside.y >= room.y - 0.5, what + " starts inside the page: " + inside.h + " of " + room.h);
        verify(inside.y + inside.h <= room.y + room.h + 0.5, what + " ends inside the page: " + inside.h + " of " + room.h);
    }
    // Checks that no text on `box` is cut, drawn past the panel or past its own width, but those `skip` leaves out.
    function uncut(box, skip) {
        const shown = o => o.visible && o.text !== undefined && o.font !== undefined && !!o.text && !typed(o)
            && !(skip && skip(o));
        const said = Board.all(box.contentItem, shown);
        verify(said.length > 0, "texts show");
        const cut = said.filter(o => o.truncated);
        compare(cut.map(o => o.text), [], "no text elided");
        const out = said.filter(o => {
            const r = rect(box, o);
            return r.x < -0.5 || r.y < -0.5 || r.x + r.w > box.width + 0.5 || r.y + r.h > box.height + 0.5;
        });
        compare(out.map(o => o.text), [], "every text inside the panel");
        const wide = said.filter(o => o.contentWidth > o.width + 0.5);
        compare(wide.map(o => o.text), [], "every text within its own width");
    }
}

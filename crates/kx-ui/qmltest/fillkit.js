.pragma library
.import "fixtures/fill.js" as Fill
.import "board.js" as Board
// What Quick-fill's tests share: the hub's views, the panel's parts by gate id or by what they say, and its rows.

// A copy of view `name` in language `tag`.
function view(tag, name) {
    return JSON.parse(JSON.stringify(Fill.views[tag][name]));
}

// The one item in `hits`, or null; a library cannot read attached properties, so each test finds the hits by gate id.
function only(hits) {
    return hits.length === 1 ? hits[0] : null;
}

// The shown texts under `item`, by what they say.
function texts(item, said) {
    return Board.all(item, o => o.visible && o.text === said && o.font !== undefined);
}

// Every row slot under `item`, padding too, in place order: rows and fields alike.
function slots(item) {
    const all = Board.all(item, o => o.visible && o.place !== undefined && o.slot !== undefined);
    return all.sort((a, b) => a.place - b.place);
}

// The slots under `item` that hold a row.
function filled(item) {
    return slots(item).filter(o => o.slot !== null);
}

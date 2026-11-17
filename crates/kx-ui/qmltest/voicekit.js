.pragma library
.import "fixtures/voicepanel.js" as Panel
.import "board.js" as Board
// What the Voice panel's tests share: the hub's views, the panel's texts by what they say, and WCAG contrast over the
// colours the QML draws, laid over what lies under them.

// WCAG 2.2's least contrast for text, and for icons and other shapes.
var text = 4.5;
var shapes = 3;

// A copy of the hub's whole view `name` in language `tag`.
function view(tag, name) {
    return JSON.parse(JSON.stringify(Panel.views[tag][name]));
}

// The one item in `hits`, or null; a library cannot read attached properties, so each test finds the hits by gate id.
function only(hits) {
    return hits.length === 1 ? hits[0] : null;
}

// The shown texts under `item`, by what they say.
function texts(item, said) {
    return Board.all(item, o => o.visible && o.text === said && o.font !== undefined);
}

// Colour `c`, a colour or a fixture's `#AARRGGBB` text, as a colour.
function rgba(c) {
    return Qt.tint(c, "transparent");
}

// Colour `t` laid over the opaque colour `u` by its alpha.
function over(t, u) {
    const [top, under] = [rgba(t), rgba(u)];
    const a = top.a;
    const mix = (x, y) => x * a + y * (1 - a);
    return Qt.rgba(mix(top.r, under.r), mix(top.g, under.g), mix(top.b, under.b), 1);
}

// WCAG 2.2's relative luminance of an opaque colour.
function luminance(o) {
    const c = rgba(o);
    const ch = v => v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    return 0.2126 * ch(c.r) + 0.7152 * ch(c.g) + 0.0722 * ch(c.b);
}

// The contrast of `ink`, laid over the opaque `bg`, against `bg`.
function ratio(ink, bg) {
    const a = luminance(over(ink, bg));
    const b = luminance(bg);
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
}

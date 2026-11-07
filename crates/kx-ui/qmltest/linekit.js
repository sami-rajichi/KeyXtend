.pragma library
.import "board.js" as Board
// What the tests of a panel line's caret share: its words, its parts and where they stand in the line.

function latin(text) {
    return { text: text, rtl: false, arabic: false };
}

function arabic(text) {
    return { text: text, rtl: true, arabic: true };
}

// The fonts and shared parts of look `lk` in language `ui`, with a bundled Arabic font, as a board's are.
function res(lk, ui) {
    const base = Board.resOf(lk, ui);
    const font = lk.look.arabic_fonts[lk.look.arabic_fonts.length - 1];
    return Object.assign({}, base, { fam: Object.assign({}, base.fam, { arabic: font }) });
}

function caret(line) {
    return Board.all(line, o => o.caret === true)[0];
}

// The editor that draws the letters, and the plain label of a line that does not take them.
function editor(line) {
    return Board.all(line, o => o.cursorPosition !== undefined)[0];
}

function label(line) {
    return Board.all(line, o => o.font !== undefined && o.elide !== undefined && o.text === line.drawn)[0];
}

// Where the caret stands in the line.
function xOf(line) {
    return caret(line).mapToItem(line, 0, 0).x;
}

// Where place `at` of the editor falls in the line.
function spot(line, at) {
    const ed = editor(line);
    return ed.mapToItem(line, ed.positionToRectangle(at).x, 0).x;
}

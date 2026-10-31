.pragma library
.import "fixtures/clip.js" as Clip
.import "board.js" as Board
// What the clipboard panel's tests share: the hub's views with their pictures drawn from data URLs, since
// qmltestrunner has no `clip` provider, and the panel's parts by what they say.

// A 2 by 2 picture, and the one clear pixel the provider gives when it has no picture.
var picture = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEUlEQVR4nGO4o6HxH4QZYAwATUwIraFLCzAAAAAASUVORK5CYII=";
var clear = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAAC0lEQVR4nGNgAAIAAAUAAXpeqz8AAAAASUVORK5CYII=";

// A copy of view `name` in language `tag`, each `image://clip/` URL drawn as `picture`.
function view(tag, name) {
    const v = JSON.parse(JSON.stringify(Clip.views[tag][name]));
    for (const r of v.rows)
        r.thumb = r.thumb ? picture : null;
    if (v.preview && v.preview.url)
        v.preview.url = picture;
    return v;
}

// The one item in `hits`, or null; a library cannot read attached properties, so each test finds the hits by gate id.
function only(hits) {
    return hits.length === 1 ? hits[0] : null;
}

// The shown texts under `item`, by what they say.
function texts(item, said) {
    return Board.all(item, o => o.visible && o.text === said && o.font !== undefined);
}

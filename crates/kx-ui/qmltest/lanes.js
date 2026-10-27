.pragma library
.import "fixtures/words.js" as Words
.import "fixtures/ui-en.js" as En
// The chip lanes the words tests draw, made from the lanes the hub writes (fixture words.js).

// The hub's chips lane holding only its chips at `places`, with the fields of `over` set on it, such as `forgetting`.
function some(places, over) {
    const all = Words.lanes.chips;
    return Object.assign({}, all, { chips: places.map(i => all.chips[i]) }, over || {});
}

// A lane of word chips showing `texts`, each named as the hub names it.
function words(texts) {
    const word = Words.lanes.chips.chips[0];
    const name = t => En.texts["chip.name"].replace("{$word}", t);
    const chips = texts.map(t => Object.assign({}, word, { text: t, name: name(t) }));
    return Object.assign({}, Words.lanes.chips, { chips: chips });
}

// The language key's own designs over the face's fixtures: Cycle shows a globe, the name and one dot per layout, its
// dots wrap inside the cap, Carousel's side names take their tokens, and each design draws its own icons.
import QtQuick
import QtTest
import "../qml"
import "fixtures/assets.js" as Assets
import "board.js" as Board
import "fixtures/face.js" as Face
import "fixtures/ui-en.js" as En
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    readonly property var res: ({
        fam: { latin: "Segoe UI", arabic: "Segoe UI", mark: "Segoe UI", ui: "Segoe UI" },
        icons: Assets.assets.icons,
        ext: Assets.assets.ext,
        tr: k => En.texts[k] !== undefined ? En.texts[k] : k,
        bar: id => id,
        dpad: id => id,
        uid: (group, id) => group + "." + id,
        space: parts => parts.join(" · ")
    })

    Component { id: cycleC; LangCycle {} }
    Component { id: keyC; Key {} }

    TestCase {
        name: "langkey"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }

        function test_the_cycle_design_shows_a_globe_the_name_and_one_dot_per_layout() {
            const lang = Face.states.en.lang;
            const c = createTemporaryObject(cycleC, root, { lang: lang, lk: N.lk, res: root.res, s: 1, ink: "black", room: 40 });
            const cy = N.lk.shape.lang.cycle;
            const dots = Board.all(c, o => o.lit !== undefined && o.radius !== undefined);
            const each = (on, off) => [...Array(lang.count).keys()].map(i => i === lang.at ? on : off);
            compare(dots.length, lang.count, "one dot per layout");
            compare(dots.map(d => d.opacity), each(1, cy.dot_off));
            // An unlit dot keeps 3:1 too, so the current layout's dot stands out by size.
            compare(dots.map(d => d.width), each(cy.dot_on_px, cy.dot_px));
            compare(new Set(dots.map(d => d.mapToItem(c, d.width / 2, 0).x)).size, 1, "one column, centred");
            compare(Board.all(c, o => o.tint !== undefined && o.icon !== undefined)[0].icon, N.lk.icons.lang.cycle);
            compare(Board.all(c, o => o.text === lang.cur.short).length, 1, "the current name");
            c.lang = Face.states.ar.lang;
            compare(Board.all(c, o => o.text === Face.states.ar.lang.cur.short).length, 1, "the next name rises in");
        }

        function test_many_layouts_wrap_their_dots_inside_the_cap() {
            const room = 18;
            const lang = Object.assign({}, Face.states.en.lang, { count: 8, at: 0 });
            const c = createTemporaryObject(cycleC, root, { lang: lang, lk: N.lk, res: root.res, s: 1, ink: "black", room: room });
            const dots = () => Board.all(c, o => o.lit !== undefined && o.radius !== undefined);
            compare(dots().length, 8);
            // The key centres the design, so the cap's edges sit half the room above and below its middle.
            const inside = d => {
                const y = d.mapToItem(c, 0, 0).y - (c.height - room) / 2;
                return y >= 0 && y + d.height <= room;
            };
            for (let at = 0; at < 8; at++) {
                c.lang = Object.assign({}, lang, { at: at });
                tryVerify(() => dots().every(inside), 2000, "every dot inside the cap with " + at + " lit");
            }
            verify(new Set(dots().map(d => d.mapToItem(c, d.width / 2, 0).x)).size > 1, "more than one column");
        }

        // The side names keep the owner's faded decoration (kx-theme's one contrast exception) in their own weight.
        function test_the_carousel_side_names_take_their_size_fade_and_weight_from_the_tokens() {
            const kv = Face.board.keys.find(k => k.kind === "lang");
            const key = createTemporaryObject(keyC, root, { kv: kv, ks: Face.states.en.keys[kv.id], lk: N.lk, lang: Face.states.en.lang, design: "carousel", res: root.res, s: 1 });
            const sh = N.lk.shape.lang;
            const names = Board.all(key, o => o.cur !== undefined && o.modelData !== undefined);
            compare(names.length, 3, "back, current and next");
            for (const n of names.filter(n => !n.cur)) {
                compare(n.opacity, sh.side[1]);
                compare(n.font.pixelSize, sh.side[0]);
                compare(n.font.weight, sh.side_weight);
            }
            compare(names.filter(n => n.cur)[0].font.weight, sh.weight);
        }

        function test_each_design_draws_the_language_key_its_own_way() {
            const kv = Face.board.keys.find(k => k.kind === "lang");
            const lang = Face.states.en.lang;
            const shows = design => {
                const key = createTemporaryObject(keyC, root, { kv: kv, ks: Face.states.en.keys[kv.id], lk: N.lk, lang: lang, design: design, res: root.res, s: 1 });
                const visible = o => {
                    for (let p = o; p && p !== key; p = p.parent)
                        if (!p.visible)
                            return false;
                    return true;
                };
                return Board.all(key, o => o.icon !== undefined && o.tint !== undefined && o.icon.length > 0 && visible(o)).map(o => o.icon);
            };
            compare(shows("carousel"), []);
            compare(shows("cycle"), [N.lk.icons.lang.cycle]);
            compare(shows("list"), [N.lk.icons.lang.list]);
        }
    }
}

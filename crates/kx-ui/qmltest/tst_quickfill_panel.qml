// Quick-fill's head and foot over the hub's views: New, Backup and Close, dimmed where they cannot act; the pager, the
// note and the waiting line; every control's name and id for readers and gates in each language; the panel's size
// across its pages; and no text cut in any look in English, French or Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "fillkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/fill.js" as Fill
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-dolch-light.js" as DL

Item {
    id: root
    width: 200
    height: 100

    readonly property var langs: ({ en: En, fr: Fr, ar: Ar })
    // Every view the fixtures hold.
    readonly property var names: Object.keys(Fill.views.en)

    Component { id: uiC; FakeUi {} }
    Component { id: fillC; FakeFill {} }
    Component { id: boxC; QuickFillBox {} }
    Component { id: namesC; QuickFillNames {} }

    TestCase {
        name: "quickfill_panel"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        // The one shown part whose gate id is `id`, or null.
        function one(item, id) {
            return Kit.only(Board.all(item, o => o.visible && o.Accessible.id === id));
        }
        // The panel showing view `name` in language `tag` (English by default) and `look` (Native by default), laid out.
        function open(name, tag, look) {
            const ui = langs[tag || "en"];
            const lk = (look || NL).lk;
            const props = { ui: createTemporaryObject(uiC, root), fill: createTemporaryObject(fillC, root), lk: lk, res: Board.resOf(lk, ui), s: 1, view: Kit.view(tag || "en", name), rows: En.settings.pageRows, rtl: ui.lang.rtl };
            const box = createTemporaryObject(boxC, root, props);
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        function button(box, id) {
            const b = one(box.contentItem, id);
            verify(b, id + " shows once");
            return b;
        }
        // Runs `check` on each view in language `tag` and `look`, in one panel, as a window for each would run out of memory.
        function eachView(tag, look, check) {
            const box = open(root.names[0], tag, look);
            for (const name of root.names) {
                box.view = Kit.view(tag, name);
                verify(waitForPolish(box), name + " laid out");
                check(box, name);
            }
            box.visible = false;
        }

        function test_every_button_id_qml_sends_is_one_the_hub_words() {
            const names = createTemporaryObject(namesC, root);
            const words = Fill.views.en.list.buttons;
            for (const id of Object.values(names.buttons))
                verify(words[id] !== undefined, id + " is a panel button");
        }

        // Each kind, place, row button and field button the hub names in any view and language has its icon from icons.toml.
        function test_every_kind_and_button_the_hub_names_has_an_icon() {
            const names = createTemporaryObject(namesC, root);
            const ic = NL.lk.icons.fill;
            const known = new Set(Object.values(ic));
            const has = (icon, what) => verify(!!icon && known.has(icon), what + " has an icon");
            const place = (kind, at) => {
                verify(names.placeIcons[kind] !== undefined, at + "place " + kind + " is named");
                has(names.placeIcon(ic, kind), at + "place " + kind);
            };
            const lists = [names.pages.places, names.pages.folders, names.pages.found];
            for (const tag of ["en", "fr", "ar"]) {
                for (const name of root.names) {
                    const s = Fill.views[tag][name].sheet;
                    const at = tag + " " + name + ": ";
                    const placed = lists.indexOf(s.is) >= 0;
                    (s.kinds || []).forEach(k => has(names.kindIcon(ic, k.id), at + "kind " + k.id));
                    (s.rows || []).filter(r => r.kind).forEach(r => placed ? place(r.kind, at) : has(names.kindIcon(ic, r.kind), at + "kind " + r.kind));
                    (s.places || []).forEach(r => place(r.kind, at));
                    if (s.is === names.pages.foreign)
                        place(s.kind, at);
                    (s.rows || []).concat(s.places || []).forEach(r => r.buttons.forEach(b => has(names.rowIcon(ic, b.id), at + "row " + b.id)));
                    for (const f of s.fields || [])
                        f.buttons.forEach(b => [false, true].forEach(on => has(names.fieldIcon(ic, b.id, on), at + "field " + b.id)));
                }
            }
            compare(names.kindIcon(ic, "unknown"), ic.kind_other, "a kind the panel does not know takes Other's");
            compare(names.placeIcon(ic, "unknown"), ic.place_folder, "a place the panel does not know takes a folder's");
            verify(names.isFile("file") && !names.isFile("folder"), "only a file is chosen");
        }

        // The field buttons that read as on keep their icons under the ids `field` holds, spelt once.
        function test_the_field_icons_follow_the_field_ids() {
            const names = createTemporaryObject(namesC, root);
            const ic = NL.lk.icons.fill;
            compare([false, true].map(on => names.fieldIcon(ic, names.field.show, on)), [ic.show, ic.hide]);
            compare([false, true].map(on => names.fieldIcon(ic, names.field.hidden, on)), [ic.hidden_off, ic.hidden_on]);
        }

        // Every page the hub names has its view, so none shows blank.
        function test_every_page_the_hub_names_has_a_view() {
            const names = createTemporaryObject(namesC, root);
            const box = open(root.names[0]);
            for (const name of root.names) {
                const is = Fill.views.en[name].sheet.is;
                verify(Object.values(names.pages).indexOf(is) >= 0, name + ": " + is + " is named");
                verify(box.sheets[is], name + ": " + is + " has a view");
            }
            box.visible = false;
        }

        // A view for another page reaches the shown page before the panel swaps it, so any page can follow any other.
        function test_any_page_can_follow_any_other_with_no_warning() {
            const each = {};
            root.names.forEach(n => each[Fill.views.en[n].sheet.is] = n);
            const pages = Object.values(each);
            const box = open(pages[0]);
            for (const from of pages) {
                for (const to of pages) {
                    for (const name of [from, to]) {
                        box.view = Kit.view("en", name);
                        verify(waitForPolish(box), name + " laid out");
                        const shown = Board.all(box.contentItem, o => o.draws !== undefined);
                        compare(shown.length, 1, name + " shows one page");
                        verify(shown[0].view === box.view, name + " draws its own view, so a wrong `draws` fails here");
                    }
                }
            }
            box.visible = false;
        }

        function test_the_head_sends_new_backup_and_close_and_waits_on_new_or_edit() {
            const box = open("list");
            verify(Kit.texts(box.contentItem, box.view.title)[0], "the title");
            for (const id of ["new", "backup", "close"])
                mouseClick(button(box, "fill." + id));
            compare(box.fill.calls, ["button new 0", "button backup 0", "button close 0"]);
            const edit = open("edit");
            for (const id of ["new", "backup"]) {
                verify(!button(edit, "fill." + id).enabled, id + " waits while an entry is edited");
                verify(button(edit, "fill." + id).opacity < 1, id + " looks off");
            }
            verify(button(edit, "fill.close").enabled, "Close always acts");
            box.visible = edit.visible = false;
        }

        function test_the_foot_pages_and_back_is_off_on_the_first() {
            const box = open("list");
            const [back, on] = [button(box, "fill.page-back"), button(box, "fill.page-on")];
            verify(!back.enabled && on.enabled, "page 1 of 3");
            verify(Kit.texts(box.contentItem, "1 of 3")[0], "where the list is");
            mouseClick(back);
            mouseClick(on);
            compare(box.fill.calls, ["page 1"]);
            compare(one(open("short").contentItem, "fill.page-on"), null, "one page needs no pager");
            box.visible = false;
        }

        // A note is an alert readers hear, and it takes the waiting line's place while it shows.
        function test_the_foot_shows_a_note_or_the_waiting_line() {
            const box = open("noted");
            const note = Kit.texts(box.contentItem, box.view.note)[0];
            verify(note, "the note shows");
            const alert = Board.all(box.contentItem, o => o.visible && o.Accessible.name === box.view.note && !o.Accessible.ignored);
            compare(alert.length, 1, "readers hear the note once");
            compare(alert[0].Accessible.role, Accessible.AlertMessage);
            compare(Kit.texts(box.contentItem, box.view.waiting.text).length, 0, "the waiting line gives way");
            const v = Kit.view("en", "noted");
            v.note = null;
            box.view = v;
            verify(waitForPolish(box));
            verify(Kit.texts(box.contentItem, v.waiting.text)[0], "then the waiting line shows");
            box.visible = false;
        }

        // Readers hear every control by name, and gates find each by its id, in every language.
        function test_every_control_has_a_name_and_an_id_in_each_language() {
            const roles = [Accessible.Button, Accessible.RadioButton, Accessible.CheckBox, Accessible.EditableText];
            for (const tag of ["en", "fr", "ar"]) {
                eachView(tag, NL, (box, name) => {
                    const all = Board.all(box.contentItem, o => o.visible && roles.indexOf(o.Accessible.role) >= 0);
                    verify(all.length > 0, tag + " " + name + " has controls");
                    for (const c of all) {
                        verify(c.Accessible.name.length > 0, tag + " " + name + ": " + c.Accessible.id + " is named");
                        verify(c.Accessible.id.startsWith("fill."), tag + " " + name + ": " + c.Accessible.name + " has an id");
                    }
                    const ids = all.map(c => c.Accessible.id);
                    compare(new Set(ids).size, ids.length, tag + " " + name + ": each id once");
                });
            }
        }

        function test_the_panel_keeps_its_size_across_its_pages_and_never_takes_focus() {
            const heights = [];
            eachView("en", NL, (box, n) => {
                verify(box.flags & Qt.WindowDoesNotAcceptFocus, n + " never takes focus");
                heights.push(n + " " + box.height);
            });
            const first = heights[0].split(" ")[1];
            verify(heights.every(h => h.split(" ")[1] === first || h.startsWith("noted")), heights.join(", "));
        }

        // Every text the hub words fits in each look: none is cut and each stays inside the panel; names, values and
        // hints may end in an ellipsis by design.
        function test_no_text_is_cut_in_any_look_in_english_french_or_arabic() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                for (const tag of ["en", "fr", "ar"])
                    eachView(tag, look, (box, name) => fits(box, look.lk.look.id + " " + tag + " " + name));
            }
        }
        // The panel `box` cuts no text and its buttons stand apart, named `where` in a failure.
        function fits(box, where) {
            const shown = Board.all(box.contentItem, o => o.visible && o.font !== undefined && o.text && o.elide === Text.ElideNone);
            for (const t of shown) {
                const at = t.mapToItem(box.contentItem, 0, 0);
                const said = where + ": " + t.text;
                verify(!t.truncated, said + " is cut");
                verify(at.x >= -0.5 && at.x + t.width <= box.width + 0.5, said + " leaves the panel");
                verify(t.implicitWidth <= t.width + 0.5 || t.wrapMode !== Text.NoWrap, said + " runs past its box");
            }
            const buttons = Board.all(box.contentItem, o => o.visible && o.uid !== undefined && o.tone !== undefined);
            const rows = {};
            for (const b of buttons) {
                const p = b.mapToItem(box.contentItem, 0, 0);
                verify(p.x >= -0.5 && p.x + b.width <= box.width + 0.5, where + ": " + b.uid + " leaves the panel");
                const y = Math.round(p.y);
                (rows[y] = rows[y] || []).push({ b: b, x: p.x });
            }
            for (const y in rows) {
                const line = rows[y].sort((a, c) => a.x - c.x);
                for (let i = 1; i < line.length; i++)
                    verify(line[i - 1].x + line[i - 1].b.width <= line[i].x + 0.5, where + ": " + line[i - 1].b.uid + " meets " + line[i].b.uid);
            }
        }
    }
}

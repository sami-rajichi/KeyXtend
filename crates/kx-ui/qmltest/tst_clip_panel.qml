// The clipboard panel's head, search line, filters, footer, Delete-all question, note and storage line over the hub's
// views: each click sends its kind, a button with nothing to act on shows disabled, the panel never takes focus, and no
// text is cut in English, French or Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "clipkit.js" as Kit
import "pointer.js" as Pointer
import "fixtures/clip.js" as Clip
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

    Component { id: uiC; FakeUi {} }
    Component { id: boxC; ClipBox {} }
    Component { id: namesC; ClipNames {} }
    Component { id: spyC; SignalSpy {} }

    TestCase {
        name: "clip_panel"
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
            const props = { ui: createTemporaryObject(uiC, root), lk: lk, res: Board.resOf(lk, ui), s: 1, view: Kit.view(tag || "en", name), rows: En.settings.pageRows, rtl: ui.lang.rtl };
            const box = createTemporaryObject(boxC, root, props);
            Pointer.through([box]);
            box.open();
            verify(waitForPolish(box), "laid out");
            return box;
        }
        function acts(box) {
            return box.ui.calls.filter(c => c.startsWith("clipAct") || c === "openFolder");
        }
        function button(box, id) {
            const b = one(box.contentItem, id);
            verify(b, id + " shows once");
            return b;
        }

        function test_the_click_kinds_are_the_hubs() {
            const names = createTemporaryObject(namesC, root);
            compare(names.acts, Clip.acts);
        }

        function test_insert_counts_the_ticks_and_the_footer_sends_its_acts() {
            const box = open("list");
            const insert = button(box, "clip.insert");
            compare(insert.label, "Insert (3)");
            for (const id of ["clip.preview", "clip.pin", "clip.delete", "clip.insert", "clip.delete-all", "clip.close"]) {
                verify(button(box, id).enabled, id);
                mouseClick(button(box, id));
            }
            mouseClick(button(box, "clip.open-folder"));
            compare(acts(box), ["clipAct preview 0", "clipAct pin 0", "clipAct delete 0", "clipAct insert 0", "clipAct delete-all 0", "clipAct close 0", "openFolder"]);
            box.visible = false;
        }

        function test_buttons_with_nothing_to_act_on_show_disabled_not_hidden() {
            const box = open("short");
            for (const id of ["clip.pin", "clip.delete", "clip.insert"]) {
                verify(!button(box, id).enabled, id + " with no tick");
                verify(button(box, id).opacity < 1, id + " looks off");
            }
            verify(button(box, "clip.preview").enabled, "the highlighted row can be previewed");
            mouseClick(button(box, "clip.insert"));
            const pinned = open("pinned");
            compare(button(pinned, "clip.pin").label, pinned.view.words.pin);
            compare(pinned.view.words.pin, "Unpin");
            compare(button(pinned, "clip.pin").icon, NL.lk.icons.clip.unpin);
            verify(!button(pinned, "clip.delete-all").enabled, "nothing unpinned to delete");
            const empty = open("empty");
            verify(!button(empty, "clip.preview").enabled, "nothing to preview");
            compare(acts(box).concat(acts(pinned), acts(empty)), []);
            box.visible = pinned.visible = empty.visible = false;
        }

        function test_the_filter_chips_send_their_place_and_mark_the_shown_one() {
            const box = open("list");
            const chips = box.view.filters.map(f => button(box, "clip.filter-" + f.id));
            compare(chips.map(c => c.Accessible.checked), [true, false, false, false, false]);
            const check = Board.all(chips[0], o => o.visible && o.icon === NL.lk.icons.clip.shown);
            compare(check.length, 1, "the shown chip is marked by more than its colour");
            mouseClick(chips[2]);
            compare(acts(box), ["clipAct filter 2"]);
            box.visible = false;
        }

        function test_the_search_line_shows_its_hint_then_the_typed_text_and_never_takes_focus() {
            const box = open("list");
            const search = button(box, "clip.search");
            compare(Kit.texts(search, box.view.search.hint).length, 1, "the hint while empty");
            const v = Kit.view("en", "list");
            v.search = { text: "مرحبا", hint: v.search.hint, arabic: true, rtl: true };
            box.view = v;
            const typed = Kit.texts(search, "مرحبا")[0];
            verify(typed, "the typed text");
            compare(typed.font.family, box.res.fam.arabic);
            compare(typed.effectiveHorizontalAlignment, Text.AlignRight);
            compare(Kit.texts(search, v.search.hint).length, 0);
            verify(box.flags & Qt.WindowDoesNotAcceptFocus, "letters come from our keyboard, never focus");
            box.visible = false;
        }

        // Ruling 33: only a click on Yes deletes; the question covers the list until it is answered.
        function test_delete_all_asks_with_the_count_and_yes_or_no_answers() {
            const box = open("confirm");
            const question = Kit.texts(box.contentItem, box.view.confirm.text)[0];
            verify(question, "the question names the count");
            verify(!button(box, "clip.insert").enabled, "the footer waits for the answer");
            compare(Board.all(box.contentItem, o => o.visible && o.place !== undefined && o.hi !== undefined && !o.Accessible.ignored).length, 0, "the list is covered");
            mouseClick(question);
            const card = Kit.only(Board.all(box.contentItem, o => o.visible && o.confirm !== undefined && o.yes !== undefined));
            mouseClick(card, 2, 2);
            compare(acts(box), [], "a click on the question's body answers nothing");
            mouseClick(button(box, "clip.no"));
            mouseClick(button(box, "clip.yes"));
            compare(acts(box), ["clipAct no 0", "clipAct yes 0"]);
            box.visible = false;
        }

        // The note takes the storage line's place, under the footer, so it adds little height.
        function test_the_note_takes_the_storage_lines_place_while_it_shows() {
            const box = open("confirm");
            const plain = open("list");
            const note = Kit.texts(box.contentItem, box.view.note)[0];
            verify(note, "the note shows");
            compare(Kit.texts(box.contentItem, box.view.room).length, 0, "the storage line gives way");
            verify(!one(box.contentItem, "clip.open-folder"), "with its button");
            const y = o => o.mapToItem(box.contentItem, 0, 0).y;
            verify(y(note) > y(button(box, "clip.insert")), "under the footer");
            verify(box.height < plain.height + note.height, "no band of its own: " + box.height + " vs " + plain.height);
            compare(Kit.texts(plain.contentItem, plain.view.room).length, 1, "without a note the storage line shows");
            box.visible = plain.visible = false;
        }

        function test_a_press_on_the_panel_is_seen_and_still_acts() {
            const box = open("list");
            const spy = createTemporaryObject(spyC, root, { target: box, signalName: "touched" });
            mouseClick(button(box, "clip.close"));
            compare(spy.count, 1, "seen, so the tray menu closes");
            compare(acts(box), ["clipAct close 0"]);
            box.visible = false;
        }

        // Every text the hub words fits in each look: none is cut, each stays inside the panel and the footer's buttons
        // never meet; row titles may end in an ellipsis by design.
        function test_no_text_is_cut_in_any_look_in_english_french_or_arabic() {
            const fonts = Board.loadFonts(root);
            tryVerify(() => Board.loaded(fonts), 5000, "the bundled fonts load");
            for (const look of [NL, EL, DL]) {
                for (const tag of ["en", "fr", "ar"]) {
                    for (const name of ["list", "confirm", "pinned", "empty", "text"])
                        fits(open(name, tag, look), look.lk.look.id + " " + tag + " " + name);
                }
            }
        }
        // The panel `box` cuts no text and its footer's buttons stand apart, named `where` in a failure.
        function fits(box, where) {
            const titles = box.view.rows.map(r => r.title);
            const shown = Board.all(box.contentItem, o => o.visible && o.font !== undefined && o.text && titles.indexOf(o.text) < 0);
            for (const t of shown) {
                const at = t.mapToItem(box.contentItem, 0, 0);
                const said = where + ": " + t.text;
                verify(!t.truncated, said + " is cut");
                verify(at.x >= 0 && at.x + t.width <= box.width + 0.5, said + " leaves the panel");
                verify(t.implicitWidth <= t.width + 0.5 || t.wrapMode !== Text.NoWrap, said + " runs past its box");
            }
            const foot = ["clip.preview", "clip.pin", "clip.delete", "clip.insert"].map(id => button(box, id));
            const at = foot.map(o => ({ o: o, x: o.mapToItem(box.contentItem, 0, 0).x })).sort((a, b) => a.x - b.x);
            for (let i = 1; i < at.length; i++)
                verify(at[i - 1].x + at[i - 1].o.width <= at[i].x + 0.5, where + ": " + at[i - 1].o.label + " meets " + at[i].o.label);
            box.visible = false;
        }
    }
}

// The key page's lines over the hub's views (ADR-0049): a typed key draws only its dots and its caret, the sealed key
// keeps its dots with the caret after them once the keys aim at it, an address reads left to right in Arabic too, a click
// places the caret, a reader's press picks its line, an empty line's hint starts at its caret, each Save dims while its
// line is empty, has a name of its own and sends its line, and each Paste sends its line, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit

Item {
    id: root
    width: 200
    height: 100

    VoiceCase {
        name: "voicecloudlines"
        host: root

        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // The caret `line` draws, or undefined while it draws none.
        function caret(line) {
            return Board.all(line, o => o.caret === true && o.visible)[0];
        }

        function test_a_typed_key_draws_only_its_dots_and_its_caret_data() {
            return langs_data();
        }
        function test_a_typed_key_draws_only_its_dots_and_its_caret(d) {
            const box = make("serviceTyped", d.t);
            const k = box.view.sheet.key;
            const key = one(box, "voice.line-key");
            verify(key.masked && key.secret, "the key never shows");
            compare(key.drawn, k.mask.charAt(0).repeat(k.typed), "the typed key's dots");
            compare(key.Accessible.description, k.mask, "readers hear the mask");
            verify(!key.takes && !caret(key), "no caret until the keys aim at it");
            box.aimed = true;
            verify(key.takes && !!caret(key), "its caret");
            compare(key.at, k.at);
            const save = one(box, "voice.save-key");
            verify(save.can, "a typed key saves");
            mouseClick(save);
            compare(acts(box), ["act save key 0 " + box.round]);
            box.visible = false;
        }

        // Hands the panel view `v`, as the hub's next view does.
        function turn(box, v) {
            box.view = v.panel;
            box.round = v.round;
            verify(waitForPolish(box), "laid out");
        }
        // Checks that `line` draws its caret after its last dot, as the dots count from the left.
        function after(line, why) {
            const text = Board.all(line, o => o.visible && o.readOnly === true)[0];
            verify(!!caret(line) && !!text, why);
            fuzzyCompare(caret(line).mapToItem(text, 0, 0).x, text.width, 1, why);
        }
        // The hints `line` shows.
        function hints(line) {
            return Board.all(line, o => o.visible && o.text === line.hint).length;
        }

        function test_aimed_a_sealed_key_keeps_its_dots_with_its_caret_after_them_data() {
            return langs_data();
        }
        // The owner's rule (2026-11-21): the sealed key's dots stay while the keys aim at its line, and the first
        // letter starts a new key whose dots replace them; Save lights only then.
        function test_aimed_a_sealed_key_keeps_its_dots_with_its_caret_after_them(d) {
            const box = make("service", d.t);
            const k = box.view.sheet.key;
            let key = one(box, "voice.line-key");
            box.aimed = true;
            verify(key.takes && key.masked, "the sealed key's dots take the letters");
            compare(key.drawn, k.mask.charAt(0).repeat(k.dots));
            after(key, "its caret after the dots");
            compare(hints(key), 0, "no hint over the dots");
            verify(!one(box, "voice.save-key").can, "Save dims while nothing is typed");
            const v = Kit.view(d.t, "serviceTyped");
            v.panel.sheet.key.typed = v.panel.sheet.key.at = 1;
            turn(box, v);
            key = one(box, "voice.line-key");
            compare(key.drawn, k.mask.charAt(0), "the new key's one dot");
            after(key, "its caret after it");
            verify(one(box, "voice.save-key").can, "a typed key saves");
            box.visible = false;
        }

        function test_aimed_a_new_sealed_key_shows_its_dots_at_once_data() {
            return langs_data();
        }
        // After Paste, Save or Enter the next view holds the new key's dots and nothing typed while the keys stay
        // aimed, and they show at once; after Remove the empty line shows its hint beside the caret.
        function test_aimed_a_new_sealed_key_shows_its_dots_at_once(d) {
            const box = make("serviceTyped", d.t);
            box.aimed = true;
            const v = Kit.view(d.t, "service");
            const k = v.panel.sheet.key;
            turn(box, v);
            let key = one(box, "voice.line-key");
            verify(key.takes, "the keys stay aimed");
            compare(key.drawn, k.mask.charAt(0).repeat(k.dots), "the new key's dots");
            after(key, "its caret after them");
            const gone = Kit.view(d.t, "service");
            gone.panel.sheet.key.dots = gone.panel.sheet.key.at = 0;
            turn(box, gone);
            key = one(box, "voice.line-key");
            verify(key.takes && !key.masked && key.drawn === "", "an empty line takes the letters");
            compare(hints(key), 1, "its hint beside the caret");
            verify(!one(box, "voice.save-key").can, "Save dims while nothing is typed");
            box.visible = false;
        }

        function test_an_address_reads_left_to_right_and_a_click_places_its_caret_data() {
            return langs_data();
        }
        function test_an_address_reads_left_to_right_and_a_click_places_its_caret(d) {
            const box = make("serviceCustom", d.t);
            const a = box.view.sheet.address;
            const line = one(box, "voice.line-address");
            box.aimed = true;
            verify(line.takes && !line.wrote, "left to right in every language");
            compare(line.at, a.at);
            verify(caret(line).mapToItem(line, 0, 0).x < line.width / 2, "after its scheme, on the left");
            compare(one(box, "voice.line-model").takes, false, "one line takes the letters");
            mouseClick(line, 1, line.height / 2);
            mouseClick(line, line.width - 1, line.height / 2);
            const act = (kind, n) => "act " + kind + " address " + n + " " + box.round;
            const end = a.text.length;
            const sent = [act("pick", end), act("caret", 0), act("pick", end), act("caret", end)];
            compare(acts(box), sent, "each click picks the line, then places its caret");
            box.visible = false;
        }

        function test_each_save_sends_its_line_data() {
            return langs_data();
        }
        function test_each_save_sends_its_line(d) {
            const box = make("serviceCustom", d.t);
            verify(!one(box, "voice.save-key").can, "no key typed");
            mouseClick(one(box, "voice.save-address"));
            mouseClick(one(box, "voice.save-model"));
            const sent = ["address", "model"].map(id => "act save " + id + " 0 " + box.round);
            compare(acts(box), sent);
            const save = box.view.sheet.model.buttons.find(b => b.kind === "save");
            compare(one(box, "voice.save-model").Accessible.name, save.name);
            box.visible = false;
        }

        function test_each_lines_paste_is_drawn_as_the_keys_and_sends_its_line_data() {
            return langs_data();
        }
        // The owner's rule (2026-11-21): the address and model lines take the clipboard as the key line does.
        function test_each_lines_paste_is_drawn_as_the_keys_and_sends_its_line(d) {
            const box = make("serviceCustom", d.t);
            const ids = ["address", "model", "key"];
            const pastes = ids.map(id => one(box, "voice.paste-" + id));
            const key = pastes[2];
            for (const p of pastes) {
                compare([p.icon, p.label, p.width, p.height], [key.icon, key.label, key.width, key.height], "as the key's");
                verify(p.can, "it always acts");
                mouseClick(p);
            }
            compare(acts(box), ids.map(id => "act paste " + id + " 0 " + box.round));
            box.visible = false;
        }

        function test_a_readers_press_on_a_line_picks_it_data() {
            return langs_data();
        }
        function test_a_readers_press_on_a_line_picks_it(d) {
            const box = make("serviceCustom", d.t);
            const s = box.view.sheet;
            one(box, "voice.line-model").aim();
            one(box, "voice.line-address").aim();
            const at = (id, n) => "act pick " + id + " " + n + " " + box.round;
            compare(acts(box), [at("model", s.model.text.length), at("address", s.address.text.length)], "at its end");
            box.visible = false;
        }

        function test_each_save_has_its_own_name_data() {
            return langs_data();
        }
        function test_each_save_has_its_own_name(d) {
            for (const [page, ids] of [["service", ["key", "model"]], ["serviceCustom", ["key", "address", "model"]]]) {
                const box = make(page, d.t);
                const saves = ids.map(id => one(box, "voice.save-" + id));
                const names = saves.map(b => b.Accessible.name);
                compare(new Set(names).size, names.length, page + ": " + names);
                verify(saves.every(b => b.label === saves[0].label && b.Accessible.name !== b.label), "a short label, a full name");
                box.visible = false;
            }
        }

        // A left-to-right line's hint starts at its start, where the caret waits, as a text box shows its placeholder.
        function test_an_empty_left_to_right_line_shows_its_hint_from_the_caret_in_arabic() {
            const v = Kit.view("ar", "serviceCustom");
            v.panel.sheet.address.text = v.panel.sheet.model.text = "";
            const box = show(v, "ar");
            box.aimed = true;
            const hintOf = line => Board.all(line, o => o.visible && o.text === line.hint)[0];
            for (const id of ["address", "model"]) {
                const hint = hintOf(one(box, "voice.line-" + id));
                compare(hint.effectiveHorizontalAlignment, Text.AlignLeft, id + "'s hint on the left");
            }
            const line = one(box, "voice.line-address");
            const [mark, hint] = [caret(line).mapToItem(line, 0, 0).x, hintOf(line).mapToItem(line, 0, 0).x];
            verify(mark < hint && hint < line.width / 2, "the caret, then the hint, from the left");
            box.visible = false;
        }
    }
}

// The key page's lines over the hub's views (ADR-0049): a typed key draws only its dots and its caret, the sealed key
// gives way to an empty line once the keys aim at it, an address reads left to right in Arabic too, a click places the
// caret, and each Save dims while its line is empty and sends its line, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board

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

        function test_aimed_the_sealed_key_gives_way_to_an_empty_line_data() {
            return langs_data();
        }
        function test_aimed_the_sealed_key_gives_way_to_an_empty_line(d) {
            const box = make("service", d.t);
            const key = one(box, "voice.line-key");
            verify(key.masked, "the sealed key's dots");
            box.aimed = true;
            verify(key.takes && !key.masked && key.drawn === "", "an empty line takes the letters");
            compare(Board.all(key, o => o.visible && o.text === key.hint).length, 1, "its hint beside the caret");
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
            const at = n => "act caret address " + n + " " + box.round;
            compare(acts(box), [at(0), at(a.text.length)], "at its start, then past its end");
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
            compare(one(box, "voice.save-model").Accessible.name, box.view.sheet.model.buttons[0].label);
            box.visible = false;
        }
    }
}

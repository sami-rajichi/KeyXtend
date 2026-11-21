// The cloud pages as the approved mock-up draws them (design/voice-cloud.css): Remove key in the footer in the danger
// look, the switch band's hover tint, Untested as an outlined tag beside a service's name, Key works as a lit mark, a
// refused answer or line as a warning band, and Models' side chips in their places in Arabic too (ADR-0020).
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "voicekit.js" as Kit
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    VoiceCase {
        name: "voicecloudlook"
        host: root

        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // `o`'s top left corner in the window.
        function at(o) {
            return o.mapToItem(null, 0, 0);
        }
        // The words of the warning bands under `box`, top to bottom.
        function bands(box) {
            return Board.all(box.contentItem, o => o.visible && o.warn === true && o.pieces !== undefined).map(b => b.Accessible.name);
        }

        function test_remove_key_sits_in_the_footer_in_the_key_pages_danger_look_data() {
            return langs_data();
        }
        // The ticked row's Remove key (owner, 2026-11-21) looks as the key page's does (mock-up rmkey); no row has one.
        function test_remove_key_sits_in_the_footer_in_the_key_pages_danger_look(d) {
            const box = make("cloudSealed", d.t);
            const remove = one(box, "voice.remove", box.foot);
            compare([remove.tone, remove.can], ["danger", true]);
            const key = make("service", d.t);
            compare(one(key, "voice.remove").icon, remove.icon, "the key page's icon");
            verify(rows(box).every(r => buttons(r).length === 0), "no button on a row");
            box.visible = key.visible = false;
        }

        function test_the_switch_band_takes_the_hover_tint() {
            const box = make("cloud");
            const sw = one(box, "voice.cloud");
            const tints = () => Board.all(sw, o => o.visible && o.tint !== undefined && o.icon === undefined).length;
            compare(tints(), 0, "no tint at rest");
            mouseMove(sw, sw.width / 2, sw.height / 2);
            tryCompare(sw, "hot", true);
            compare(tints(), 1, "the hover tint under the pointer");
            box.visible = false;
        }

        function test_untested_is_an_outlined_tag_beside_the_name_data() {
            return langs_data();
        }
        function test_untested_is_an_outlined_tag_beside_the_name(d) {
            const box = make("cloud", d.t);
            const row = rows(box)[0];
            const s = box.view.sheet.rows[0];
            compare(marks(row).map(m => m.markId), s.marks.map(m => m.id), "Untested is no mark at the row's end");
            const word = Kit.only(Kit.texts(row, s.tag));
            verify(word, "the tag shows once");
            const tag = word.parent;
            verify(tag.border.width > 0 && Qt.colorEqual(tag.border.color, N.lk.palette.pop_line), "outlined");
            compare(tag.Accessible.name, s.tag, "readers hear it");
            const name = row.nameText;
            const [n, t] = [at(name), at(tag)];
            verify(t.y < n.y + name.height && n.y < t.y + tag.height, "on the name's line");
            if (d.t === "ar")
                verify(t.x + tag.width <= n.x + name.width - name.contentWidth + 0.5, "after the name, reading from the right");
            else
                verify(t.x >= n.x + name.contentWidth - 0.5, "after the name");
            box.visible = false;
        }

        function test_key_works_shows_as_the_lit_mark_data() {
            return langs_data();
        }
        function test_key_works_shows_as_the_lit_mark(d) {
            const box = make("service", d.t);
            const s = box.view.sheet;
            const mark = Kit.only(Board.all(box.contentItem, o => o.visible && o.markId !== undefined && o.Accessible.name === s.check.text));
            verify(mark, "Key works as a mark");
            compare([mark.tone, mark.icon], ["lit", N.lk.icons.voice.panel.cloud.ok]);
            compare(bands(box), [], "no warning");
            box.visible = false;
        }

        function test_a_refused_answer_or_line_shows_as_the_warning_band_data() {
            return langs_data();
        }
        function test_a_refused_answer_or_line_shows_as_the_warning_band(d) {
            const key = make("serviceLong", d.t);
            compare(bands(key), [key.view.sheet.check.text], "the key's answer");
            const custom = make("serviceCustom", d.t);
            const c = custom.view.sheet;
            compare(bands(custom), [c.address.fault, c.model.fault], "each refused line");
            key.visible = custom.visible = false;
        }

        function test_the_side_chips_keep_their_places_in_every_language_data() {
            return langs_data();
        }
        function test_the_side_chips_keep_their_places_in_every_language(d) {
            const box = make("modelsCloud", d.t);
            const [local, cloud] = [at(one(box, "voice.side-local")), at(one(box, "voice.side-cloud"))];
            verify(local.x < cloud.x, "Local, then Cloud");
            const caption = Kit.only(Kit.texts(box.contentItem, box.view.sheet.sides.name));
            verify(at(caption).x < local.x, "the caption first");
            box.visible = false;
        }
    }
}

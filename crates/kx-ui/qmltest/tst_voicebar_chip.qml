// The bar's chip over the hub's views (owner, 2026-11-22): while a model on this PC listens or writes, the processor's
// picture and the model's short name, which readers hear in a sentence, as the cloud mark names its service; a user's
// own long file name cut in the middle at the theme's width; and the bar holding the longest short name in all eight
// looks, in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer
import "fixtures/voice.js" as Voice
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as NL
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NH
import "../../kx-theme/fixtures/look-native-light-reduced.js" as NR
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's box the bar is placed by, as the app hands it (JSON, physical pixels).
    readonly property string over: JSON.stringify({ x: 100, y: 600, w: 900, h: 300 })
    readonly property var texts: ({ en: En, fr: Fr, ar: Ar })
    readonly property var looks: ({ "native-light": NL, "native-dark": ND, "native-contrast": NH, "native-reduced": NR, "et66-light": EL, "et66-dark": ED, "dolch-light": DL, "dolch-dark": DD })
    // The views whose chip names the model on this PC with the table's longest short name.
    readonly property var device: ["deviceListening", "deviceTranscribing"]

    Component { id: voiceC; FakeVoice {} }
    Component { id: barC; VoiceBar {} }

    TestCase {
        name: "voicebar_chip"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // A shown bar over the hub's view `name` in language `tag` and look `lk` (Native light by default).
        function make(name, tag, lk) {
            const look = lk || NL.lk;
            const voice = createTemporaryObject(voiceC, root, { page: null });
            const bar = createTemporaryObject(barC, root, {
                voice: voice, lk: look, res: Board.resOf(look, root.texts[tag]), s: 1, ar: tag === "ar",
                box: () => root.over, logical: (x, y) => ({ x: x, y: y })
            });
            Pointer.through([bar]);
            voice.page = JSON.parse(JSON.stringify(Voice.views[tag][name]));
            tryVerify(() => bar.visible, 2000, "it shows");
            waitForRendering(bar.contentItem);
            return bar;
        }
        // The shown parts readers hear as `name`.
        function heard(bar, name) {
            return Board.all(bar.contentItem, o => o.visible && !o.Accessible.ignored && o.Accessible.name === name);
        }
        // The chip's name on `bar`, shown once.
        function chipName(bar) {
            const name = bar.view.mark.name;
            const all = Board.all(bar.contentItem, o => o.visible && o.text === name && o.font !== undefined);
            compare(all.length, 1, "the chip's name shows once");
            return all[0];
        }
        // The shown icons on `bar` named `icon`.
        function icons(bar, icon) {
            return Board.all(bar.contentItem, o => o.visible && o.icon === icon && o.tint !== undefined);
        }

        function test_the_chip_names_the_model_on_this_pc_while_it_listens_or_writes_data() {
            return langs_data();
        }
        function test_the_chip_names_the_model_on_this_pc_while_it_listens_or_writes(d) {
            const fam = Board.resOf(NL.lk, root.texts[d.t]).fam;
            for (const name of root.device) {
                const bar = make(name, d.t);
                const mark = bar.view.mark;
                compare(heard(bar, mark.said).length, 1, name + ": readers hear who writes");
                const label = chipName(bar);
                verify(!label.truncated, name + ": the short name whole");
                compare(label.font.family, d.t === "ar" ? fam.arabic : fam.latin, "in the language's own font, as the cloud mark");
                compare(icons(bar, NL.lk.icons.voice.device).length, 1, name + ": the processor's picture");
                compare(icons(bar, NL.lk.icons.voice.cloud).length, 0, name + ": no cloud");
                bar.visible = false;
            }
        }

        function test_a_long_own_file_name_is_cut_in_the_middle_data() {
            return langs_data();
        }
        // Past the theme's width a user's own file name keeps its start and its end, while readers hear it whole.
        function test_a_long_own_file_name_is_cut_in_the_middle(d) {
            const bar = make("deviceOwn", d.t);
            const mark = bar.view.mark;
            const label = chipName(bar);
            compare(label.elide, Text.ElideMiddle);
            verify(label.truncated, "cut");
            fuzzyCompare(label.width, NL.lk.shape.voice.chip_name_px * bar.s, 0.5, "at the theme's width");
            compare(heard(bar, mark.said).length, 1, "readers hear it");
            verify(mark.said.indexOf(mark.name) >= 0, "its whole name in the sentence");
            bar.visible = false;
        }

        function test_the_bar_holds_the_longest_short_name_in_every_look_data() {
            const all = [];
            for (const [tag, f] of Object.entries(root.looks))
                for (const name of root.device.concat(["deviceOwn", "cloudListening"]))
                    ["en", "fr", "ar"].forEach(t => all.push({ tag: tag + " " + name + " " + t, lk: f.lk, name, t }));
            return all;
        }
        // Every part sits inside the bar, and the wave or the shimmer keeps its least room: the bar widens for it, never
        // past its text width, while the cloud mark's bar keeps the width it had.
        function test_the_bar_holds_the_longest_short_name_in_every_look(d) {
            const bar = make(d.name, d.t, d.lk);
            const label = chipName(bar);
            if (d.name !== "deviceOwn")
                verify(!label.truncated, "the name whole");
            const chip = label.parent.parent;
            const at = chip.mapToItem(bar.body, 0, 0);
            verify(at.x >= 0 && at.x + chip.width <= bar.body.width + 0.5, "the chip inside the bar");
            const v = d.lk.shape.voice;
            const room = v.wave_room_px * bar.s;
            verify(bar.wave.width >= room - 0.5, "the middle keeps " + room + " px, has " + bar.wave.width);
            verify(bar.width <= Math.ceil(v.bar_px[1] * bar.s), "never past the text's width");
            if (d.name === "cloudListening")
                compare(bar.width, Math.ceil(v.bar_px[0] * bar.s), "the cloud mark's bar as before");
            const shown = Board.all(bar.contentItem, o => o.visible && o.Accessible.role === Accessible.Button);
            for (const b of shown) {
                const x = b.mapToItem(bar.body, 0, 0).x;
                verify(x >= 0 && x + b.width <= bar.body.width + 0.5, b.Accessible.id + " inside the bar");
                verify(x >= at.x + chip.width - 0.5 || x + b.width <= at.x + 0.5, b.Accessible.id + " clear of the chip");
            }
            bar.visible = false;
        }
    }
}

// The dictation bar's cloud parts over the hub's views (ADR-0047): the cloud mark with its service while a service takes
// the voice, which readers hear as a sentence, and a cloud note's Try again, Use the local model dimmed unless one is
// ready, and Download when none is (edge 20), in English, French and Arabic.
import QtQuick
import QtTest
import "../qml"
import "board.js" as Board
import "pointer.js" as Pointer
import "fixtures/voice.js" as Voice
import "fixtures/ui-en.js" as En
import "fixtures/ui-fr.js" as Fr
import "fixtures/ui-ar.js" as Ar
import "../../kx-theme/fixtures/look-native-light.js" as N

Item {
    id: root
    width: 200
    height: 100

    // The keyboard's box the bar is placed by, as the app hands it (JSON, physical pixels).
    readonly property string over: JSON.stringify({ x: 100, y: 600, w: 900, h: 300 })
    readonly property var texts: ({ en: En, fr: Fr, ar: Ar })

    Component { id: voiceC; FakeVoice {} }
    Component { id: barC; VoiceBar {} }

    TestCase {
        name: "voicecloud_bar"
        when: windowShown

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function langs_data() {
            return ["en", "fr", "ar"].map(t => ({ tag: t, t: t }));
        }
        // A shown bar over a stand-in Voice object for the hub's view `name` in language `tag`.
        function make(name, tag) {
            const v = JSON.parse(JSON.stringify(Voice.views[tag][name]));
            const voice = createTemporaryObject(voiceC, root, { page: null });
            const bar = createTemporaryObject(barC, root, {
                voice: voice, lk: N.lk, res: Board.resOf(N.lk, root.texts[tag]), s: 1, ar: tag === "ar",
                box: () => root.over, logical: (x, y) => ({ x: x, y: y })
            });
            Pointer.through([bar]);
            voice.page = v;
            tryVerify(() => bar.visible, 2000, "it shows");
            waitForRendering(bar.contentItem);
            return bar;
        }
        // The bar's shown buttons, left to right on screen.
        function buttons(bar) {
            const all = Board.all(bar.contentItem, o => o.Accessible.role === Accessible.Button && o.visible);
            return all.sort((a, b) => a.mapToItem(null, 0, 0).x - b.mapToItem(null, 0, 0).x);
        }
        // The shown parts readers hear as `name`.
        function heard(bar, name) {
            return Board.all(bar.contentItem, o => o.visible && !o.Accessible.ignored && o.Accessible.name === name);
        }

        function test_the_cloud_mark_names_its_service_only_while_one_takes_the_voice_data() {
            return langs_data();
        }
        function test_the_cloud_mark_names_its_service_only_while_one_takes_the_voice(d) {
            const bar = make("cloudListening", d.t);
            const mark = bar.view.cloud;
            compare(heard(bar, mark.said).length, 1, "readers hear where the voice goes");
            const name = Board.all(bar.contentItem, o => o.visible && o.text === mark.name && o.font !== undefined);
            compare(name.length, 1, "the service's name");
            const icon = Board.all(bar.contentItem, o => o.visible && o.icon === N.lk.icons.voice.cloud && o.tint !== undefined);
            compare(icon.length, 1, "the cloud mark");
            const local = make("listening", d.t);
            compare(heard(local, mark.said).length, 0, "a local dictation shows no cloud mark");
            bar.visible = local.visible = false;
        }

        function test_a_cloud_note_offers_the_local_model_dimmed_with_download_when_none_is_ready_data() {
            return langs_data();
        }
        function test_a_cloud_note_offers_the_local_model_dimmed_with_download_when_none_is_ready(d) {
            const ids = name => buttons(make(name, d.t)).map(b => b.Accessible.id);
            const gear = "voice.gear";
            compare(ids("cloudKey"), ["voice.again", "voice.local", gear, "voice.close"]);
            compare(ids("cloudNet"), ["voice.again", "voice.local", "voice.download", gear, "voice.close"], "edge 20");
            compare(ids("cloudNoLocal"), ["voice.local", gear, "voice.close"]);
            const ready = make("cloudKey", d.t);
            const use = buttons(ready).find(b => b.Accessible.id === "voice.local");
            verify(use.enabled, "a local model is ready");
            compare(use.icon, N.lk.icons.voice.local, "its own icon");
            mouseClick(use);
            compare(ready.voice.calls, ["act local local 0 " + ready.view.round]);
            const none = make("cloudNet", d.t);
            const dim = buttons(none).find(b => b.Accessible.id === "voice.local");
            verify(!dim.enabled, "dimmed, never hidden");
            compare(dim.opacity, N.lk.shape.panel.off_share);
            mouseClick(dim);
            compare(none.voice.calls, [], "a dimmed button sends nothing");
            ready.visible = none.visible = false;
        }
    }
}

// The hold ring (spec §3.2), moved from the P1 test build: after its wait the waves shrink in over the rest of the hold,
// once; a fire bursts once, reduced motion keeps the waves and skips the burst, and a cleared ring hides at once.
import QtQuick
import QtTest
import "../qml"
import "../../kx-theme/fixtures/look-native-light.js" as F
import "../../kx-theme/fixtures/look-native-light-reduced.js" as R
import "../../kx-theme/fixtures/look-native-light-contrast.js" as NC
import "../../kx-theme/fixtures/look-native-dark.js" as ND
import "../../kx-theme/fixtures/look-et66-light.js" as EL
import "../../kx-theme/fixtures/look-et66-dark.js" as ED
import "../../kx-theme/fixtures/look-dolch-light.js" as DL
import "../../kx-theme/fixtures/look-dolch-dark.js" as DD

Item {
    id: root
    width: 200
    height: 200

    Component {
        id: ringC
        HoldRing {}
    }
    SignalSpy {
        id: bursts
        signalName: "burstChanged"
    }
    SignalSpy {
        id: busies
        signalName: "busyChanged"
    }

    TestCase {
        name: "HoldRing"
        when: windowShown

        // How late a run may end on a busy machine; it never ends early, as its clock is real time.
        readonly property int lateMs: 1000

        // Any warning fails the test, so the run stays clean.
        function init() {
            failOnWarning(/./);
        }
        function make(look) {
            return createTemporaryObject(ringC, root, { lk: look });
        }
        // The first wave, the second, the core, the burst ring and the dots.
        function parts(f) {
            const c = f.children;
            const dots = [];
            for (let i = 4; i < c.length; i++)
                if (c[i].index !== undefined)
                    dots[c[i].index] = c[i];
            return { w1: c[0], w2: c[1], core: c[2], ring: c[3], dots: dots };
        }
        function far(f, d) {
            return Math.hypot(d.x + d.width / 2 - f.width / 2, d.y + d.height / 2 - f.height / 2);
        }
        // Whether a wave or the core shows.
        function waving(p) {
            return [p.w1, p.w2, p.core].some(o => o.opacity > 0);
        }
        // Plays waves after `after` ms over `len` ms: the wait, then the first wave and the core over the rest of the hold
        // and the second wave ending with them, once; nothing shows before the wait nor ends before the hold.
        function hold(after, len) {
            const f = make(F.lk);
            const p = parts(f);
            const t0 = Date.now();
            f.wave(after, len);
            verify(!waving(p), "nothing at once");
            const [wait, run] = f.waves.animations;
            const [first, second, grow, fade] = run.animations;
            compare([wait.duration, first.len, grow.duration, fade.duration, f.waves.loops], [after, len, len, len, 1]);
            fuzzyCompare(second.animations[0].duration + second.animations[1].len, len, 1, "the second wave ends too, in whole ms");
            tryVerify(() => waving(p), after + lateMs, "the waves show");
            verify(Date.now() - t0 >= after, "not before the wait");
            tryVerify(() => !f.waves.running, after + len + lateMs, "the waves end");
            verify(Date.now() - t0 >= after + len, "not before the hold's time");
            fuzzyCompare(p.w1.scale, F.lk.motion.ring.amount.wave_end, 0.001);
            verify(waving(p) && f.busy, "the last frame stays until the hold ends or fires");
            return f;
        }

        function test_the_ring_takes_its_box_and_rests_hidden() {
            const f = make(F.lk);
            const p = parts(f);
            compare(f.width, F.lk.shape.ring.box_px);
            compare(p.dots.length, F.lk.shape.ring.dots);
            for (const o of [p.w1, p.w2, p.core, p.ring].concat(p.dots))
                compare(o.opacity, 0);
            verify(!f.busy);
            compare(p.w1.border.width, F.lk.shape.ring.wave_px[0]);
            compare(p.w2.border.width, F.lk.shape.ring.wave_px[1]);
            fuzzyCompare(p.core.width, F.lk.shape.ring.box_px * F.lk.shape.ring.core_share, 0.001);
        }

        function test_every_theme_draws_the_ring_in_its_ring_colour() {
            for (const look of [F.lk, R.lk, NC.lk, ND.lk, EL.lk, ED.lk, DL.lk, DD.lk]) {
                const p = parts(make(look));
                const tint = look.palette.ring;
                const id = look.look.id + (look.dark ? " dark" : "") + (look.contrast ? " contrast" : "");
                for (const o of [p.w1, p.w2, p.ring])
                    verify(Qt.colorEqual(o.border.color, tint), id);
                for (const o of [p.core].concat(p.dots))
                    verify(Qt.colorEqual(o.color, tint), id);
            }
        }

        function test_the_burst_ring_grows_and_fades_and_the_dots_fly_out_evenly() {
            const f = make(F.lk);
            const p = parts(f), sh = F.lk.shape.ring, am = F.lk.motion.ring.amount;
            f.burst = 0;
            fuzzyCompare(p.ring.scale, am.burst_scale[0], 0.001);
            compare(p.ring.opacity, 1);
            for (let i = 0; i < p.dots.length; i++) {
                const d = p.dots[i];
                fuzzyCompare(far(f, d), sh.fly_px[0], 0.01);
                const turn = Math.atan2(d.y + d.height / 2 - f.height / 2, d.x + d.width / 2 - f.width / 2);
                const want = 2 * Math.PI * i / sh.dots;
                fuzzyCompare(Math.cos(turn), Math.cos(want), 0.001);
                fuzzyCompare(Math.sin(turn), Math.sin(want), 0.001);
            }
            f.burst = 1;
            fuzzyCompare(p.ring.scale, am.burst_scale[1], 0.001);
            compare(p.ring.opacity, 0);
            fuzzyCompare(far(f, p.dots[0]), sh.fly_px[1], 0.01);
            fuzzyCompare(p.dots[0].scale, am.dot_end, 0.001);
            verify(p.ring.width * am.burst_scale[1] <= f.width, "the burst stays in the window");
        }

        // The owner's hold of 1.5 s: the ring shows after 150 ms and its waves run the 1350 ms left.
        function test_after_its_wait_the_ring_runs_once_to_the_hold_time() {
            hold(150, 1350);
        }

        // The shortest hold, 500 ms: the waves run the 350 ms left after the wait.
        function test_a_short_hold_ends_its_waves_at_its_own_time() {
            hold(150, 350);
        }

        function test_the_second_wave_waits_for_its_share_of_the_time() {
            const f = make(F.lk);
            const p = parts(f);
            f.wave(0, 2000);
            tryVerify(() => p.w1.opacity > 0, 500);
            compare(p.w2.opacity, 0, "the second wave has not started");
            tryVerify(() => p.w2.opacity > 0, 1000, "then it starts");
            verify(p.w2.scale > p.w1.scale, "and trails the first");
        }

        function test_a_fire_hides_the_waves_and_bursts_once() {
            const f = make(F.lk);
            const p = parts(f);
            f.wave(0, 200);
            tryVerify(() => waving(p), 1000);
            f.fire();
            verify(!waving(p), "the waves go at once");
            tryVerify(() => p.ring.opacity > 0, 1000, "the burst follows");
            tryVerify(() => f.burst === 1 && !f.busy, 1000, "the burst ends");
            verify(!f.waves.running && !waving(p), "nothing plays again");
        }

        function test_reduced_motion_keeps_the_waves_but_skips_the_burst() {
            const f = make(R.lk);
            const p = parts(f);
            compare(R.lk.motion.moves.burst.ms, 0);
            bursts.clear();
            bursts.target = f;
            f.wave(0, 300);
            tryVerify(() => p.w1.scale < 0.5, 1000, "the waves still take the hold's time");
            f.fire();
            verify(!waving(p) && !f.busy, "the waves go and nothing waits");
            compare(bursts.count, 0, "the burst never shows");
        }

        // One run after another keeps `busy` on through the call, so the overlay window never hides and shows again.
        function test_busy_stays_on_from_one_run_to_the_next() {
            const f = make(F.lk);
            f.wave(0, 1000);
            busies.clear();
            busies.target = f;
            f.fire();
            f.fire();
            f.wave(0, 1000);
            f.fire();
            verify(f.busy, "the last burst plays");
            compare(busies.count, 0, "busy never dropped");
            f.clear();
            verify(!f.busy && busies.count === 1, "a clear ends it once");
        }

        function test_a_cleared_ring_hides_at_once() {
            const f = make(F.lk);
            const p = parts(f);
            f.wave(0, 1000);
            tryVerify(() => waving(p), 1000);
            f.clear();
            verify(!waving(p) && !f.busy, "the waves");
            f.fire();
            tryVerify(() => p.ring.opacity > 0, 1000);
            f.clear();
            verify(p.ring.opacity === 0 && !f.busy, "the burst");
            f.fire();
            tryVerify(() => p.ring.opacity > 0, 1000);
            f.wave(0, 1000);
            compare(p.ring.opacity, 0, "a new hold ends a burst at once");
        }
    }
}

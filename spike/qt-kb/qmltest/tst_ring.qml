// Offline check of the G12 ring: waves over the hold, a burst after, the loop, reduced motion and the burst's geometry.
import QtQuick
import QtTest
import "../qml"
import "look-ring-full.js" as F
import "look-ring-reduced.js" as R

Item {
    id: root
    width: 200
    height: 200

    Component {
        id: faceC
        RingFace {
            afterMs: 50
            waveMs: 600
            gapMs: 300
        }
    }
    SignalSpy { id: bursts; signalName: "burstChanged" }

    TestCase {
        name: "RingFace"
        when: windowShown

        function make(look, extra) {
            const props = Object.assign({ lk: look }, extra || {});
            return createTemporaryObject(faceC, root, props);
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

        function test_the_ring_takes_its_box_and_rests_hidden() {
            const f = make(F.lk);
            const p = parts(f);
            compare(f.width, F.lk.shape.ring.box_px);
            compare(p.dots.length, F.lk.shape.ring.dots);
            for (const o of [p.w1, p.w2, p.core, p.ring].concat(p.dots))
                compare(o.opacity, 0);
            compare(p.w1.border.width, F.lk.shape.ring.wave_px[0]);
            compare(p.w2.border.width, F.lk.shape.ring.wave_px[1]);
            fuzzyCompare(p.core.width, F.lk.shape.ring.box_px * F.lk.shape.ring.core_share, 0.001);
            verify(Qt.colorEqual(p.core.color, F.lk.palette.ring));
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

        function test_waves_shrink_in_then_hide_for_the_burst_and_the_loop_repeats() {
            const f = make(F.lk);
            const p = parts(f);
            f.loop.start();
            tryVerify(() => p.w1.scale < 0.5 && p.core.opacity > 0.3, 1000, "the waves shrink and the core shows");
            tryVerify(() => p.ring.opacity > 0, 1000, "the burst follows the waves");
            compare(p.w1.opacity, 0);
            compare(p.w2.opacity, 0);
            compare(p.core.opacity, 0);
            tryVerify(() => f.burst === 1 && p.w1.opacity === 0, 1000, "the burst ends");
            tryVerify(() => p.w1.opacity > 0, 1500, "the loop starts again");
            f.loop.stop();
        }

        function test_the_second_wave_waits_for_its_share_of_the_time() {
            const f = make(F.lk, { waveMs: 2000 });
            const p = parts(f);
            f.loop.start();
            tryVerify(() => p.w1.opacity > 0, 500);
            compare(p.w2.opacity, 0, "the second wave has not started");
            tryVerify(() => p.w2.opacity > 0, 1000, "then it starts");
            verify(p.w2.scale > p.w1.scale, "and trails the first");
            f.loop.stop();
        }

        function test_reduced_motion_keeps_the_waves_but_skips_the_burst() {
            const f = make(R.lk);
            const p = parts(f);
            compare(R.lk.motion.moves.burst.ms, 0);
            bursts.clear();
            bursts.target = f;
            f.loop.start();
            tryVerify(() => p.w1.scale < 0.5, 1000, "the waves still take the hold's time");
            tryVerify(() => p.w1.opacity === 0 && p.core.opacity === 0, 1000);
            tryVerify(() => p.w1.opacity > 0, 1000, "the loop starts again");
            compare(bursts.count, 0, "the burst never shows");
            f.loop.stop();
        }
    }
}

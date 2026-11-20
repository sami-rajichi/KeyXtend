// Voice typing mock-up (design/voice-bar.html): the app and keyboard behind the bar, the controls, placing, motion and
// clicks. Part 2's voice-cloud-data.js and voice-cloud.js load after it and before boot(). Reference only.
'use strict';

// The scene: the app behind and the keyboard (selection-chips.html), with the microphone key lit while listening.
function app() {
  const a = l.app, dir = l.ar ? ' dir="rtl"' : '', cls = l.ar ? ' class="ar"' : '';
  if (S.bar === 'password') return '<div class="app"><div class="a-t"' + cls + '>' + a[3] + '</div><div class="a-b"' + dir + cls + '><h3 style="margin:0 0 14px">' + a[3] + '</h3>' +
    '<div class="a-f"><span>' + a[4] + '</span><span dir="ltr">maya@example.org</span></div><div class="a-f"><span>' + a[5] + '</span><span>' + (S.undone ? '' : '••••••••••') + '<span class="caret"></span></span></div></div></div>';
  const typed = S.bar === 'text' && !S.undone ? ' <span class="new">' + l.said + '</span>' : '';
  return '<div class="app"><div class="a-t"' + cls + '>' + a[0] + '</div><div class="a-b"' + dir + cls + '><p>' + a[1] + '</p><p>' + a[2] + typed + S.extra + '<span class="caret"></span></p></div></div>';
}
const keyEl = (w, cls, inner) => '<div class="key ' + cls + '" style="grid-column:span ' + Math.round(w * 4) + '"><div class="cap">' + inner + '</div></div>';
// The Dictation key works in the mock-up; while preparing, its Narrator name says so.
const micAct = () => ' data-a="mic" role="button" tabindex="0" aria-label="' + (S.bar === 'preparing' ? l.namePrep : l.title) + '"';
function keyboard() {
  const k = KB[S.lang], ar = S.lang === 'ar', lb = s => '<span class="lbl">' + s + '</span>';
  const row = a => a.map(s => keyEl(1, 'char', '<span class="lg' + (ar && /[\u0600-\u06FF]/.test(s) ? ' ar' : '') + '">' + s.replace('<', '&lt;') + '</span>')).join('');
  const side = [['mouse-right',1,1,'act'],['hand-grab',2,1,'act'],['command',3,1,'act'],['clipboard-clock',3,3,'act'],['key-round',1,5,'act'],['mic',2,5,S.bar === 'listening' ? 'rec' : S.bar === 'preparing' ? 'prep' : 'act'],['power',3,5,'danger']];
  return '<div class="kb"><div class="kb-top"><div class="lane">' + k.words.map(w => '<span class="wchip' + (ar ? ' ar' : '') + '">' + w + '</span>').join('') + '</div><span class="tb">' + ic('minus') + '</span><span class="tb">' + ic('x') + '</span></div>' +
    '<div class="kb-body"><div class="kb-main">' +
    '<div class="row">' + keyEl(1, 'danger', lb(k.keys[0])) + row(k.rows[0]) + keyEl(2, 'danger', ic('delete')) + '</div>' +
    '<div class="row">' + keyEl(1.5, 'mod', ic('arrow-right-to-line')) + row(k.rows[1]) + keyEl(1.5, 'mod', lb(k.keys[1])) + '</div>' +
    '<div class="row">' + keyEl(1.75, 'mod', lb(k.keys[2])) + row(k.rows[2]) + keyEl(3.25, 'enter', ic('corner-down-left')) + '</div>' +
    '<div class="row">' + keyEl(1.25, 'mod', ic('arrow-big-up')) + row(k.rows[3]) + keyEl(1.75, 'mod', ic('arrow-big-up')) + keyEl(1, 'mod', ic('chevron-up')) + keyEl(1, 'mod', ic('square-menu')) + '</div>' +
    '<div class="row">' + keyEl(1.25, 'mod', lb(k.keys[3])) + keyEl(1, 'mod', ic('win-logo')) + keyEl(1, 'mod', lb(k.keys[4])) + keyEl(1.75, 'lang', '<span class="lbl' + (ar ? ' ar' : '') + '">' + k.lang + '</span>') +
    keyEl(7, 'space', '<span class="spn' + (ar ? ' ar' : '') + '">' + k.sp + '</span>') + keyEl(1, 'mod', lb(k.keys[5])) + keyEl(1, 'mod', ic('chevron-left')) + keyEl(1, 'mod', ic('chevron-down')) + keyEl(1, 'mod', ic('chevron-right')) + '</div>' +
    '</div><div class="kb-side"><div class="key act" style="grid-column:1/3;grid-row:2/5"><div class="cap dpad"><span class="d up">' + ic('chevron-up') + '</span><span class="d lf">' + ic('chevron-left') +
    '</span><span class="d rt">' + ic('chevron-right') + '</span><span class="d bt">' + ic('chevron-down') + '</span></div></div>' +
    side.map(([n, cx, ry, c]) => '<div class="key ' + c + '"' + (n === 'mic' ? micAct() : '') + ' style="grid-column:' + cx + ';grid-row:' + ry + '"><div class="cap">' + ic(n) + '</div></div>').join('') + '</div></div></div>';
}

// Drawing, placing and fitting: the bar sits just above the keyboard, centred; the open panel sits just above the bar.
function controls() {
  const g = (label, k, opts, cur) => '<div class="grp"><b>' + label + '</b><div class="opts">' + opts.map(([v, s]) => '<button class="o" data-k="' + k + '" data-v="' + v + '" aria-pressed="' + (String(v) === String(cur)) + '">' + s + '</button>').join('') + '</div></div>';
  return g('Bar', 'bar', CTL.bar, S.bar) + g('Cloud notes (part 2)', 'bar', CTL.notes, S.bar) + g('Voice panel', 'panel', CTL.panel, S.panel) +
    g('Cloud (part 2)', 'panel', CTL.cloud, S.panel) + g('Dictating with', 'engine', CTL.engine, S.engine) +
    g('Theme', 'theme', CTL.theme, S.theme) + g('Light or dark', 'mode', CTL.mode, S.mode) +
    g('Language', 'lang', CTL.lang, S.lang) + g('Keyboard size', 'size', CTL.size, S.size) + g('Motion', 'still', CTL.still, S.still ? 1 : 0) + g('View', 'view', CTL.view, S.view);
}
function render() {
  l = L[S.lang]; A = l.ar ? ' ar' : ''; D = l.ar ? ' dir="rtl" lang="ar"' : '';
  const de = document.documentElement;
  de.dataset.theme = S.theme; de.dataset.mode = S.mode; de.style.setProperty('--s', S.size); de.style.colorScheme = S.mode === 'dark' ? 'dark' : 'light';
  document.body.classList.toggle('still', S.still);
  $('ctl').innerHTML = controls();
  $('stage').innerHTML = (S.view === 'kb' || S.bar === 'preparing' ? app() + keyboard() : '') + bar() + panel();
  const marked = inUse().cloud && ['listening','transcribing'].includes(S.bar);
  $('say').innerHTML = SAY[S.bar].replace('{name}', '<bdi>' + l.namePrep + '</bdi>') + (marked ? SAY.cloudbar : '') +
    (S.panel === 'none' ? '' : SAY[S.panel] + (S.view === 'kb' ? SAY.zoom : '')) + (S.still ? SAY.still : '');
  seen.bar = S.bar; seen.panel = S.panel;
  place();
}
function place() {
  const st = $('stage'), u = S.size, gp = 10 * u, m = 14;
  const kb = st.querySelector('.kb'), vb = st.querySelector('.vbar'), vp = st.querySelector('.vp'), ap = st.querySelector('.app');
  const els = [kb, vb, vp].filter(Boolean);
  const W = kb ? Math.max(760, kb.offsetWidth + 40) : Math.max(320, ...els.map(e => e.offsetWidth)) + 8 * m;
  st.style.width = W + 'px';
  let y = kb ? m : 3 * m;
  els.forEach(e => { e.style.left = Math.round((W - e.offsetWidth) / 2) + 'px'; e.style.bottom = y + 'px'; y += e.offsetHeight + gp; });
  const H = Math.max(y - gp + (kb ? m : 3 * m), kb ? m + 240 * u + gp + kb.offsetHeight + m : 0);
  st.style.height = H + 'px';
  if (ap) ap.style.height = (H - m - kb.offsetHeight - m - gp) + 'px';
  const wv = st.querySelector('.wave');
  if (wv) { const cs = getComputedStyle(de()), w = +cs.getPropertyValue('--voice-wave-px-0') * u, g = +cs.getPropertyValue('--voice-wave-px-1') * u;
    wv.innerHTML = '<b></b>'.repeat(Math.max(4, Math.floor((wv.clientWidth + g) / (w + g)))); wave.h = []; }
  fit();
}
const de = () => document.documentElement;
function fit() {
  const st = $('stage'), fb = $('fit'), k = Math.min(fb.clientWidth / st.offsetWidth, fb.clientHeight / st.offsetHeight, S.view === 'kb' ? 1.25 : 1.6);
  st.style.transform = 'translate(' + Math.max(0, (fb.clientWidth - st.offsetWidth * k) / 2) + 'px,' + Math.max(0, (fb.clientHeight - st.offsetHeight * k) / 2) + 'px) scale(' + k + ')';
}

// Motion: the wave follows a sample voice (speech, then a pause), the timer counts, downloads and checks move on.
const wave = {h:[], meter:0, last:0};
function frame(now) {
  const s = (now - S.t0) / 1000, dt = Math.min(.1, (now - wave.last) / 1000 || 0); wave.last = now;
  const env = (s % 5) < 3.4 ? .3 + .7 * Math.abs(Math.sin(s * 2.3)) * Math.abs(Math.sin(s * .9 + 1)) : 0;
  const tm = document.querySelector('.tm');
  if (tm) tm.textContent = Math.floor(s / 60) + ':' + String(Math.floor(s) % 60).padStart(2, '0');
  const bars = document.querySelectorAll('.wave b'), u = S.size;
  bars.forEach((b, n) => { const goal = 3 + 21 * env * (.4 + .6 * Math.abs(Math.sin(n * .9 + s * 7))); wave.h[n] = (wave.h[n] || 3) + (goal - (wave.h[n] || 3)) * .22; b.style.height = (wave.h[n] * u).toFixed(1) + 'px'; });
  const mt = document.querySelector('.meter i');
  if (mt && now - wave.meter > 100) { mt.style.width = Math.round(env * 100) + '%'; wave.meter = now; }
  S.models.forEach((r, i) => {
    if (r.st !== 'dl' || r.paused || S.panel !== 'models') return;
    r.p = Math.min(1, r.p + dt / 120);
    if (r.p >= 1) { r.st = r.after || 'ready'; delete r.after; render(); return; }
    const bar = document.querySelector('[data-dl="' + i + '"]'), txt = document.querySelector('[data-dlt="' + i + '"]');
    if (bar) bar.style.width = (r.p * 100).toFixed(1) + '%';
    if (txt) txt.textContent = dlText(r);
  });
  if (S.check) {
    S.check.p = Math.min(1, S.check.p + dt / 5);
    const c = document.querySelector('[data-chk]');
    if (c) c.style.width = (S.check.p * 100).toFixed(1) + '%';
    if (S.check.p >= 1) { const id = S.check.id; S.check = null; if (id === 'amd') S.failed.amd = true; else { S.passed[id] = true; delete S.failed[id]; } render(); }
  }
  requestAnimationFrame(frame);
}

// Clicks: the controls on top, then the bar's and the panel's own buttons, which work as the product's will.
function transcribe() {
  clearTimeout(timer); S.bar = 'transcribing'; S.undone = false;
  timer = setTimeout(() => { S.bar = S.pw ? 'password' : 'text'; save(); render(); }, PACE.write);
}
function flash(k) { S[k] = true; setTimeout(() => { S[k] = false; render(); }, PACE.flash); }
function go(p) { S.panel = p; S.pg = 0; S.tick = []; S.ask = false; if (p === 'device' && !S.passed.nv && !S.check) S.check = {id:'nv', p:.3}; cloudGo(p); }
// Cancel, the gear or the Dictation key while preparing (owner, 2026-11-17): nothing was recorded, so nothing is sent.
function end() { clearTimeout(timer); S.bar = S.bar === 'preparing' ? 'none' : 'cancelled'; }
const ACT = {
  stop:transcribe, anyway:transcribe, again:transcribe, copy:() => flash('copied'), undo:() => { S.undone = true; },
  cancel:end, close:() => { clearTimeout(timer); S.bar = 'none'; },
  listen:() => { clearTimeout(timer); S.bar = 'preparing'; timer = setTimeout(() => { S.bar = 'listening'; S.t0 = performance.now(); save(); render(); }, PACE.prep); },
  mic:() => { if (S.bar === 'preparing') end(); else if (S.bar === 'listening') transcribe(); else ACT.listen(); },
  gear:() => { if (['preparing','listening','transcribing'].includes(S.bar)) end(); go(S.panel === 'none' ? 'home' : 'none'); },
  getmodel:() => { const r = S.models[0]; if (r.st !== 'dl') { r.after = r.st; r.st = 'dl'; r.p = 0; } go('models'); },
  back:() => go('home'), pclose:() => go('none'), go:p => go(p), pg:d => { S.pg += +d; }, noop:() => {},
  tick:x => { const i = +x, n = S.tick.indexOf(i); if (S.panel === 'history') { if (n < 0) S.tick.push(i); else S.tick.splice(n, 1); } else S.tick = n < 0 ? [i] : []; },
  use:() => { S.models.forEach(m => { if (m.st === 'use') m.st = 'ready'; }); ticked().st = 'use'; },
  dl:() => { Object.assign(ticked(), {st:'dl', p:0, paused:false}); },
  del:() => { const r = ticked(); if (r.st === 'other') S.models.splice(S.tick[0], 1); else r.st = 'none'; S.tick = []; },
  pause:x => { S.models[+x].paused = !S.models[+x].paused; }, dlcancel:x => { Object.assign(S.models[+x], {st:'none', p:0, paused:false}); },
  devuse:() => { S.devUse = DEV[S.tick[0]].id; }, devcheck:() => { S.check = {id:DEV[S.tick[0]].id, p:0}; },
  start:() => go('comparing'), ccancel:() => go('compare'), cagain:() => go('compare'),
  cuse:() => { const id = RESULT[S.lang].rows[S.tick[0]][0]; S.models.forEach(m => { if (m.st === 'use') m.st = 'ready'; if (m.id === id) m.st = 'use'; }); },
  hcopy:() => flash('hcopied'),
  hpin:() => { const all = S.tick.every(i => S.hist[i].pin); S.tick.forEach(i => { S.hist[i].pin = !all; }); S.tick = []; },
  hdel:() => { S.hist = S.hist.filter((h, i) => !S.tick.includes(i)); S.tick = []; },
  hins:() => { S.extra += S.tick.map(i => ' ' + S.hist[i].text).join(''); S.tick = []; },
  hall:() => { S.ask = true; }, hno:() => { S.ask = false; }, hyes:() => { S.hist = S.hist.filter(h => h.pin); S.ask = false; S.tick = []; S.pg = 0; },
  sw:k => { S.opt[k] = !S.opt[k]; }, pick:(k, i) => { S.opt[k] = +i; },
  st:(k, d) => {
    const [, kind, o] = OPTS.find(x => x[0] === k), v = S.opt[k];
    if (kind === 'ch') S.opt[k] = Math.max(0, Math.min(2, v + +d));
    else if (o.list) S.opt[k] = o.list[Math.max(0, Math.min(o.list.length - 1, o.list.indexOf(v) + +d))];
    else S.opt[k] = Math.max(o.min, Math.min(o.max, v + +d));
  }
};
document.addEventListener('click', e => {
  const o = e.target.closest('.o');
  if (o) {
    const k = o.dataset.k, v = o.dataset.v;
    S[k] = k === 'size' ? +v : k === 'still' ? v === '1' : v;
    if (k === 'lang') fresh();
    if (k === 'bar') { clearTimeout(timer); S.undone = false; S.t0 = performance.now(); if (v === 'password' || v === 'text') S.pw = v === 'password'; }
    if (k === 'panel') go(v);
    if (k === 'engine') useEngine(v);
    save(); render(); return;
  }
  const a = e.target.closest('[data-a]');
  if (!a || a.disabled) return;
  const [name, x, y] = a.dataset.a.split(':');
  if (ACT[name]) { ACT[name](x, y); save(); render(); }
});
document.addEventListener('keydown', e => { if ((e.key === 'Enter' || e.key === ' ') && e.target.matches('[data-a]:not(button)')) { e.preventDefault(); e.target.click(); } });
addEventListener('resize', fit);
// Starts once voice-cloud-data.js and voice-cloud.js have added part 2's texts, pages and acts.
function boot() {
  fresh();
  if (S.panel === 'device' || PRESET[S.panel]) go(S.panel);
  render();
  if (document.fonts && document.fonts.ready) document.fonts.ready.then(place);
  requestAnimationFrame(frame);
}
document.addEventListener('DOMContentLoaded', boot);

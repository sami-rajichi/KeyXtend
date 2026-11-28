// Snip mock-up: the controls, the state and the drawing of each step.
'use strict';

const S = {step:'bar', bar:'keys', editor:'top', kb:'bubble', theme:'native', mode:'light', lang:'en',
  kind:'photo', tool:'pen', ink:'red', speed:2, tab:0, fps:30, mic:false, snd:false, taken:false, done:null, kbUp:false, toast:null};
try { Object.assign(S, JSON.parse(localStorage.getItem('kx-snip') || '{}'), {done:null, toast:null, taken:false, kbUp:false}); } catch (e) {}
const save = () => { try { localStorage.setItem('kx-snip', JSON.stringify(S)); } catch (e) {} };
const $ = id => document.getElementById(id);
let timer = 0;

const ctl = (label, opts, cur, key) => '<div class="grp"><b>' + label + '</b><div class="opts">' +
  opts.map(([v, t]) => '<button class="o" aria-pressed="' + (String(v) === String(cur)) + '" data-k="' + key + '" data-v="' + v + '">' + t + '</button>').join('') + '</div></div>';

function controls() {
  $('bar').innerHTML = ctl('Step', STEPS, S.step, 'step') +
    ctl('Capture bar', Object.entries(BARS).map(([k, b]) => [k, b.label]), S.bar, 'bar') +
    ctl('Editor', Object.entries(EDITORS), S.editor, 'editor') +
    ctl('Keyboard', Object.entries(KBS), S.kb, 'kb') +
    ctl('Theme', Object.entries(T).map(([k, t]) => [k, t.name]), S.theme, 'theme') +
    ctl('Light or dark', [['light', 'Light'], ['dark', 'Dark']], S.mode, 'mode') +
    ctl('Language', Object.entries(L).map(([k, l]) => [k, l.name]), S.lang, 'lang');
}

function vars() {
  const t = T[S.theme], p = t[S.mode], r = document.documentElement.style, light = S.mode === 'light';
  const set = {
    '--plate':p.plate,'--key':p.key,'--key-mod':p.keyMod,'--key-act':p.keyAct,'--enter':p.enter,'--enter-ink':p.enterInk,'--danger':p.danger,'--danger-ink':p.dangerInk,
    '--legend':p.legend,'--legend-mod':p.legendMod,'--legend-2':p.legend2,'--legend-act':p.legendAct,'--ring':p.ring,'--badge-bg':p.badgeBg,'--badge-ink':p.badgeInk,
    '--pop-bg':p.popBg,'--pop-ink':p.popInk,'--pop-muted':p.popMuted,'--pop-line':p.popLine,'--pop-hover':p.popHover,'--pop-sel':p.popSel,'--chip-bg':p.chipBg,
    '--key-shadow':p.keyShadow,'--win-shadow':p.winShadow,'--key-sk':p.keySk || p.key,'--mod-sk':p.modSk || p.keyMod,'--enter-sk':p.enterSk || p.enter,'--danger-sk':p.dangerSk || p.danger,
    '--r-key':t.rKey + 'px','--r-win':t.rWin + 'px','--g':t.g,'--fs':t.fs + 'px','--fs-mod':t.fsMod + 'px','--fw':t.fw,'--ar':t.ar,'--f-latin':t.latin,'--f-arabic':t.arabic,
    '--rec':REC,'--dim':DIM,'--edge':EDGE,'--tag':TAG,'--ink':(INK.find(x => x[0] === S.ink) || INK[0])[1],
    '--wall':light ? '#C6CFD6' : '#191C20','--a-bg':light ? '#FFFFFF' : '#202124','--a-ink':light ? '#1B1B1B' : '#E6E6E6','--a-top':light ? '#F3F3F3' : '#2B2C2F',
    '--a-mute':light ? '#5F5F5F' : '#A9AEB5','--a-line':light ? '#00000014' : '#FFFFFF14','--a-bar':light ? '#0078D4' : '#4CA0E0','--task':light ? '#EEF0F2E6' : '#1F2125E6'
  };
  for (const k in set) r.setProperty(k, set[k]);
  r.setProperty('color-scheme', light ? 'light' : 'dark');
  document.body.style.fontFamily = t.latin;
}

// What sits above the frozen or live screen: the keyboard, its bubble, or nothing.
function ours(l) {
  if (S.kb === 'shown' || S.kbUp) return keyboard(l, 0.8, true);
  return S.kb === 'bubble' ? bubble() : '';
}

function scene(l) {
  const full = {x:0, y:0, w:W, h:H}, kbShown = S.kb === 'shown';
  const bar = on => captureBar(l, S.bar, on, S.kind, kbShown);
  switch (S.step) {
    case 'bar': return dimAll() + ours(l) + bar('region') + hint(l.hint.region, S.bar, kbShown);
    case 'region': return hole(BOX.region) + ours(l) + photoStrip(l, BOX.region, S.done);
    case 'window': return S.taken ? hole(BOX.notes) + ours(l) + photoStrip(l, BOX.notes, S.done)
      : glow(BOX.notes) + ours(l) + bar('window') + hint(l.hint.window, S.bar, kbShown);
    case 'full': return hole(full) + ours(l) + photoStrip(l, null, S.done);
    case 'vready': return hole(BOX.video) + ours(l) + readyStrip(l, BOX.video, S.mic, S.snd);
    case 'count': return ours(l) + countdown(l, BOX.video, 3);
    case 'rec': return ours(l) + recording(l, BOX.video, '00:12', S.mic, S.snd);
    case 'vdone': return '<div class="soon" style="' + at(BOX.video) + '"></div>' + ours(l) +
      strip(l, [['len', '', '1:24'], ['copy', 'copy', l.strip.copy], ['edit', 'square-pen', l.strip.edit], ['save', 'save', l.strip.save], ['close', 'x', l.strip.close]], BOX.video, S.done);
    case 'edit': return keyboard(l, 0.75, false) + editor(l, S, false) + toast(l);
    case 'vedit': return keyboard(l, 0.75, false) + editor(l, S, true) + toast(l);
    case 'ask': return keyboard(l, 0.75, false) + editor(l, S, false) + ask(l);
    default: return keyboard(l, 0.8, false) + panel(l, S);
  }
}

const toast = l => S.toast ? '<div class="toast" style="left:' + px(ED.x + ED.w / 2) + ';top:' + px(ED.y + ED.h - 56) + '">' + ic('check') + word(l, l.done[S.toast]) + '</div>' : '';

function render() {
  vars(); controls();
  const l = L[S.lang], st = $('stage');
  st.dataset.style = S.theme;
  st.style.width = W + 'px'; st.style.height = H + 'px';
  st.innerHTML = desktop(l) + scene(l);
  const kbNote = ['edit', 'vedit', 'ask', 'panel'].includes(S.step) ? '' : ' ' + KB_SAY[S.kb];
  const barNote = ['bar', 'window'].includes(S.step) ? ' ' + BARS[S.bar].say : '';
  $('say').innerHTML = SAY[S.step] + barNote + kbNote;
  fit();
}

function fit() {
  const st = $('stage'), f = $('fit');
  const k = Math.min(f.clientWidth / W, f.clientHeight / H);
  st.style.transform = 'translate(' + Math.max(0, (f.clientWidth - W * k) / 2) + 'px,0) scale(' + k + ')';
}

function go(step) { S.step = step; S.done = null; S.toast = null; S.taken = false; clearTimeout(timer); save(); render(); }
function later(ms, fn) { clearTimeout(timer); timer = setTimeout(fn, ms); }

// The strip's keycaps: Copy and Save tick and end the snip, Edit opens the editor, Record counts down.
function onStrip(id) {
  if (id === 'copy' || id === 'save') { S.done = id; render(); later(1100, () => go('bar')); return; }
  if (id === 'edit') return go(S.step === 'vdone' ? 'vedit' : 'edit');
  if (id === 'close') return go('ask');
  if (id === 'record') { go('count'); later(1400, () => go('rec')); return; }
  if (id === 'mic') S.mic = !S.mic;
  if (id === 'sound') S.snd = !S.snd;
  render();
}

function onWay(id) {
  if (id === 'photo' || id === 'video') { S.kind = id; return render(); }
  if (id === 'settings') return go('panel');
  if (id === 'close') return go('bar');
  if (id === 'window') return go('window');
  if (id === 'full') return go(S.kind === 'video' ? 'vready' : 'full');
  if (id === 'vregion' || id === 'vfull' || (id === 'region' && S.kind === 'video')) return go('vready');
  return go('region');
}

function onEditor(id) {
  if (TOOLS.some(t => t[0] === id)) { S.tool = id; return render(); }
  if (id === 'copy' || id === 'save' || id === 'asksave') { S.toast = id === 'asksave' ? 'save' : id; render(); later(1200, () => { S.toast = null; render(); }); return; }
  if (id === 'close') return go('ask');
  if (id === 'drop') return go('bar');
  if (id === 'back') return go('edit');
  if (id === 'pclose') return go('bar');
}

document.addEventListener('click', e => {
  const o = e.target.closest('.o');
  if (o) { const k = o.dataset.k; if (k === 'step') return go(o.dataset.v); S[k] = o.dataset.v; save(); return render(); }
  const b = e.target.closest('button,[data-strip],.hover,.dim,.tb');
  if (!b) return;
  const d = b.dataset;
  if (d.strip) return onStrip(d.strip);
  if (d.way) return onWay(d.way);
  if (d.rec) { if (d.rec === 'stop') return go('vdone'); if (d.rec === 'cancel') return go('bar'); S[d.rec === 'mic' ? 'mic' : 'snd'] = !S[d.rec === 'mic' ? 'mic' : 'snd']; return render(); }
  if (d.ed) return onEditor(d.ed);
  if (d.ink) { S.ink = d.ink; return render(); }
  if (d.speed) { S.speed = Number(d.speed); return render(); }
  if (d.tab) { S.tab = Number(d.tab); return render(); }
  if (d.kb) { S.kb = d.kb; save(); return render(); }
  if (d.fps) { S.fps = Number(d.fps); return render(); }
  if (d.micd || d.sndd) { S[d.micd ? 'mic' : 'snd'] = (d.micd || d.sndd) === '1'; return render(); }
  if (b.classList.contains('bubble')) { S.kbUp = true; return render(); }
  if (b.classList.contains('tb') && S.kbUp) { S.kbUp = false; return render(); }
  if (b.classList.contains('hover')) { S.taken = true; return render(); }
  if (b.classList.contains('dim') && S.step === 'bar') return go(S.kind === 'video' ? 'vready' : 'region');
});
addEventListener('resize', fit);
if (document.fonts && document.fonts.ready) document.fonts.ready.then(render);
render();

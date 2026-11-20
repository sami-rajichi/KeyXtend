// Voice typing mock-up (design/voice-bar.html): icons, shared parts, the bar in each state and the Voice panel's pages,
// drawn from voice-bar-data.js, which loads first. voice-bar-scene.js places them; part 2's cloud files load last.
'use strict';

// Lucide icons (ISC), copied from assets/icons/lucide; win-logo is our own drawing (selection-chips.html).
const ICONS = {
  'mic':'<path d="M12 19v3"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><rect x="9" y="2" width="6" height="13" rx="3"/>',
  'mic-off':'<path d="M12 19v3"/><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33"/><path d="M16.95 16.95A7 7 0 0 1 5 12v-2"/><path d="M18.89 13.23A7 7 0 0 0 19 12v-2"/><path d="m2 2 20 20"/><path d="M9 9v3a3 3 0 0 0 5.12 2.12"/>',
  'audio-lines':'<path d="M2 10v3"/><path d="M6 6v11"/><path d="M10 3v18"/><path d="M14 8v7"/><path d="M18 5v13"/><path d="M22 10v3"/>',
  'captions':'<rect width="18" height="14" x="3" y="5" rx="2" ry="2"/><path d="M7 15h4M15 15h2M7 11h2M13 11h4"/>',
  'copy':'<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
  'undo-2':'<path d="M9 14 4 9l5-5"/><path d="M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11"/>',
  'x':'<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
  'settings-2':'<path d="M14 17H5"/><path d="M19 7h-9"/><circle cx="17" cy="17" r="3"/><circle cx="7" cy="7" r="3"/>',
  'square':'<rect width="18" height="18" x="3" y="3" rx="2"/>',
  'history':'<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>',
  'trash-2':'<path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
  'pin':'<path d="M12 17v5"/><path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/>',
  'pin-off':'<path d="M12 17v5"/><path d="M15 9.34V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H7.89"/><path d="m2 2 20 20"/><path d="M9 9v1.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h11"/>',
  'check':'<path d="M20 6 9 17l-5-5"/>',
  'folder':'<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  'folder-open':'<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/>',
  'download':'<path d="M12 15V3"/><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="m7 10 5 5 5-5"/>',
  'pause':'<rect x="14" y="3" width="5" height="18" rx="1"/><rect x="5" y="3" width="5" height="18" rx="1"/>',
  'play':'<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z"/>',
  'cpu':'<path d="M12 20v2"/><path d="M12 2v2"/><path d="M17 20v2"/><path d="M17 2v2"/><path d="M2 12h2"/><path d="M2 17h2"/><path d="M2 7h2"/><path d="M20 12h2"/><path d="M20 17h2"/><path d="M20 7h2"/><path d="M7 20v2"/><path d="M7 2v2"/><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="8" y="8" width="8" height="8" rx="1"/>',
  'gpu':'<path d="M2 17h18a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2H2"/><path d="M2 21V3"/><path d="M7 17v3a1 1 0 0 0 1 1h5a1 1 0 0 0 1-1v-3"/><circle cx="16" cy="11" r="2"/><circle cx="8" cy="11" r="2"/>',
  'rotate-ccw':'<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/>',
  'scale':'<path d="M12 3v18"/><path d="m19 8 3 8a5 5 0 0 1-6 0zV7"/><path d="M3 7h1a17 17 0 0 0 8-2 17 17 0 0 0 8 2h1"/><path d="m5 8 3 8a5 5 0 0 1-6 0zV7"/><path d="M7 21h10"/>',
  'activity':'<path d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2"/>',
  'arrow-left':'<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>',
  'arrow-left-right':'<path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/>',
  'chevron-up':'<path d="m18 15-6-6-6 6"/>', 'chevron-down':'<path d="m6 9 6 6 6-6"/>', 'chevron-left':'<path d="m15 18-6-6 6-6"/>', 'chevron-right':'<path d="m9 18 6-6-6-6"/>',
  'plus':'<path d="M5 12h14"/><path d="M12 5v14"/>', 'minus':'<path d="M5 12h14"/>',
  'lock':'<rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
  'hard-drive':'<path d="M10 16h.01"/><path d="M2.212 11.577a2 2 0 0 0-.212.896V18a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-5.527a2 2 0 0 0-.212-.896L18.55 5.11A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/><path d="M21.946 12.013H2.054"/><path d="M6 16h.01"/>',
  'file':'<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z"/><path d="M14 2v5a1 1 0 0 0 1 1h5"/>',
  'clipboard-paste':'<path d="M11 14h10"/><path d="M16 4h2a2 2 0 0 1 2 2v1.344"/><path d="m17 18 4-4-4-4"/><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 1.793-1.113"/><rect x="8" y="2" width="8" height="4" rx="1"/>',
  'delete':'<path d="M10 5a2 2 0 0 0-1.344.519l-6.328 5.74a1 1 0 0 0 0 1.481l6.328 5.741A2 2 0 0 0 10 19h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2z"/><path d="m12 9 6 6"/><path d="m18 9-6 6"/>',
  'arrow-right-to-line':'<path d="M17 12H3"/><path d="m11 18 6-6-6-6"/><path d="M21 5v14"/>',
  'corner-down-left':'<path d="M20 4v7a4 4 0 0 1-4 4H4"/><path d="m9 10-5 5 5 5"/>',
  'arrow-big-up':'<path d="M9 19a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1v-6a1 1 0 0 1 1-1h3.293a.707.707 0 0 0 .5-1.207l-7.086-7.086a1 1 0 0 0-1.414 0l-7.086 7.086a.707.707 0 0 0 .5 1.207H8a1 1 0 0 1 1 1z"/>',
  'square-menu':'<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M7 8h10"/><path d="M7 12h10"/><path d="M7 16h10"/>',
  'command':'<path d="M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3"/>',
  'hand-grab':'<path d="M18 11.5V9a2 2 0 0 0-2-2a2 2 0 0 0-2 2v1.4"/><path d="M14 10V8a2 2 0 0 0-2-2a2 2 0 0 0-2 2v2"/><path d="M10 9.9V9a2 2 0 0 0-2-2a2 2 0 0 0-2 2v5"/><path d="M6 14a2 2 0 0 0-2-2a2 2 0 0 0-2 2"/><path d="M18 11a2 2 0 1 1 4 0v3a8 8 0 0 1-8 8h-4a8 8 0 0 1-8-8 2 2 0 1 1 4 0"/>',
  'mouse-right':'<path d="M12 7.318V10"/><path d="M19 10v5a7 7 0 0 1-14 0V9c0-3.527 2.608-6.515 6-7"/><circle cx="17" cy="4" r="2"/>',
  'clipboard-clock':'<path d="M16 14v2.2l1.6 1"/><path d="M16 4h2a2 2 0 0 1 2 2v.832"/><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h2"/><circle cx="16" cy="16" r="6"/><rect x="8" y="2" width="8" height="4" rx="1"/>',
  'key-round':'<path d="M2.586 17.414A2 2 0 0 0 2 18.828V21a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h1a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h.172a2 2 0 0 0 1.414-.586l.814-.814a6.5 6.5 0 1 0-4-4z"/><circle cx="16.5" cy="7.5" r=".5" fill="currentColor"/>',
  'power':'<path d="M12 2v10"/><path d="M18.4 6.6a9 9 0 1 1-12.77.04"/>',
  'win-logo':'<path fill="currentColor" d="M3.5 3.5h8v8h-8zM12.5 3.5h8v8h-8zM3.5 12.5h8v8h-8zM12.5 12.5h8v8h-8z"/>'
};
const ic = n => '<svg class="i" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' + (ICONS[n] || '') + '</svg>';

// Formats in the language shown, and a text with its {names} filled.
const f = () => FM[S.lang];
const fileSize = mb => mb >= 1000 ? f().gb(nf(+(mb / 1000).toFixed(2))) : f().mb(mb);
const binSize = mb => mb >= 1024 ? f().gb(nf(mb / 1024)) : f().mb(mb);
const gb1 = x => f().gb(nf(x, 1));
const secs = x => f().s(nf(x, 1));
const t = (s, v = {}) => s.replace(/\{(\w+)\}/g, (_, k) => v[k] === undefined ? '' : v[k]);

const $ = id => document.getElementById(id);
const seen = {bar:'', panel:''};
let l, A, D, timer = 0;

// Shared parts: buttons, marks, ticks, rows, list with page buttons.
const btn = (act, icon, label, tone = '', dis = false, name = '') => '<button class="btn ' + tone + (label ? A : '') + '" data-a="' + act + '"' + (dis ? ' disabled' : '') + (name ? ' aria-label="' + name + '"' : '') + '>' + (icon ? ic(icon) : '') + (label ? '<span' + D + '>' + label + '</span>' : '') + '</button>';
const mark = (label, icon, cls = '') => '<span class="mk ' + cls + A + '">' + (icon ? ic(icon) : '') + '<span' + D + '>' + label + '</span></span>';
const tickBox = (i, one) => { const n = S.tick.indexOf(i); return '<span class="ck">' + (n < 0 ? '' : one ? ic('check') : n + 1) + '</span>'; };
const cell = (name, sub) => '<div class="tx' + A + '"' + D + '><div>' + name + '</div>' + (sub === undefined ? '' : '<div class="m">' + sub + '</div>') + '</div>';
const tile = n => '<span class="tile">' + ic(n) + '</span>';
const CK = 'calc(var(--clip-tick-px-0) * var(--u))', TL = 'calc(var(--panel-tile-px-0) * var(--u))', COL = 'calc(var(--voice-col-px) * var(--u))';
const row = (cols, inner, act, cls = '', extra = '') => '<div class="pr ' + cls + '" style="grid-template-columns:' + cols.join(' ') + '"' + (act ? ' data-a="' + act + '" role="button" tabindex="0"' : '') + extra + '>' + inner + '</div>';
const tickRow = (cols, inner, i) => row(cols, inner, 'tick:' + i, S.tick.includes(i) ? 'sel' : '', ' aria-pressed="' + S.tick.includes(i) + '"');
const note = (icon, html) => '<div class="pn' + A + '">' + ic(icon) + '<span class="x"' + D + '>' + html + '</span></div>';
const gap = '<span style="flex:1"></span>';
// Filter chips (PanelChip.qml): a radio group of acts, the shown one filled and checked.
const chips = (name, items, on, cls = '', desc = '') => '<div class="flt' + cls + '" role="radiogroup" aria-label="' + name + '"' + (desc ? ' aria-describedby="' + desc + '"' : '') + '>' + items.map(([act, label], i) =>
  '<button class="fl' + A + '" role="radio" aria-checked="' + (i === on) + '" data-a="' + act + '">' + (i === on ? ic('check') : '') + '<span' + D + '>' + label + '</span></button>').join('') + '</div>';
function list(rows, head = '') {
  const pages = Math.max(1, Math.ceil(rows.length / 5));
  S.pg = Math.max(0, Math.min(S.pg, pages - 1));
  const shown = rows.slice(S.pg * 5, S.pg * 5 + 5);
  while (shown.length < 5) shown.push('<div class="pr pad"></div>');
  return '<div class="pb"><div class="pl' + (head ? ' th' : '') + '">' + head + shown.join('') + '</div><div class="pager">' +
    '<button class="pg" data-a="pg:-1" aria-label="' + l.pageBack + '"' + (S.pg <= 0 ? ' disabled' : '') + '>' + ic('chevron-up') + '</button>' +
    '<div class="pgn' + A + '">' + t(l.of, {a:S.pg + 1, b:pages}) + '</div>' +
    '<button class="pg" data-a="pg:1" aria-label="' + l.pageOn + '"' + (S.pg >= pages - 1 ? ' disabled' : '') + '>' + ic('chevron-down') + '</button></div></div>';
}

// The bar in each state; Cancel stays at the left end and Stop or Close at the right end in every language.
function bar() {
  // Preparing (owner, 2026-11-17): no bar until the microphone is open; only the Dictation key glows amber.
  if (S.bar === 'none' || S.bar === 'preparing') return '';
  const vb = (act, icon, name, cls = '', dis = false) => '<button class="vb ' + cls + '" data-a="' + act + '" aria-label="' + name + '"' + (dis ? ' disabled' : '') + '>' + ic(icon) + '</button>';
  const vt = (act, icon, label, cls = '', dis = false) => '<button class="vt ' + cls + A + '" data-a="' + act + '"' + (dis ? ' disabled' : '') + '>' + ic(icon) + '<span' + D + '>' + label + '</span></button>';
  const slot = n => '<span class="slot" aria-hidden="true">' + ic(n) + '</span>';
  const msg = s => '<span class="msg' + A + '"' + D + '>' + s + '</span>';
  const gear = vb('gear', 'settings-2', l.gear), close = vb('close', 'x', l.close), cancel = vb('cancel', 'x', l.cancel);
  let inner = '', wide = false;
  const own = '<span class="lc' + A + '"' + D + '>' + l.own + '</span>';
  if (S.bar === 'listening') inner = cancel + '<span class="st' + A + '"><span class="dot"></span><span' + D + '>' + l.listening + '</span></span>' + own + cloudChip('listen') +
    (S.still ? '<span class="meter"><i></i></span>' : '<span class="wave"></span>') + '<span class="tm">0:00</span>' + gear + vb('stop', 'square', l.stop, 'stop');
  if (S.bar === 'transcribing') inner = cancel + '<span class="st' + A + '">' + ic('audio-lines') + '<span' + D + '>' + l.transcribing + '</span></span>' + cloudChip('send') +
    '<span class="shim"></span>' + gear + vb('stop', 'square', l.stop, 'stop', true);
  if (S.bar === 'text') { wide = true; inner = slot('captions') + '<div class="vtx' + A + '"' + D + '><div class="w">' + l.said + '</div></div>' +
    vt('copy', S.copied ? 'check' : 'copy', S.copied ? l.copied : l.copy) + vt('undo', 'undo-2', l.undo, '', S.undone) + gear + close; }
  if (S.bar === 'password') { wide = true; inner = slot('lock') + '<div class="vtx' + A + '"><div class="w mask">••••••••</div><small' + D + '>' + l.pwLine + '</small></div>' +
    vt('undo', 'undo-2', l.undo, '', S.undone) + gear + close; }
  if (S.bar === 'cancelled') inner = slot('x') + msg(l.cancelled) + vt('anyway', 'rotate-ccw', l.anyway, 'enter') + gear + close;
  if (S.bar === 'nothing') inner = slot('mic-off') + msg(l.nothing) + vt('listen', 'mic', l.listenAgain, 'enter') + gear + close;
  if (S.bar === 'nomodel') inner = slot('hard-drive') + msg(l.noModel) + vt('getmodel', 'download', l.download + ' · ' + fileSize(M[S.models[0].id][1]), 'enter') + gear + close;
  return '<div class="vbar' + (wide ? ' wide' : '') + (S.bar !== seen.bar ? ' in' : '') + '" role="toolbar" aria-label="' + l.title + '">' + inner + '</div>';
}

// The Voice panel's pages: each gives its header, the lines above and below its list, and its footer.
const inUse = () => S.models.find(r => r.st === 'use') || S.models[0];
const ticked = () => S.tick.length ? S.models[S.tick[0]] : null;
const PAGE = {
  home() {
    const changed = Object.keys(OPT0).filter(k => S.opt[k] !== OPT0[k]).length;
    const dev = S.devUse === 'auto' ? l.homeDevice : S.devUse === 'cpu' ? l.cpu : DEV.find(d => d.id === S.devUse).name;
    const rows = [['models','audio-lines',M[inUse().id][0]], ['cloud','cloud',cloudHome()], ['device','cpu',dev], ['compare','scale',l.homeCompare], ['history','history',f().texts(S.hist.length)],
      ['options','settings-2',changed ? t(l.changed, {n:changed}) : l.homeOptions]].map(([p, icon, v]) => row([TL, '1fr'], tile(icon) + cell(l[p], v), 'go:' + p));
    return {icon:'mic', title:l.title, sub:l.langLine, body:list(rows),
      foot:'<span class="x' + A + '"' + D + '>' + l.againHint + '</span>' + btn('again', 'rotate-ccw', l.again, 'enter', !['text','password'].includes(S.bar))};
  },
  models() {
    // The Local side (owner, 2026-11-17). "Models, slow PC" draws the default as Not for this PC, and the next ready local model in use in its place.
    const weak = S.weak, def = S.models[0], spare = weak && def.st === 'use' ? S.models.find(m => !m.cloud && m.st === 'ready') : null;
    const look = r => weak && r === def ? 'unfit' : r === spare ? 'use' : r.st;
    const rows = S.models.map((r, i) => {
      if (r.cloud) return '';
      const [name, mb] = M[r.id], st = look(r);
      let sub = fileSize(mb), right = '';
      if (st === 'use') right = mark(l.inUse, 'check', 'lit') + (r.def ? mark(l.def) : '');
      if (st === 'ready') right = (r.def ? mark(l.def) : '') + mark(l.ready, 'check');
      if (st === 'none') right = mark(l.notDl, 'download', 'out');
      if (st === 'other') right = mark(l.other, 'file', 'out');
      if (st === 'unfit') { right = mark(l.notHere, 'x', 'out'); sub = t(l.unfitWhy, {need:gb1(3.2), free:gb1(2.8)}); }
      if (r.st === 'dl') {
        sub = '<span data-dlt="' + i + '">' + dlText(r) + '</span><div class="prog"><i data-dl="' + i + '" style="width:' + (r.p * 100).toFixed(1) + '%"></i></div>';
        right = '<div class="acts">' + btn('pause:' + i, r.paused ? 'play' : 'pause', r.paused ? l.resume : l.pause) + btn('dlcancel:' + i, 'x', l.cancel) + '</div>';
      }
      return tickRow([CK, '1fr', 'auto'], tickBox(i, true) + cell(name, sub) + '<div class="marks">' + right + '</div>', i);
    }).filter(Boolean);
    const r = ticked();
    return {icon:'audio-lines', title:l.models, sub:l.forLang, top:sides('local') + note('check', l.best + ' <b>' + M[S.models.find(m => m.def).id][0] + '</b>') + (weak ? slowBand() : ''), body:list(rows),
      room:'<div class="room' + A + '">' + ic('hard-drive') + '<span class="x"' + D + '>' + t(l.folderLine, {path:'<bdi dir="ltr">D:\\KeyXtend\\models</bdi>', free:fileSize(212000)}) + '</span>' +
        btn('noop', 'folder', l.change) + btn('noop', 'folder-open', l.open) + '</div>',
      foot:btn('del', 'trash-2', l.delFile, 'danger', !r || !['ready','other'].includes(look(r))) + btn('dl', 'download', l.download, '', !r || look(r) !== 'none') + gap +
        btn('use', 'check', l.use, 'enter', !r || look(r) !== 'ready')};
  },
  device() {
    const rows = DEV.map((d, i) => {
      const name = d.id === 'auto' ? l.auto : d.id === 'cpu' ? l.cpu : d.name;
      const sub = d.id === 'auto' ? l.autoSub : d.id === 'cpu' ? t(l.cpuSub, {n:d.cores, mem:f().gb(d.mem), free:gb1(d.free)}) : t(d.built ? l.builtSub : l.cardSub, {mem:f().gb(d.mem), free:gb1(d.free)});
      let right = '';
      if (S.check && S.check.id === d.id) right = '<div class="chk' + A + '"><span' + D + '>' + l.checking + '</span><div class="prog"><i data-chk style="width:' + (S.check.p * 100).toFixed(1) + '%"></i></div></div>';
      else if (S.devUse === d.id) right = mark(l.inUse, 'check', 'lit');
      else if (S.passed[d.id]) right = mark(t(l.passed, {x:f().times(d.gain)}), 'check');
      else if (S.failed[d.id]) right = mark(l.failed, 'x', 'out');
      return tickRow([CK, TL, '1fr', 'auto'], tickBox(i, true) + tile(d.icon) + cell(name, sub) + '<div class="marks">' + right + '</div>', i);
    });
    const amd = DEV[3], d = S.tick.length ? DEV[S.tick[0]] : null;
    return {icon:'cpu', title:l.device, sub:l.devHint, body:list(rows),
      top:S.failed.amd ? note('x', '<bdi>' + amd.name + '</bdi>: ' + l.failed + '. ' + t(l.failWhy, {a:f().words(amd.diff), b:f().words(S.opt.cardDiff)})) : '',
      foot:btn('devcheck', 'activity', l.check, '', !d || !d.name || !!S.check) + gap + btn('devuse', 'check', l.devUse, 'enter', !d || d.id === S.devUse)};
  },
  compare() {
    const n = S.models.filter(r => !r.cloud && (r.st === 'use' || r.st === 'ready')).length;
    return {icon:'scale', title:l.compare, sub:l.forLang,
      body:'<div class="pb"><div class="read' + A + '"><p class="m"' + D + '>' + l.intro + '</p><div class="big' + A + '"' + D + '>' + l.read + '</div><p class="m"' + D + '>' + f().ready(n) + ' · ' + l.homeDevice + '</p></div></div>',
      foot:gap + btn('start', 'mic', l.start, 'enter')};
  },
  comparing() {
    const ready = S.models.filter(r => !r.cloud && (r.st === 'use' || r.st === 'ready')), cur = 2;
    const wait = id => (RESULT[S.lang].rows.find(r => r[0] === id) || [0, 0, 1.5])[2];
    const rows = ready.map((r, i) => row(['1fr', 'auto'], cell(M[r.id][0]) + '<div class="marks">' + (i < cur ? mark(t(l.doneIn, {t:secs(wait(r.id))}), 'check') :
      i === cur ? '<div class="chk' + A + '"><span' + D + '>' + l.writing + '</span><div class="prog run"><i></i></div></div>' : mark(l.waiting, '', 'out')) + '</div>', '', 'flat'));
    return {icon:'scale', title:l.compare, sub:l.forLang, top:note('scale', t(l.step, {a:cur + 1, b:ready.length}) + ' · <b>' + M[ready[cur].id][0] + '</b>'), body:list(rows),
      foot:gap + btn('ccancel', 'x', l.cancel)};
  },
  results() {
    const R = RESULT[S.lang], cols = [CK, '1fr', COL, COL, COL];
    const head = row(cols, '<span></span><span' + D + '>' + l.cols[0] + '</span>' + l.cols.slice(1).map(c => '<span class="num"' + D + '>' + c + '</span>').join(''), '', 'head' + A);
    const rows = R.rows.map(([id, wr, w, mem], i) => tickRow(cols, tickBox(i, true) + cell(M[id][0]) + '<span class="num"' + D + '>' + t(l.wrongOf, {a:wr, b:R.total}) + '</span>' +
      '<span class="num"' + D + '>' + secs(w) + '</span><span class="num"' + D + '>' + gb1(mem) + '</span>', i));
    return {icon:'scale', title:l.compare, sub:l.resultLine, top:note('check', l.kept), body:list(rows, head),
      foot:btn('cagain', 'rotate-ccw', l.againBtn) + gap + btn('cuse', 'check', l.use, 'enter', !S.tick.length)};
  },
  history() {
    const rows = S.hist.map((h, i) => tickRow([CK, '1fr', 'auto'], tickBox(i) + '<div class="tx"><div dir="' + (h.lang === 'ar' ? 'rtl' : 'ltr') + '" lang="' + h.lang + '">' + h.text + '</div>' +
      '<div class="m' + A + '"' + D + '>' + l.days[h.day] + ' ' + h.time + ' · ' + l.langs[h.lang] + ' · ' + M[h.model][0] + '</div></div>' + (h.pin ? '<span class="pin">' + ic('pin') + '</span>' : '<span></span>'), i));
    const n = S.tick.length, all = n && S.tick.every(i => S.hist[i].pin), loose = S.hist.filter(h => !h.pin).length;
    const foot = S.ask ? '<span class="x ask' + A + '"' + D + '>' + t(l.ask, {n:f().texts(loose)}) + '</span>' + btn('hyes', '', l.yes, 'danger') + btn('hno', '', l.no)
      : btn('hcopy', S.hcopied ? 'check' : 'copy', S.hcopied ? l.copied : l.copy, '', !n) + btn('hpin', all ? 'pin-off' : 'pin', all ? l.unpin : l.pin, '', !n) +
        btn('hdel', 'trash-2', l.del, 'danger', !n) + gap + btn('hins', 'clipboard-paste', n > 1 ? t(l.insert, {n}) : l.insertOne, 'enter', !n);
    return {icon:'history', title:l.history, sub:t(l.histSub, {n:f().texts(S.hist.length)}), extra:btn('hall', 'trash-2', l.deleteAll, '', !loose || S.ask), body:list(rows), foot};
  },
  options() {
    const rows = OPTS.flatMap(([k, kind, o]) => {
      const v = S.opt[k], label = cell(l.opt[k]);
      if (kind === 'sw') return row(['1fr', 'auto', 'auto'], label + '<span class="onoff' + A + '"' + D + '>' + (v ? l.on : l.off) + '</span><span class="sw' + (v ? ' on' : '') + '"></span>',
        'sw:' + k, '', ' role="switch" aria-checked="' + v + '"');
      // Insert the text (owner, 2026-11-17): Type it or Paste at once; while Paste is chosen, its hint takes the next row.
      if (kind === 'two') return [row(['1fr', 'auto'], label + chips(l.opt[k], l[o.texts].map((s, i) => ['pick:' + k + ':' + i, s]), v, ' in', v ? 'hint-' + k : ''), '', 'flat')]
        .concat(v ? [row(['auto', '1fr'], ic('clipboard-paste') + '<div class="tx' + A + '"' + D + '><div class="m" id="hint-' + k + '">' + l[o.hint] + '</div></div>', '', 'flat hint')] : []);
      const off = o && o.dep && !S.opt[o.dep], ends = kind === 'ch' ? [v <= 0, v >= 2] : o.list ? [v === o.list[0], v === o.list[o.list.length - 1]] : [v <= o.min, v >= o.max];
      const value = kind === 'ch' ? l.places[v] : fmtOpt(o.fmt, v), icons = kind === 'ch' ? ['chevron-left', 'chevron-right'] : ['minus', 'plus'];
      return row(['1fr', 'auto'], label + '<span class="ctl' + (off ? ' off' : '') + '">' + btn('st:' + k + ':-1', icons[0], '', 'sq', off || ends[0], l.less + ': ' + l.opt[k]) +
        '<span class="v' + (kind === 'ch' ? ' wide' : '') + A + '"' + D + '>' + value + '</span>' + btn('st:' + k + ':1', icons[1], '', 'sq', off || ends[1], l.more + ': ' + l.opt[k]) + '</span>', '', 'flat');
    });
    return {icon:'settings-2', title:l.options, sub:l.forAll, body:list(rows), foot:'<span class="x' + A + '"' + D + '>' + l.applies + '</span>'};
  }
};
const dlText = r => t(r.paused ? l.paused : l.got, {a:fileSize(Math.round(M[r.id][1] * r.p)), b:fileSize(M[r.id][1])});
function fmtOpt(fmt, v) {
  const F = f();
  if (fmt === 'bin') return binSize(v);
  if (fmt === 'dur') return v < 60 ? F.sec(v) : F.min(v / 60);
  if (fmt === 'threads') return v === 0 ? t(l.autoN, {n:8}) : String(v);
  return F[fmt](v);
}
function panel() {
  if (S.panel === 'none') return '';
  const P = PAGE[S.panel]();
  const back = S.panel === 'home' ? '' : '<button class="ib" data-a="back" aria-label="' + l.back + '">' + ic('arrow-left') + '</button>';
  return '<div class="vp' + (S.panel !== seen.panel ? ' in' : '') + '" role="dialog" aria-label="' + l.title + '"><div class="ph">' + back + ic(P.icon) +
    '<div class="tt' + A + '"><b' + D + '>' + P.title + '</b><span' + D + '>' + P.sub + '</span></div>' + (P.extra || '') +
    '<button class="ib" data-a="pclose" aria-label="' + l.close + '">' + ic('x') + '</button></div>' + (P.top || '') + P.body + (P.room || '') + '<div class="pf">' + P.foot + '</div></div>';
}

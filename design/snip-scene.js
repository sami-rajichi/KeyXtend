// Snip mock-up scene: the desktop, the keyboard and its bubble, the frozen screen, the capture bars, the strip and recording.
'use strict';

const px = v => Math.round(v) + 'px';
const at = r => 'left:' + px(r.x) + ';top:' + px(r.y) + ';width:' + px(r.w) + ';height:' + px(r.h);
const word = (l, t) => '<span class="w' + (l.ar ? ' ar' : '') + '">' + t + '</span>';

// The app behind: a web page with a chart, a notes window and the taskbar.
function desktop(l) {
  const w = l.web, dir = l.ar ? ' dir="rtl"' : '';
  const bars = [62, 48, 70, 41, 88].map((v, i) => '<div class="bar-c"><i style="height:' + v + '%"></i><span>' + w.days[i] + '</span></div>').join('');
  return '<div class="win" style="' + at(BOX.web) + '"><div class="w-top"><span class="tab">' + w.tab + '</span></div>' +
    '<div class="w-url" dir="ltr">' + w.url + '</div><div class="w-body"' + dir + '><h2>' + w.h + '</h2><p>' + w.p + '</p>' +
    '<div class="chart"><b>' + w.chart + '</b><div class="bars">' + bars + '</div></div><p class="mail">' + w.mail + '</p></div></div>' +
    '<div class="win notes" style="' + at(BOX.notes) + '"><div class="w-top"><span class="tab">' + l.notes.title + '</span></div>' +
    '<div class="w-body"' + dir + '>' + l.notes.lines.map(t => '<p>' + t + '</p>').join('') + '</div></div>' +
    '<div class="task">' + ic('win-logo', 18) + '<span class="clock">14:30</span></div>';
}

const keyEl = (w, cls, inner) => '<div class="key ' + cls + '" style="grid-column:span ' + Math.round(w * 4) + '"><div class="cap">' + inner + '</div></div>';

// The keyboard as the product draws it, with the Snip key lit while a capture runs.
function keyboard(l, s, lit) {
  const k = n => ic(n, 14 * s), lb = t => '<span class="lbl">' + t + '</span>';
  const c = t => keyEl(1, 'char', '<span class="lg' + (l.ar && /[؀-ۿ]/.test(t) ? ' ar' : '') + '">' + t.replace('<', '&lt;') + '</span>');
  const row = a => a.map(c).join('');
  const side = [['mouse-right',1,1],['hand-grab',2,1],['command',3,1],['mic',3,2],['clipboard-clock',3,3],['scissors-line-dashed',3,4],['key-round',1,5],['settings-2',2,5],['power',3,5]];
  const top = H - TASK - 12 - 196 * s;
  return '<div class="kb" style="--s:' + s + ';top:' + px(top) + '"><div class="kb-top"><div class="lane">' +
    l.words.map(w => '<span class="wchip' + (l.ar ? ' ar' : '') + '">' + w + '</span>').join('') + '</div>' +
    '<span class="tb">' + k('minus') + '</span><span class="tb">' + k('x') + '</span></div><div class="kb-body"><div class="kb-main">' +
    '<div class="row">' + keyEl(1,'danger',lb(l.keys.esc)) + row(l.rows[0]) + keyEl(2,'danger',k('delete')) + '</div>' +
    '<div class="row">' + keyEl(1.5,'mod',k('arrow-right-to-line')) + row(l.rows[1]) + keyEl(1.5,'mod',lb(l.keys.del)) + '</div>' +
    '<div class="row">' + keyEl(1.75,'mod',lb(l.keys.caps)) + row(l.rows[2]) + keyEl(3.25,'enter',k('corner-down-left')) + '</div>' +
    '<div class="row">' + keyEl(1.25,'mod',k('arrow-big-up')) + row(l.rows[3]) + keyEl(1.75,'mod',k('arrow-big-up')) + keyEl(1,'mod',k('chevron-up')) + keyEl(1,'mod',k('square-menu')) + '</div>' +
    '<div class="row">' + keyEl(1.25,'mod',lb(l.keys.ctrl)) + keyEl(1,'mod',k('win-logo')) + keyEl(1,'mod',lb(l.keys.alt)) + keyEl(1.75,'lang','<span class="lbl' + (l.ar ? ' ar' : '') + '">' + l.lang + '</span>') +
    keyEl(7,'space','<span class="spn' + (l.ar ? ' ar' : '') + '">' + l.sp + '</span>') + keyEl(1,'mod',lb(l.keys.altgr)) + keyEl(1,'mod',k('chevron-left')) + keyEl(1,'mod',k('chevron-down')) + keyEl(1,'mod',k('chevron-right')) + '</div>' +
    '</div><div class="kb-side"><div class="key act" style="grid-column:1/3;grid-row:2/5"><div class="cap dpad"><span class="d up">' + k('chevron-up') + '</span><span class="d lf">' + k('chevron-left') + '</span><span class="d rt">' + k('chevron-right') + '</span><span class="d bt">' + k('chevron-down') + '</span></div></div>' +
    side.map(([n, cx, ry]) => '<div class="key ' + (n === 'power' ? 'danger' : 'act') + (lit && n === 'scissors-line-dashed' ? ' on' : '') + '" style="grid-column:' + cx + ';grid-row:' + ry + '"><div class="cap">' + k(n) + '</div></div>').join('') +
    '</div></div></div>';
}
const kbTop = s => H - TASK - 12 - 196 * s;

// The minimised keyboard: a round bubble above the taskbar's right corner.
const bubble = () => '<button class="bubble" aria-label="Keyboard" style="left:' + px(W - 64) + ';top:' + px(H - TASK - 64) + '">' + ic('keyboard', 22) + '</button>';

// The frozen screen: dimmed all over, or with a clear hole and its size tag (physical pixels at 175 %).
const dimAll = () => '<div class="dim"></div>';
const hole = r => '<div class="hole" style="' + at(r) + '"><span class="tag" dir="ltr">' + Math.round(r.w * 1.75) + ' × ' + Math.round(r.h * 1.75) + '</span></div>';
const glow = r => '<div class="dim"></div><div class="hover" style="' + at(r) + '"></div>';

// One button of a capture bar: its icon, its word and whether it is on.
function way(l, id, icon, on, cls) {
  return '<button class="cw ' + (cls || '') + '" aria-pressed="' + !!on + '" aria-label="' + l.bar[id] + '" data-way="' + id + '">' + ic(icon) + word(l, l.bar[id]) + '</button>';
}

// The capture bar in the chosen design; `on` names the pressed way and `kind` is photo or video.
function captureBar(l, design, on, kind, kbShown) {
  const tools = '<span class="sep"></span>' + way(l, 'settings', 'settings-2', false, 'q') + way(l, 'close', 'x', false, 'q');
  if (design === 'tool') {
    const seg = '<div class="seg">' + way(l, 'photo', 'camera', kind === 'photo') + way(l, 'video', 'video', kind === 'video') + '</div>';
    const shapes = kind === 'photo' ? way(l, 'region', 'square-dashed', on === 'region') + way(l, 'window', 'app-window', on === 'window') + way(l, 'full', 'monitor', on === 'full')
      : way(l, 'region', 'square-dashed', on === 'region') + way(l, 'full', 'monitor', on === 'full');
    return '<div class="cbar cb-tool">' + seg + '<span class="sep"></span>' + shapes + tools + '</div>';
  }
  const all = way(l, 'region', 'square-dashed', on === 'region') + way(l, 'window', 'app-window', on === 'window') + way(l, 'full', 'monitor', on === 'full') +
    '<span class="sep"></span>' + way(l, 'vregion', 'video', on === 'vregion', 'rec') + way(l, 'vfull', 'monitor-play', on === 'vfull', 'rec');
  if (design === 'keys') return '<div class="cbar cb-keys">' + all + tools + '</div>';
  const y = kbShown ? kbTop(0.8) - 86 : H - TASK - 86;
  return '<div class="cbar cb-dock" style="top:' + px(y) + '">' + way(l, 'close', 'x', false, 'q') + '<span class="sep"></span>' + all + '<span class="sep"></span>' + way(l, 'settings', 'settings-2', false, 'q') + '</div>';
}

// The hint under the bar (or above the dock).
function hint(text, design, kbShown) {
  const y = design === 'dock' ? (kbShown ? kbTop(0.8) - 124 : H - TASK - 124) : (design === 'keys' ? 72 : 64);
  return '<div class="hint" style="top:' + px(y) + '">' + text + '</div>';
}

// The keycap strip: items are [id, icon, word, extra class]; it sits under `r`, above it, or at the bottom centre.
function strip(l, items, r, done) {
  const caps = items.map(([id, icon, t, cls]) => {
    if (id === 'len') return '<div class="key len"><span class="cap">' + ic('film') + '<span dir="ltr">' + t + '</span></span></div>';
    const d = done === id, w = d ? l.done[id] : t;
    const pressed = (cls || '').startsWith('tog') ? ' aria-pressed="' + (cls === 'tog on') + '"' : '';
    return '<div class="key ' + (cls || '') + '"><button class="c cap" data-strip="' + id + '" aria-label="' + w + '"' + pressed + '>' + ic(d ? 'check' : icon) + word(l, w) + '</button></div>';
  }).join('');
  let pos;
  if (!r) pos = 'left:50%;transform:translateX(-50%);top:' + px(H - TASK - 58);
  else {
    const below = r.y + r.h + 10, y = below + 44 > H - TASK ? r.y - 52 : below;
    pos = 'left:' + px(r.x + r.w / 2) + ';transform:translateX(-50%);top:' + px(y);
  }
  return '<div class="strip' + (done ? ' go' : '') + '" style="' + pos + '">' + caps + '</div>';
}
const photoStrip = (l, r, done) => strip(l, [['copy','copy',l.strip.copy],['edit','square-pen',l.strip.edit],['save','save',l.strip.save],['close','x',l.strip.close]], r, done);

// The video strip before recording: Record, the two sound switches and Close.
function readyStrip(l, r, mic, snd) {
  return strip(l, [['record','circle-dot',l.strip.record,'enter'],['mic', mic ? 'mic' : 'mic-off', l.strip.mic, 'tog' + (mic ? ' on' : '')],
    ['sound', snd ? 'volume-2' : 'volume-x', l.strip.sound, 'tog' + (snd ? ' on' : '')],['close','x',l.strip.close]], r);
}

// The countdown before a video: the part outlined and the seconds left.
function countdown(l, r, n) {
  return '<div class="soon" style="' + at(r) + '"></div><div class="count" style="left:' + px(r.x + r.w / 2) + ';top:' + px(r.y + r.h / 2) + '">' +
    '<b dir="ltr">' + n + '</b>' + word(l, l.rec.starts) + '</div>';
}

// Recording: the red edge and the bar above it with the time, the two sound switches, Stop and Cancel.
function recording(l, r, t, mic, snd) {
  const tog = (id, a, b, on) => '<button class="rb tog' + (on ? ' on' : '') + '" data-rec="' + id + '" aria-pressed="' + on + '" aria-label="' + l.strip[id] + '">' + ic(on ? a : b) + '</button>';
  const y = r.y > 60 ? r.y - 50 : 10;
  return '<div class="edge" style="' + at(r) + '"></div><div class="recbar" style="left:' + px(r.x + r.w / 2) + ';top:' + px(y) + '">' +
    '<span class="dot" aria-hidden="true"></span><span class="time" dir="ltr" aria-label="' + l.rec.live + '">' + t + '</span><span class="sep"></span>' +
    tog('mic', 'mic', 'mic-off', mic) + tog('sound', 'volume-2', 'volume-x', snd) + '<span class="sep"></span>' +
    '<button class="rb stop" data-rec="stop" aria-label="' + l.rec.stop + '">' + ic('square') + word(l, l.rec.stop) + '</button>' +
    '<button class="rb" data-rec="cancel" aria-label="' + l.rec.cancel + '">' + ic('x') + '</button></div>';
}

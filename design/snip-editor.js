// Snip mock-up windows: the editor (photo and video), the close question and the Snip panel.
'use strict';

const TOOLS = [['crop','crop'],['pen','pencil'],['hl','highlighter'],['box','square'],['circle','circle'],['arrow','move-up-right'],
  ['line','slash'],['text','type'],['cover','eye-off'],['steps','list-ordered'],['eraser','eraser']];
const ED = {x:150, y:18, w:980, h:540};

// The picture the editor holds: the region taken in step 2, laid out at fixed spots so the drawings land on it.
function picture(l) {
  const w = l.web, dir = l.ar ? ' dir="rtl"' : '';
  const bars = [62, 48, 70, 41, 88].map((v, i) => '<div class="bar-c"><i style="height:' + v + '%"></i><span>' + w.days[i] + '</span></div>').join('');
  return '<div class="pic"><h2' + dir + '>' + w.h + '</h2><p class="p1"' + dir + '>' + w.p + '</p>' +
    '<div class="chart"' + dir + '><b>' + w.chart + '</b><div class="bars">' + bars + '</div></div><p class="mail"' + dir + '>' + w.mail + '</p></div>';
}

// The drawings on the picture, in its 520 x 300 space: a highlight, a box, an arrow, typed text, two steps and a cover.
function drawings(l, tool) {
  const red = INK[0][1], blue = INK[3][1], yellow = INK[1][1], green = INK[2][1];
  const step = (x, n) => '<circle cx="' + x + '" cy="122" r="11" fill="' + blue + '"/><text x="' + x + '" y="126.5" fill="#fff" font-size="13" font-weight="600" text-anchor="middle">' + n + '</text>';
  const tx = l.ar ? 498 : 386, anchor = l.ar ? 'end' : 'start';
  const edit = tool === 'text' ? '<rect x="' + (l.ar ? 392 : 380) + '" y="62" width="112" height="26" fill="none" stroke="' + red + '" stroke-dasharray="4 3"/>' : '';
  return '<svg class="ink" viewBox="0 0 520 300" aria-hidden="true">' +
    '<rect x="14" y="14" width="336" height="24" fill="' + yellow + '" opacity=".42"/>' +
    '<rect x="287" y="96" width="50" height="136" rx="3" fill="none" stroke="' + red + '" stroke-width="3"/>' +
    '<ellipse cx="154" cy="160" rx="32" ry="58" fill="none" stroke="' + green + '" stroke-width="3"/>' +
    '<path d="M440 96 L352 150" stroke="' + red + '" stroke-width="4" stroke-linecap="round"/><path d="M352 150 L370 132 M352 150 L377 152" stroke="' + red + '" stroke-width="4" stroke-linecap="round" fill="none"/>' +
    edit + '<text x="' + tx + '" y="81" fill="' + red + '" font-size="17" font-weight="600" text-anchor="' + anchor + '" style="font-family:var(--f-arabic)">' + l.ed.note + '</text>' +
    step(52, 1) + step(312, 2) + '<rect x="' + (l.ar ? 166 : 76) + '" y="256" width="' + (l.ar ? 138 : 124) + '" height="22" rx="2" fill="#1B1B1B"/></svg>';
}

const tbtn = (id, icon, t, on, cls) => '<button class="eb ' + (cls || '') + '" data-ed="' + id + '" aria-pressed="' + !!on + '" aria-label="' + t + '" title="' + t + '">' + ic(icon) + '</button>';
const wbtn = (id, icon, l, t, cls) => '<button class="eb wide ' + (cls || '') + '" data-ed="' + id + '" aria-label="' + t + '">' + ic(icon) + word(l, t) + '</button>';
const stepper = (l, label, value) => '<span class="olbl">' + word(l, label) + '</span><span class="stp"><button class="eb" aria-label="−">' + ic('minus') + '</button><span class="val" dir="auto">' + value + '</span><button class="eb" aria-label="+">' + ic('plus') + '</button></span>';

// The tool's options: its name, the inks and its thickness or text size.
function options(l, S) {
  const t = TOOLS.find(x => x[0] === S.tool) || TOOLS[1];
  const inks = INK.map(([id, hex]) => '<button class="ink-b" data-ink="' + id + '" aria-pressed="' + (S.ink === id) + '" aria-label="' + l.ink[id] + '" title="' + l.ink[id] + '" style="--c:' + hex + '"></button>').join('');
  const size = S.tool === 'text' ? stepper(l, l.ed.tsize, '18') : stepper(l, l.ed.size, '<i class="dotv"></i>');
  return '<span class="tname">' + ic(t[1]) + word(l, l.ed[t[0]]) + '</span><span class="sep"></span><span class="olbl">' + word(l, l.ed.colour) + '</span><span class="inks">' + inks + '</span><span class="sep"></span>' + size;
}

// The video's controls: Play, the three speeds and the length they give.
function videoBar(l, S) {
  const len = {1:'1:24', 2:'0:42', 4:'0:21'}[S.speed];
  const sp = [1, 2, 4].map(n => '<button class="eb sp" data-speed="' + n + '" aria-pressed="' + (S.speed === n) + '"><span dir="ltr">' + n + '×</span></button>').join('');
  return wbtn('play', 'play', l, l.ed.play) + '<span class="sep"></span><span class="olbl">' + word(l, l.ed.speed) + '</span><span class="segs">' + sp + '</span>' +
    '<span class="sep"></span><span class="olbl">' + word(l, l.ed.length) + '</span><span class="val" dir="ltr">' + len + '</span><span class="muted">' + word(l, l.ed.playSay) + '</span>';
}

// The editor window; `video` swaps the drawing tools for Speed and Play.
function editor(l, S, video) {
  const tools = TOOLS.map(([id, icon]) => tbtn(id, icon, l.ed[id], S.tool === id)).join('');
  const history = tbtn('undo', 'undo-2', l.ed.undo) + tbtn('redo', 'redo-2', l.ed.redo);
  const acts = wbtn('copy', 'copy', l, l.ed.copy) + wbtn('save', 'save', l, l.ed.save, 'accent');
  const title = '<div class="ed-title"><span class="grip" title="' + l.ed.move + '">' + ic('grip-horizontal') + '</span>' + ic('scissors-line-dashed') + word(l, video ? l.ed.title + ' · ' + l.ed.video : l.ed.title) +
    '<span class="fill"></span>' + tbtn('max', 'maximize-2', l.ed.max, false, 'q') + tbtn('close', 'x', l.ed.close, false, 'q') + '</div>';
  const pic = '<div class="frame"><div class="pic-wrap">' + picture(l) + (video ? '<div class="poster">' + ic('play', 46) + '<span dir="ltr">1:24</span></div>' : drawings(l, S.tool)) + '</div></div>';
  const grow = '<span class="grow" title="' + l.ed.resize + '">' + ic('move-diagonal-2') + '</span>';
  let body;
  if (video) body = '<div class="ed-row">' + videoBar(l, S) + '<span class="fill"></span>' + acts + '</div><div class="canvas">' + pic + '</div>';
  else if (S.editor === 'side') body = '<div class="ed-split"><div class="ed-col">' + tools + '<span class="hsep"></span>' + history + '</div><div class="ed-main"><div class="canvas">' + pic + '</div>' +
    '<div class="ed-row low">' + options(l, S) + '<span class="fill"></span>' + acts + '</div></div></div>';
  else body = '<div class="ed-row">' + tools + '<span class="sep"></span>' + history + '<span class="fill"></span>' + acts + '</div><div class="ed-row opts">' + options(l, S) + '</div><div class="canvas">' + pic + '</div>';
  return '<section class="editor" style="' + at(ED) + '" aria-label="' + l.ed.title + '">' + title + body + grow + '</section>';
}

// The question Close asks when the snip was neither copied nor saved.
function ask(l) {
  return '<div class="scrim" style="' + at(ED) + '"></div><div class="ask" role="alertdialog" aria-label="' + l.ask.q + '" style="left:' + px(ED.x + ED.w / 2) + ';top:' + px(ED.y + 170) + '">' +
    '<div class="ask-q">' + ic('save') + word(l, l.ask.q) + '</div><div class="ask-b">' + wbtn('asksave', 'save', l, l.ask.save, 'accent') +
    wbtn('drop', 'trash-2', l, l.ask.drop) + wbtn('back', 'undo-2', l, l.ask.back) + '</div></div>';
}

// One row of the Snip panel: its label and its control.
const prow = (l, label, ctl, say) => '<div class="prow"><span class="plbl">' + word(l, label) + '</span><span class="pctl">' + ctl + '</span>' + (say ? '<span class="psay">' + word(l, say) + '</span>' : '') + '</div>';
const pick = (l, key, opts, cur) => '<span class="segs">' + opts.map(([v, t]) => '<button class="eb sp" data-' + key + '="' + v + '" aria-pressed="' + (String(cur) === String(v)) + '">' + word(l, t) + '</button>').join('') + '</span>';

// The Snip panel above the keyboard, with a Capture page and a Video page.
function panel(l, S) {
  const p = l.panel, tabs = p.tabs.map((t, i) => '<button class="tabb" data-tab="' + i + '" aria-pressed="' + (S.tab === i) + '">' + word(l, t) + '</button>').join('');
  const step = (v) => '<span class="stp"><button class="eb" aria-label="−">' + ic('minus') + '</button><span class="val">' + word(l, v) + '</span><button class="eb" aria-label="+">' + ic('plus') + '</button></span>';
  const rows = S.tab === 0
    ? prow(l, p.kb, pick(l, 'kb', [['bubble', p.bubble], ['hidden', p.hidden], ['shown', p.shown]], S.kb), p.say[S.kb]) +
      prow(l, p.folder, '<span class="path" dir="ltr">…\\AppData\\Local\\KeyXtend\\Snips</span>') +
      '<div class="prow acts">' + wbtn('change', 'folder', l, p.change) + wbtn('open', 'folder-open', l, p.open) + '</div>'
    : prow(l, p.count, step(l.secs(3))) + prow(l, p.fps, pick(l, 'fps', [[15, '15'], [30, '30']], S.fps)) + prow(l, p.max, step(l.mins(30))) +
      prow(l, p.mic, pick(l, 'micd', [[0, p.off], [1, p.on]], S.mic ? 1 : 0)) + prow(l, p.sound, pick(l, 'sndd', [[0, p.off], [1, p.on]], S.snd ? 1 : 0));
  const y = kbTop(0.8) - 318;
  return '<section class="panel" style="top:' + px(y) + '" aria-label="' + p.title + '"><div class="ph">' + ic('scissors-line-dashed') + word(l, p.title) + '<span class="fill"></span>' +
    tbtn('pclose', 'x', p.close, false, 'q') + '</div><div class="tabs">' + tabs + '</div><div class="pbody">' + rows + '</div></section>';
}

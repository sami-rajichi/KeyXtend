// Part 2 of the voice typing mock-up (design/voice-bar.html): cloud services. Its icons, pages and acts draw the texts
// and rows of voice-cloud-data.js, which loads first, and extend part 1's PAGE, ACT, CTL and SAY before boot()
// draws the scene. Reference only, never shipped.
'use strict';

// Lucide icons (ISC): cloud is bundled in assets/icons/lucide; part 2 adds key, triangle-alert and external-link.
Object.assign(ICONS, {
  'cloud':'<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/>',
  'key':'<path d="m15.5 7.5 2.3 2.3a1 1 0 0 0 1.4 0l2.1-2.1a1 1 0 0 0 0-1.4L19 4"/><path d="m21 2-9.6 9.6"/><circle cx="7.5" cy="15.5" r="5.5"/>',
  'triangle-alert':'<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  'external-link':'<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>'
});

// Counts, money and dates in the language shown, how long ago a card's facts were read, and a row by its id.
const cn = () => COUNT[S.lang];
const money = (v, cur) => CUR[cur][S.lang](Number.isInteger(v) ? String(v) : nf(v, 2), v);
const dateOf = iso => { const [y, m, d] = iso.split('-').map(Number); return d + ' ' + l.mo[m - 1] + ' ' + y; };
const daysTo = iso => Math.round((Date.parse(TODAY) - Date.parse(iso)) / 864e5);
const byId = id => SVC.find(s => s.id === id);
const svcName = s => s.id === 'custom' ? l.custom : s.name;

// A card's sentences, from the row's fact ids and numbers: counts by their kind, money in the row's currency, a switch's
// name as it is.
const fct = v => Array.isArray(v) ? v : [v, {}];
function fill(text, o, cur) {
  const v = {}, c = cn();
  Object.keys(o).forEach(k => { const x = o[k]; v[k] = ['c', 'p', 'lo', 'hi'].includes(k) ? money(x, cur) : k === 'fee' ? nf(x) : k === 'sw' ? bdi(x) : c[k] ? c[k](x) : x; });
  return t(text, v);
}
const fact = (group, s, k = group) => { const [id, o] = fct(s[k]); return fill(l.fx[group][id], o, s.cur); };
const freeText = s => fact('free', s);
const bdi = s => '<bdi dir="ltr">' + s + '</bdi>';
// A service's mark; a key set but not yet tested is Ready. Untested is a tag for a connection never checked with a real key.
const MARK = {off:['mOff', '', 'out'], key:['mKey', 'key', ''], ready:['mReady', 'check', '']};
const stateMark = id => { const [k, icon, cls] = MARK[S.svc[id].st]; return mark(l[k], icon, cls); };
const untestedTag = s => s.untested ? '<span class="tag' + A + '"' + D + '>' + l.mUntested + '</span>' : '';
const cloudReady = id => S.cloud && S.svc[id].st === 'ready';
// The Cloud side lists the models of switched-on services (a typed id once typed); one needing a key says so. Edge 17:
// with OpenRouter's keep-nothing switch on, a model with no such host is Unavailable (the sample's typed id has none).
const cloudOn = r => S.cloud && S.svc[r.cloud].st !== 'off' && (r.id !== 'or-typed' || !!S.typed.model.openrouter);
const cloudShown = r => cloudReady(r.cloud) && cloudOn(r);
const cloudLook = r => r.cloud === 'openrouter' && S.svc.openrouter.keep0 ? 'na' : S.svc[r.cloud].st === 'key' ? 'key' : r.st;

// Picks: edge 21, a cloud pick whose service stops being ready falls back to the language's local default.
function fallback() {
  const u = S.models && S.models.find(m => m.st === 'use');
  if (!u || !u.cloud || (cloudShown(u) && cloudLook(u) !== 'na')) return;
  u.st = 'ready'; if (S.models[0].st === 'ready') S.models[0].st = 'use'; S.engine = 'local';
}
// "Dictating with": a cloud engine switches its service on, ready, and picks its model.
const ENGINE = {cloud:['groq', 'gq-turbo'], elevenlabs:['elevenlabs', 'el-scribe']};
function useEngine(v) {
  const e = ENGINE[v];
  if (e) { S.cloud = true; const sv = S.svc[e[0]]; Object.assign(sv, {st:'ready', key:sv.key || KEY_LEN, test:sv.test || 'ok', on:sv.on || READ}); }
  const want = S.models.find(m => m.id === (e ? e[1] : S.models[0].id));
  S.models.forEach(m => { if (m.st === 'use') m.st = 'ready'; });
  if (want.st === 'ready') want.st = 'use';
}
function addCloud() {
  S.models.splice(1, 0, ...Object.keys(CM).map(id => ({id, st:'ready', def:false, p:0, paused:false, cloud:CM[id][1]})));
  if (ENGINE[S.engine]) useEngine(S.engine);
}

// The bar's cloud mark, the home row's value, the Models page's Local | Cloud chips, a cloud model's row and its bands.
function cloudChip(kind) {
  const u = inUse();
  if (!u.cloud) return '';
  const name = byId(u.cloud).name;
  return '<span class="lc cm" role="img" aria-label="' + t(kind === 'send' ? l.onBarSend : l.onBar, {name}) + '">' + ic('cloud') + '<span>' + name + '</span></span>';
}
// The first dictation to a service with a warning (VoiceWarn.qml): the bar widens and the card's warning wraps under its
// row in warn_ink while it listens, holding nothing back.
const barBase = bar;
bar = () => {
  if (NOTE[S.bar]) return noteBar();
  const html = barBase(), u = S.models && inUse(), s = u && u.cloud && byId(u.cloud);
  if (S.bar !== 'listening' || !s || !s.warn) return html;
  const line = '<div class="vw' + A + '"' + D + '>' + t(l.warns[s.warn], {sw:bdi(s.sw || '')}) + '</div>';
  return html.replace('<div class="vbar', '<div class="vbar wide warns').replace(/<\/div>$/, line + '</div>');
};
// A cloud note that the service must settle (owner, 2026-11-20) adds Open service to Try again and Use the local model:
// one click to the card, or to the key page for the Key note. The acts sit under a long line, as the Windows 11 InfoBar's.
const NOTE = {nkey:'key', nterms:'terms', naccount:'account', nallow:'allowance', noff:'off'};
const noteSvc = () => { const u = S.models && inUse(); return u && u.cloud ? u.cloud : SVC[0].id; };
function noteBar() {
  const k = NOTE[S.bar], line = '<div class="vtx' + A + '"' + D + '><div class="w">' + t(l.notes[k], {at:bdi(RENEW_AT)}) + '</div></div>';
  const acts = vt('again', 'rotate-ccw', l.again, 'enter') + vt('uselocal', 'cpu', l.useLocal) + vt('opensvc', k === 'key' ? 'key' : 'file', l.openSvc);
  return '<div class="vbar wide cn' + (S.bar !== seen.bar ? ' in' : '') + '" role="toolbar" aria-label="' + l.title + '">' + slot('cloud') + line +
    vb('gear', 'settings-2', l.gear) + vb('close', 'x', l.close) + '<div class="cna">' + acts + '</div></div>';
}
const cloudHome = () => S.cloud ? l.on + ' · ' + cn().svc(SVC.filter(s => S.svc[s.id].st === 'ready').length) : l.off;
// Local | Cloud (owner, 2026-11-17): the panel's filter chips (PanelChip.qml), the shown one filled and checked.
const sides = on => chips(l.sides, [['side:local', l.sideLocal], ['side:cloud', l.sideCloud]], on === 'cloud' ? 1 : 0);
const CMARK = {na:['mNa', 'x', 'out'], key:['mKey', 'key', ''], ready:['ready', 'check', '']};
function cloudModelRow(r, i) {
  if (!cloudOn(r)) return '';
  const st = cloudLook(r), svc = mark(byId(r.cloud).name, 'cloud'), [k, icon, cls] = CMARK[st] || CMARK.ready;
  const right = st === 'use' ? mark(l.inUse, 'check', 'lit') + svc : svc + mark(l[k], icon, cls);
  return tickRow([CK, '1fr', 'auto'], tickBox(i, true) + cell(M[r.id][0], st === 'na' ? l.noKeep : l.cloudModelSub) + '<div class="marks">' + right + '</div>', i);
}
function cloudBand() {
  const on = SVC.some(s => S.svc[s.id].st !== 'off'), say = !S.cloud ? l.masterOff : on ? l.cloudLine : l.noneOn;
  return '<div class="pn' + A + '">' + ic('cloud') + '<span class="x"' + D + '>' + say + '</span>' + btn('go:cloud', 'settings-2', l.cloud) + '</div>';
}
// The Cloud side: its own paged list; Use works only on a model whose service has a key.
function cloudModels() {
  const rows = S.models.map((r, i) => r.cloud ? cloudModelRow(r, i) : '').filter(Boolean), r = ticked();
  return {icon:'audio-lines', title:l.models, sub:l.forLang, top:sides('cloud') + cloudBand(), body:list(rows),
    foot:gap + btn('use', 'check', l.use, 'enter', !r || !r.cloud || cloudLook(r) !== 'ready')};
}
const slowBand = () => '<div class="pn' + A + '">' + ic('cpu') + '<span class="x"' + D + '>' + t(l.slow, {lang:(l.langIn || l.langs)[S.lang]}) + '</span>' + btn('go:cloud', 'cloud', l.cloud) + '</div>';

// The Cloud page: the master switch, then the services, five a page; a ticked row's acts in the footer. Remove key acts on
// the ticked row whenever its key is sealed, on or off (owner, 2026-11-21). With the master switch off it lists none.
function cloudPage() {
  const on = S.cloud, s = S.tick.length ? SVC[S.tick[0]] : null, st = s ? S.svc[s.id].st : '', live = on && !!s;
  const rows = on ? SVC.map((v, i) => {
    const sub = v.id === 'custom' ? l.customSub : t(l.freeLine, {v:freeText(v)});
    const inner = tickBox(i, true) + cell(svcName(v) + untestedTag(v), sub) + '<div class="marks">' + (v.warn ? mark(l[WARN_MARK[v.warn]], 'triangle-alert', 'warn') : '') + stateMark(v.id) + '</div>';
    return tickRow([CK, '1fr', 'auto'], inner, i);
  }) : [];
  const band = '<div class="pn swb' + A + '" data-a="cloudsw" role="switch" aria-checked="' + on + '" tabindex="0">' + ic('cloud') + '<span class="x"' + D + '>' + l.master + '</span>' +
    '<span class="onoff' + A + '"' + D + '>' + (on ? l.on : l.off) + '</span><span class="sw' + (on ? ' on' : '') + '"></span></div>';
  return {icon:'cloud', title:l.cloud, sub:l.cloudSub, top:band + (on ? '' : note('power', l.masterOff)), body:list(rows),
    foot:btn('facts', 'file', l.facts, '', !live) + btn('svc', 'key', l.keyModel, '', !live || st === 'off') + btn('rmsvc', 'trash-2', l.remove, 'danger', !live || !S.svc[s.id].key) +
      btn('svcoff', 'power', l.swOff, '', !live || st === 'off') + gap +
      btn('swon', 'power', l.swOn, 'enter', !live || st !== 'off')};
}

// A service's card: its facts with the date read, a link to its page, a warning band when it warns. The least billed and
// the terms of use show only for a row that has them.
function cardPage() {
  const s = byId(S.card.id), v = S.svc[s.id], ask = S.card.ask, read = S.card.read || s.read || READ, old = daysTo(read) > FACTS_DAYS;
  const rows = [['where'], ['trains', s.tw], ['keeps'], ['free'], ['light', false, USE_H.light, 'price'], ['heavy', false, USE_H.heavy, 'price'], ['bills'], ['rule']]
    .filter(([k]) => s[k]).map(([k, warm, h, group]) => [k, fact(group || k, s, k), warm, h]);
  const facts = '<div class="facts' + A + '"' + D + '>' + rows.map(([k, val, warm, h]) => '<span class="fk">' + t(l['c_' + k], {h:h ? cn().hm(h) : ''}) + '</span>' +
    '<span class="fv' + (warm ? ' warn' : '') + '">' + (warm ? ic('triangle-alert') : '') + '<span>' + val + '</span></span>').join('') + '</div>';
  const host = s.site ? s.site.split('/')[0] : '', name = s.id === 'custom' ? l.customTo : s.name;
  const when = !host ? '' : '<div class="ro' + (old ? ' old' : '') + A + '">' + ic(old ? 'triangle-alert' : 'history') + '<span class="x"' + D + '>' + t(l.readOn, {date:dateOf(read)}) +
    (old ? ' ' + l.old : '') + '</span>' + btn('link', 'external-link', t(l.link, {site:bdi(host)}), '', false, t(l.linkName, {site:s.site})) + '</div>';
  const untested = s.untested ? '<div class="ro' + A + '">' + ic('activity') + '<span class="x"' + D + '>' + l.untestedLine + '</span></div>' : '';
  const warn = s.warn ? '<div class="pn warn' + A + '">' + ic('triangle-alert') + '<span class="x"' + D + '>' + t(l.warns[s.warn], {sw:bdi(s.sw || '')}) + '</span></div>' : '';
  const foot = ask ? '<span class="x' + A + '"' + D + '>' + t(l.confirmHint, {name}) + '</span>' + btn('cardno', 'x', l.cancel) + btn('confirm', 'check', l.confirm, 'enter')
    : '<span class="x' + A + '"' + D + '>' + t(l.cardOn, {date:dateOf(v.on || READ)}) + '</span>';
  return {icon:'cloud', title:svcName(s), sub:ask ? l.cardSub : l[MARK[v.st][0]], top:warn, body:'<div class="pb"><div class="card">' + facts + when + untested + '</div></div>', foot};
}

// The panel's own line (PanelLine.qml): its caption, then its text, dots for a key, or its hint; a caret while aimed at.
function pline(name, cap, val, hint, secret) {
  const on = S.aim === name, crt = on ? '<i class="crt"></i>' : '';
  const body = val ? bdi((secret ? '•'.repeat(val) : val) + crt) : crt || '<span>' + hint + '</span>';
  return '<div class="pline' + (on ? ' on' : '') + '" data-a="aim:' + name + '" role="textbox" aria-label="' + cap + '" tabindex="0"><span class="lk' + A + '"' + D + '>' + cap + '</span>' +
    '<span class="lv' + (secret && val ? ' mask' : '') + (val ? '' : ' lh' + A) + '">' + body + '</span></div>';
}
// A Test's answer, or what a key without one means.
function keyResult(s, v) {
  const line = (cls, icon, text) => '<div class="res' + cls + A + '">' + ic(icon) + '<span' + D + '>' + text + '</span></div>';
  if (v.test === 'testing') return '<div class="res' + A + '"><span' + D + '>' + l.testing + '</span><div class="prog run"><i></i></div></div>';
  if (v.test === 'ok') return '<div class="res">' + mark(l.ok, 'check', 'lit') + '</div>';
  if (v.test === 'bad') return line(' bad', 'triangle-alert', l.bad);
  return v.key && !s.test ? line('', 'activity', l.firstTest) : '';
}
// A service's page: the key line with Paste and Test, the address and model lines, its switch and where to make a key.
function svcPage() {
  const s = byId(S.cur), v = S.svc[s.id], addr = S.typed.addr[s.id] || '', typed = S.typed.model[s.id] || '';
  const lrow = inner => '<div class="lrow">' + inner + '</div>';
  const na = s.keep0 && v.keep0 && typed ? '<div class="res' + A + '">' + mark(l.mNa, 'x', 'out') + '<span' + D + '>' + l.noKeep + '</span></div>' : '';
  const lines = lrow(pline('key', l.keyCap, v.key, l.keyHint, true) + btn('paste', 'clipboard-paste', l.paste) + btn('test', 'activity', l.test, '', !v.key || !s.test || v.test === 'testing')) +
    keyResult(s, v) +
    (s.addr ? lrow(pline('addr', l.addrCap, addr, t(l.addrHint, {ex:bdi('https://example.com/v1/audio/transcriptions')}))) : '') +
    (addr && !addr.startsWith('https://') ? '<div class="res bad' + A + '">' + ic('triangle-alert') + '<span' + D + '>' + t(l.addrBad, {ex:bdi('https://example.com')}) + '</span></div>' : '') +
    lrow(pline('model', l.modelCap, typed, l.modelHint)) + na +
    (s.keep0 ? '<div class="swr" data-a="keep0" role="switch" aria-checked="' + v.keep0 + '" tabindex="0"><span class="x' + A + '"' + D + '>' + l.keepNothing + '</span>' +
      '<span class="onoff' + A + '"' + D + '>' + (v.keep0 ? l.on : l.off) + '</span><span class="sw' + (v.keep0 ? ' on' : '') + '"></span></div>' : '') +
    (s.keys ? '<div class="lrow ko' + A + '">' + ic('key') + '<span class="x"' + D + '>' + t(l.makeKey, {site:bdi(s.keys)}) + '</span>' + btn('link', 'external-link', l.open, '', false, t(l.linkName, {site:s.keys})) + '</div>' : '');
  return {icon:'key', title:svcName(s), sub:l[MARK[v.st][0]], body:'<div class="pb"><div class="lines">' + lines + '</div></div>',
    foot:btn('facts', 'file', l.facts) + btn('rmkey', 'trash-2', l.remove, 'danger', !v.key) + '<span class="x' + A + '"' + D + '>' + l.keySealed + '</span>' + btn('svcoff', 'power', l.swOff)};
}
Object.assign(PAGE, {cloud:cloudPage, svccard:cardPage, card:cardPage, cardwarn:cardPage, cardgem:cardPage, cardold:cardPage, svcpage:svcPage, key:svcPage, keybad:svcPage,
  address:svcPage, model:svcPage, slow:() => PAGE.models(), mcloud:cloudModels, paste:() => PAGE.options()});

// Navigation: the controls' presets, and Back from a card or a service's page to where it was opened.
function openCard(id, ask, ret, read) { S.card = {id, ask, read:read || ''}; S.ret = ret; }
function openSvc(id) { S.cur = id; S.ret = 'cloud'; }
const PRESET = {
  models:() => { S.weak = false; }, slow:() => { S.weak = true; }, paste:() => { S.opt.insert = 1; S.pg = Math.floor(OPTS.findIndex(o => o[0] === 'insert') / 5); },
  card:() => openCard('groq', true, 'cloud'), cardwarn:() => openCard('elevenlabs', true, 'cloud'), cardgem:() => openCard('gemini', true, 'cloud'),
  cardold:() => openCard('groq', true, 'cloud', OLD_READ),
  key:() => { openSvc('groq'); Object.assign(S.svc.groq, {st:'ready', key:KEY_LEN, test:'ok', will:'ok', on:S.svc.groq.on || READ}); },
  keybad:() => { openSvc('groq'); Object.assign(S.svc.groq, {st:'key', key:KEY_LEN, test:'bad', will:'bad', on:S.svc.groq.on || READ}); },
  address:() => { openSvc('custom'); if (S.svc.custom.st === 'off') Object.assign(S.svc.custom, {st:'key', on:TODAY}); S.typed.addr.custom = HOME; S.aim = 'addr'; },
  model:() => { openSvc('openrouter'); Object.assign(S.svc.openrouter, {st:'ready', key:KEY_LEN, test:'', will:'ok', keep0:true, on:S.svc.openrouter.on || TODAY});
    S.typed.model.openrouter = OR_MODEL; S.aim = 'model'; }};
function cloudGo(p) { S.aim = ''; if (PRESET[p]) PRESET[p](); if (p === 'svcpage') S.ret = 'cloud'; fallback(); }
const SUB = ['card','cardwarn','cardgem','cardold','key','keybad','address','model','svccard','svcpage'];
const tickedSvc = () => SVC[S.tick[0]];
const curSvc = () => S.panel === 'cloud' ? tickedSvc() : byId(S.cur);
let ctimer = 0;
const backBase = ACT.back, useBase = ACT.use;
Object.assign(ACT, {
  back:() => SUB.includes(S.panel) ? go(S.ret || 'cloud') : backBase(),
  use:() => { useBase(); const c = inUse().cloud; S.engine = ENGINE[c] ? c : c ? 'cloud' : 'local'; },
  cloudsw:() => { S.cloud = !S.cloud; S.tick = []; fallback(); },
  swon:() => { openCard(tickedSvc().id, true, 'cloud'); go('svccard'); },
  facts:() => { const s = curSvc(); openCard(s.id, S.svc[s.id].st === 'off', S.panel); go('svccard'); },
  cardno:() => go(S.ret),
  confirm:() => { const v = S.svc[S.card.id]; Object.assign(v, {st:v.key ? 'ready' : 'key', on:TODAY}); openSvc(S.card.id); go('svcpage'); },
  svc:() => { openSvc(tickedSvc().id); go('svcpage'); },
  svcoff:() => { Object.assign(S.svc[curSvc().id], {st:'off', test:''}); fallback(); if (S.panel !== 'cloud') go('cloud'); },
  paste:() => { Object.assign(S.svc[S.cur], {key:KEY_LEN, test:'', will:'ok', st:'ready'}); S.aim = ''; fallback(); },
  test:() => {
    const v = S.svc[S.cur]; v.test = 'testing'; clearTimeout(ctimer);
    ctimer = setTimeout(() => { const ok = v.will === 'ok'; Object.assign(v, {test:ok ? 'ok' : 'bad', st:ok ? 'ready' : 'key'}); fallback(); render(); }, PACE.test);
  },
  rmkey:() => { Object.assign(S.svc[S.cur], {key:0, test:'', st:'key'}); fallback(); },
  rmsvc:() => { const v = S.svc[tickedSvc().id]; Object.assign(v, {key:0, test:'', st:v.st === 'off' ? 'off' : 'key'}); fallback(); },
  uselocal:transcribe,
  opensvc:() => { const id = noteSvc(), k = NOTE[S.bar]; if (k === 'key') { openSvc(id); go('svcpage'); } else { openCard(id, ['terms','off'].includes(k), 'cloud'); go('svccard'); } },
  side:x => go(x === 'cloud' ? 'mcloud' : S.weak ? 'slow' : 'models'),
  aim:x => { S.aim = S.aim === x ? '' : x; }, keep0:() => { const v = S.svc[S.cur]; v.keep0 = !v.keep0; fallback(); }, link:() => {}});

// The controls on top, in English as on the other mock-ups, and what each new state shows.
CTL.cloud = [['cloud','Cloud page'],['card','Card: Groq'],['cardwarn','Card: ElevenLabs'],['cardgem','Card: Gemini'],['cardold','Card: old facts'],['key','Key works'],
  ['keybad','Key refused'],['address','Custom address'],['model','Other model'],['mcloud','Models: Cloud'],['slow','Models, slow PC'],['paste','Options: Paste']];
CTL.engine = [['local','Local model'],['cloud','Groq (cloud)'],['elevenlabs','ElevenLabs, first dictation']];
CTL.notes = [['nkey','Key'],['nterms','New terms'],['naccount','Account step'],['nallow','Allowance used'],['noff','Service off']];
SAY.home += ' Part 2 adds Cloud services as the second row, so Options moves to page 2.';
SAY.options += ' Part 2: Insert the text, Type it or Paste at once, as two filter chips (page 3).';
SAY.models += ' Part 2: Local | Cloud at the top, as the clipboard panel’s filter chips; this Local side lists only the models on this PC.';
Object.assign(SAY, {
  preparing:'<strong>Preparing.</strong> No bar yet: from the Dictation click until the microphone is truly open, only the Dictation key glows amber, as a studio’s amber stand-by light comes before the red on-air light. Narrator hears “{name}”. Click the amber key to cancel with nothing sent; otherwise the bar opens straight in the red Listening state. Click the Dictation key, or Listen again, to start it.',
  cloudbar:' <strong>Cloud mark:</strong> a cloud and the service’s name beside the language, so you always know where the voice goes; nothing leaves before Stop. At the first dictation to a service with a warning (ElevenLabs, Gemini), the bar widens and the card’s warning wraps under its row in the warning ink, holding nothing back; pick “ElevenLabs, first dictation” above to see it.',
  cloud:' <strong>Cloud services.</strong> The master switch, then the services five a page, Groq first; tick one, then act in the footer (the clipboard panel’s pattern). Marks: Off, Needs a key, Ready (a key set but not yet tested is Ready); the Untested tag beside a name marks a row no real key has checked, which is every row until the owner’s hands-on check. Only the offered rows show: OpenAI, Mistral, Together and DeepInfra stay hidden. The sample user already switched on Groq, Lemonfox and OpenRouter; in the product the switch starts off.',
  card:' <strong>Service card, before switching on.</strong> Fixed rows, as an app store’s privacy label: where the voice goes, training, what is kept, the free allowance, light and heavy use, the least billed or the terms of use when the service sets them, the date read and a link to the service’s page.',
  cardwarn:' <strong>Card with a warning.</strong> ElevenLabs trains on the voice by default, so a band in the warning colours (Windows 11 InfoBar) names the switch in its profile menu that stops it.',
  cardgem:' <strong>Gemini’s card.</strong> While the user stays on the free allowance, Google uses the voice to improve its products and human reviewers may read it, so the band warns; in the EEA, Switzerland and the UK its terms allow only paid use, from age 18.',
  cardold:' <strong>Old facts.</strong> Read ' + daysTo(OLD_READ) + ' days before the check, past facts_days (' + FACTS_DAYS + '), so the date takes the warning colour and asks to check the service’s page.',
  svccard:' <strong>Service card.</strong> The facts with their date and a link to the service’s page.',
  key:' <strong>Key line.</strong> The panel’s own line, as Quick-fill’s: the key as dots with Paste and Test, Remove key in the footer. Test said Key works.',
  keybad:' <strong>Key refused.</strong> Test answered no, in the warning colours; Paste a new key, then Test again.',
  address:' <strong>Address line.</strong> The Custom row’s address taking the letters: ring, caret and accent line (WinUI’s focused TextBox). It must be an HTTPS host name; a plain http:// or an IP address is refused, with the reason.',
  model:' <strong>Model line.</strong> OpenRouter’s Other model taking a model id. With its keep-nothing switch on, a model with no such host is Unavailable, with the reason, here and on the Models page; switch it off and the model is Ready.',
  svcpage:' <strong>Key and model.</strong> The key line, Test, the model line and where to make a key; Custom also has its address line, which must be https. Lemonfox, AssemblyAI and Custom have no test call, so the first dictation tests the key.',
  slow:' <strong>Models on a weak PC.</strong> The Local side: the default is Not for this PC, so the band says local models will be very slow here and leads to Cloud services.',
  paste:' <strong>Insert the text: Paste at once.</strong> Two filter chips, Type it the default; while Paste is chosen, its one-line hint takes the row below.',
  mcloud:' <strong>Models, Cloud side.</strong> Its own list, five a page: only the models of the services you switched on, each with the cloud mark and its service; one still needing a key says so and Use stays dimmed. The band leads to Cloud services.',
  nkey:'<strong>Cloud note: the key.</strong> Try again, Use the local model, and Open service, which opens the service’s key page in one click; the buttons sit under the line, as the Windows 11 InfoBar’s under a long message.',
  nterms:'<strong>Cloud note: new terms.</strong> Nothing is sent until the terms are confirmed again; Open service opens the card, ready to confirm.',
  naccount:'<strong>Cloud note: a step on the service’s site.</strong> Open service opens the card, with the link to the service’s page.',
  nallow:'<strong>Cloud note: the free allowance is used.</strong> It says the clock time it renews at, with its date when that is not today, never a wait that goes stale; Open service opens the card.',
  noff:'<strong>Cloud note: the service is off.</strong> Nothing was sent; Open service opens the card, where Confirm switches it on again.'});
SAY.cloud += ' Tick a row whose key is saved and Remove key in the footer deletes it, even while the service is off (ElevenLabs here); with cloud services off the page lists none.';

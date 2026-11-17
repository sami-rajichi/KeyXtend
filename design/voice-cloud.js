// Part 2 of the voice typing mock-up (design/voice-bar.html): cloud services. Its texts, sample facts, pages and acts extend
// voice-bar.js's L, PAGE, ACT, CTL and SAY before boot() draws the scene. Reference only, never shipped.
'use strict';

// Lucide icons (ISC): cloud is bundled in assets/icons/lucide; part 2 adds key, triangle-alert and external-link.
Object.assign(ICONS, {
  'cloud':'<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/>',
  'key':'<path d="m15.5 7.5 2.3 2.3a1 1 0 0 0 1.4 0l2.1-2.1a1 1 0 0 0 0-1.4L19 4"/><path d="m21 2-9.6 9.6"/><circle cx="7.5" cy="15.5" r="5.5"/>',
  'triangle-alert':'<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/>',
  'external-link':'<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>'
});

// Texts, new in part 2 (plan: voice.cloud, voice.mark-*, voice.card-*, voice.key-*, the lines and voice.on-bar-cloud).
// fx holds a card's fact sentences, filled with the row's numbers so no price or claim is written twice.
const CT = {
en:{cloud:'Cloud services', cloudSub:'Your own key · nothing leaves before Stop', master:'Use cloud services', masterOff:'Cloud services are off: none is offered and nothing is sent.',
  freeLine:'Free use: {v}', custom:'Custom address', customTo:'the address you type', customSub:'An HTTPS address, a model and a key you type', noKeep:'No host of this model promises to keep nothing.',
  mOff:'Off', mKey:'Needs a key', mUntested:'Untested', mReady:'Ready', mNa:'Unavailable', mTrains:'Trains by default',
  facts:'Facts', keyModel:'Key and model', swOff:'Switch off', swOn:'Switch on',
  cardSub:'Read this before you switch it on', cardOn:'Switched on {date}', c_where:'Where your voice goes', c_trains:'Training', c_keeps:'What is kept', c_free:'Free use',
  c_light:'Light use · {h} a month', c_heavy:'Heavy use · {h} a month', readOn:'Facts read on {date}.', old:'They may be out of date; check the service’s page.',
  link:'Open {site}', linkName:'Open {site} in the browser', confirm:'Confirm', confirmHint:'Your voice goes to {name} only after Stop, with your own key.',
  untestedLine:'Untested: not yet checked with a real key.',
  warns:{eleven:'Trains its models on your voice by default. Before you confirm, turn off “{sw}” in your ElevenLabs account.'},
  keyCap:'Key', keyHint:'Type or paste your key', paste:'Paste', test:'Test', testing:'Testing…', ok:'Key works', bad:'Key refused. Check it on the service’s page.',
  remove:'Remove key', firstTest:'No test call here: the first dictation tests the key.', keySealed:'The key is kept encrypted on this PC and never shown.',
  addrCap:'Address', addrHint:'Your Azure resource’s address', addrHintC:'Your service’s HTTPS address', addrBad:'The address must start with {https}',
  modelCap:'Other model', modelHint:'A model id from the service’s page', keepNothing:'Only hosts that keep nothing', makeKey:'Make a key at {site}',
  cloudModelSub:'In the cloud, with your key', sides:'Where the model runs', sideLocal:'Local', sideCloud:'Cloud',
  noneOn:'No cloud service is switched on yet.', cloudLine:'Models of the services you switched on; nothing leaves before Stop.', slow:'Local models for {lang} will be very slow on this PC.',
  onBar:'Your voice goes to {name} at Stop', onBarSend:'Sending your voice to {name}',
  mo:['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'],
  fx:{where:{us:'United States', eu:'European Union', host:'The host OpenRouter picks', region:'The Azure region you pick; European ones exist', yours:'The address you type', notSaid:'Not stated on its page'},
    trains:{no:'Not used for training', optIn:'Only if you opt in', byDefault:'Used for training by default', freeMay:'May be used in free mode; an account switch stops it',
      host:'The host decides', yours:'As your service does', notSaid:'Not stated on its page'},
    keeps:{none:'Nothing kept by default', afterProc:'Deleted after processing', untilSwitch:'Requests kept until an account switch is off', memory:'Held in memory only',
      routerNone:'OpenRouter keeps nothing; the host decides', yours:'As your service does', notSaid:'Not stated on its page'},
    free:{day:'{h} a day, {r} a minute', none:'None', signup:'A small sign-up allowance', firstMonth:'The first month',
      unclear:'Unclear; the Pro plan lists API credit', unverified:'Not confirmed', monthCard:'{h} a month, classic only; a card is needed',
      monthSmall:'A small monthly allowance, non-commercial use', yours:'Set by your service'},
    price:{one:'{a} a month', range:'{a} to {b} a month', from:'from {a} a month', freeOr:'{z} within the free use; paid {p}', until:'{p} until {date}',
      fee:'{p}; {f} fee on credit', yours:'Set by your service'}}},
fr:{cloud:'Services cloud', cloudSub:'Votre propre clé · rien ne part avant l’arrêt', master:'Utiliser les services cloud', masterOff:'Services cloud désactivés : aucun n’est proposé, rien n’est envoyé.',
  freeLine:'Usage gratuit : {v}', custom:'Adresse personnalisée', customTo:'l’adresse que vous saisissez', customSub:'Une adresse HTTPS, un modèle et une clé à saisir', noKeep:'Aucun hébergeur de ce modèle ne s’engage à ne rien garder.',
  mOff:'Désactivé', mKey:'Clé requise', mUntested:'Non testé', mReady:'Prêt', mNa:'Indisponible', mTrains:'Entraînement par défaut',
  facts:'Informations', keyModel:'Clé et modèle', swOff:'Désactiver', swOn:'Activer',
  cardSub:'À lire avant d’activer', cardOn:'Activé le {date}', c_where:'Où va votre voix', c_trains:'Entraînement', c_keeps:'Conservation', c_free:'Usage gratuit',
  c_light:'Usage léger · {h} par mois', c_heavy:'Usage intensif · {h} par mois', readOn:'Informations relevées le {date}.', old:'Elles datent peut-être ; vérifiez la page du service.',
  link:'Ouvrir {site}', linkName:'Ouvrir {site} dans le navigateur', confirm:'Confirmer', confirmHint:'Votre voix ne part vers {name} qu’après l’arrêt, avec votre propre clé.',
  untestedLine:'Non testé : pas encore vérifié avec une vraie clé.',
  warns:{eleven:'Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez « {sw} » dans votre compte ElevenLabs.'},
  keyCap:'Clé', keyHint:'Tapez ou collez votre clé', paste:'Coller', test:'Tester', testing:'Test en cours…', ok:'La clé fonctionne', bad:'Clé refusée. Vérifiez-la sur la page du service.',
  remove:'Retirer la clé', firstTest:'Pas de test possible ici : la première dictée testera la clé.', keySealed:'La clé est chiffrée sur ce PC et jamais affichée.',
  addrCap:'Adresse', addrHint:'L’adresse de votre ressource Azure', addrHintC:'L’adresse HTTPS de votre service', addrBad:'L’adresse doit commencer par {https}',
  modelCap:'Autre modèle', modelHint:'L’identifiant du modèle, sur la page du service', keepNothing:'Seulement des hébergeurs qui ne gardent rien', makeKey:'Créer une clé sur {site}',
  cloudModelSub:'Dans le cloud, avec votre clé', sides:'Où le modèle s’exécute', sideLocal:'Sur ce PC', sideCloud:'Dans le cloud',
  noneOn:'Aucun service cloud n’est encore activé.', cloudLine:'Modèles des services que vous avez activés ; rien ne part avant l’arrêt.', slow:'Les modèles locaux en {lang} seront très lents sur ce PC.', langIn:{en:'anglais', fr:'français', ar:'arabe'},
  onBar:'Votre voix partira chez {name} à l’arrêt', onBarSend:'Envoi de votre voix à {name}',
  mo:['janv.','févr.','mars','avr.','mai','juin','juil.','août','sept.','oct.','nov.','déc.'],
  fx:{where:{us:'États-Unis', eu:'Union européenne', host:'L’hébergeur choisi par OpenRouter', region:'La région Azure choisie ; il en existe en Europe', yours:'L’adresse saisie', notSaid:'Non précisé sur sa page'},
    trains:{no:'Non utilisée pour l’entraînement', optIn:'Seulement si vous l’acceptez', byDefault:'Utilisée pour l’entraînement par défaut', freeMay:'Possible en mode gratuit ; un réglage du compte l’empêche',
      host:'Selon l’hébergeur', yours:'Selon votre service', notSaid:'Non précisé sur sa page'},
    keeps:{none:'Rien n’est gardé par défaut', afterProc:'Supprimée après traitement', untilSwitch:'Requêtes gardées tant qu’un réglage du compte est actif', memory:'En mémoire seulement',
      routerNone:'OpenRouter ne garde rien ; l’hébergeur décide', yours:'Selon votre service', notSaid:'Non précisé sur sa page'},
    free:{day:'{h} par jour, {r} par minute', none:'Aucun', signup:'Un petit crédit à l’inscription', firstMonth:'Le premier mois',
      unclear:'Flou ; l’offre Pro inclut un crédit API', unverified:'Non confirmé', monthCard:'{h} par mois, offre classique seulement ; carte bancaire requise',
      monthSmall:'Un petit quota mensuel, usage non commercial', yours:'Selon votre service'},
    price:{one:'{a} par mois', range:'de {a} à {b} par mois', from:'dès {a} par mois', freeOr:'{z} dans l’usage gratuit ; payant : {p}', until:'{p} jusqu’au {date}',
      fee:'{p} ; frais de {f} sur le crédit', yours:'Selon votre service'}}},
ar:{cloud:'الخدمات السحابية', cloudSub:'بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف', master:'استخدام الخدمات السحابية', masterOff:'الخدمات السحابية معطّلة: لا تُعرض أي خدمة ولا يُرسَل شيء.',
  freeLine:'الاستخدام المجاني: {v}', custom:'عنوان مخصّص', customTo:'العنوان الذي تكتبه', customSub:'عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك', noKeep:'لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.',
  mOff:'معطّلة', mKey:'تحتاج إلى مفتاح', mUntested:'غير مُختبَرة', mReady:'جاهزة', mNa:'غير متاحة', mTrains:'التدريب مفعّل افتراضيًا',
  facts:'المعلومات', keyModel:'المفتاح والنموذج', swOff:'تعطيل', swOn:'تفعيل',
  cardSub:'اقرأ هذا قبل التفعيل', cardOn:'فُعِّلت في {date}', c_where:'وجهة صوتك', c_trains:'التدريب', c_keeps:'الاحتفاظ بالصوت', c_free:'الاستخدام المجاني',
  c_light:'استخدام خفيف · {h} شهريًا', c_heavy:'استخدام مكثّف · {h} شهريًا', readOn:'قُرئت هذه المعلومات في {date}.', old:'قد تكون قديمة، فراجع صفحة الخدمة.',
  link:'فتح {site}', linkName:'فتح {site} في المتصفح', confirm:'تأكيد', confirmHint:'لا يُرسَل صوتك إلى {name} إلا بعد الإيقاف، وبمفتاحك الخاص.',
  untestedLine:'غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.',
  warns:{eleven:'تستخدم ElevenLabs صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل الخيار «{sw}» في حسابك لدى ElevenLabs.'},
  keyCap:'المفتاح', keyHint:'اكتب مفتاحك أو الصقه', paste:'لصق', test:'اختبار', testing:'جارٍ الاختبار…', ok:'المفتاح يعمل', bad:'رُفض المفتاح. تحقّق منه في صفحة الخدمة.',
  remove:'إزالة المفتاح', firstTest:'لا يوجد اختبار مسبق لهذه الخدمة، فسيُختبر المفتاح عند أول إملاء.', keySealed:'يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.',
  addrCap:'العنوان', addrHint:'عنوان موردك في Azure', addrHintC:'عنوان HTTPS لخدمتك', addrBad:'يجب أن يبدأ العنوان بـ {https}',
  modelCap:'نموذج آخر', modelHint:'معرّف النموذج من صفحة الخدمة', keepNothing:'الاقتصار على مضيفين لا يحتفظون بشيء', makeKey:'أنشئ مفتاحًا في {site}',
  cloudModelSub:'في السحابة، بمفتاحك', sides:'مكان تشغيل النموذج', sideLocal:'محلية', sideCloud:'سحابية',
  noneOn:'لم تُفعَّل أي خدمة سحابية بعد.', cloudLine:'نماذج الخدمات التي فعّلتها، ولا يُرسَل شيء قبل الإيقاف.', slow:'ستكون النماذج المحلية للغة {lang} بطيئة جدًا على هذا الحاسوب.',
  onBar:'يُرسَل صوتك إلى {name} عند الإيقاف', onBarSend:'جارٍ إرسال صوتك إلى {name}',
  mo:['يناير','فبراير','مارس','أبريل','مايو','يونيو','يوليو','أغسطس','سبتمبر','أكتوبر','نوفمبر','ديسمبر'],
  fx:{where:{us:'الولايات المتحدة', eu:'الاتحاد الأوروبي', host:'المضيف الذي يختاره OpenRouter', region:'منطقة Azure التي تختارها، ومنها مناطق أوروبية', yours:'العنوان الذي تكتبه', notSaid:'غير مذكور في صفحة الخدمة'},
    trains:{no:'لا يُستخدم للتدريب', optIn:'فقط إذا وافقت على ذلك', byDefault:'يُستخدم للتدريب افتراضيًا', freeMay:'قد يُستخدم في الوضع المجاني، ويمنع ذلك خيارٌ في الحساب',
      host:'يحدده المضيف', yours:'حسب خدمتك', notSaid:'غير مذكور في صفحة الخدمة'},
    keeps:{none:'لا يُحتفظ بشيء افتراضيًا', afterProc:'يُحذف بعد المعالجة', untilSwitch:'تُحفَظ الطلبات ما دام خيارٌ في الحساب مفعّلًا', memory:'في الذاكرة فقط',
      routerNone:'لا يحتفظ OpenRouter بشيء، والقرار للمضيف', yours:'حسب خدمتك', notSaid:'غير مذكور في صفحة الخدمة'},
    free:{day:'{h} يوميًا، و{r} في الدقيقة', none:'لا يوجد', signup:'رصيد صغير عند التسجيل', firstMonth:'الشهر الأول',
      unclear:'غير واضح، وتذكر خطة Pro رصيدًا للواجهة البرمجية', unverified:'غير مؤكد', monthCard:'{h} شهريًا للنسخة الكلاسيكية فقط، وتلزم بطاقة دفع',
      monthSmall:'حصة شهرية صغيرة للاستخدام غير التجاري', yours:'حسب خدمتك'},
    price:{one:'{a} شهريًا', range:'من {a} إلى {b} شهريًا', from:'ابتداءً من {a} شهريًا', freeOr:'{z} ضمن الاستخدام المجاني، أو {p} عند الدفع', until:'{p} حتى {date}',
      fee:'{p}، مع رسوم {f} على الرصيد', yours:'حسب خدمتك'}}}
};
Object.keys(CT).forEach(k => Object.assign(L[k], CT[k]));

// Counts and money: CLDR plural forms as voice-bar.js's AR table; the decimal mark is voice-bar.js's nf.
const COUNT = {
  en:{h:n => n + (n === 1 ? ' hour' : ' hours'), r:n => n + (n === 1 ? ' request' : ' requests'), hs:n => n + ' h',
    svc:n => n === 0 ? 'No service ready' : n + (n === 1 ? ' service ready' : ' services ready')},
  fr:{h:n => n + (n > 1 ? ' heures' : ' heure'), r:n => n + (n > 1 ? ' requêtes' : ' requête'), hs:n => n + ' h',
    svc:n => n === 0 ? 'Aucun service prêt' : n + (n > 1 ? ' services prêts' : ' service prêt')},
  ar:{h:n => arP(n, AR.hours), r:n => arP(n, ['طلب واحد','طلبان','{n} طلبات','{n} طلبًا','{n} طلب']), hs:n => arP(n, AR.hours),
    svc:n => n === 0 ? 'لا توجد خدمة جاهزة' : arP(n, ['خدمة واحدة جاهزة','خدمتان جاهزتان','{n} خدمات جاهزة','{n} خدمةً جاهزةً','{n} خدمة جاهزة'])}};
AR.hours = ['ساعة واحدة','ساعتان','{n} ساعات','{n} ساعة','{n} ساعة'];
const cn = () => COUNT[S.lang];
const CUR = {usd:{en:a => '$' + a, fr:a => a + ' $', ar:a => a + ' دولار'}};
const money = (v, cur) => CUR[cur][S.lang](Number.isInteger(v) ? String(v) : nf(v, 2));
const pct = v => nf(v) + (S.lang === 'fr' ? ' %' : '%');
const dateOf = iso => { const [y, m, d] = iso.split('-').map(Number); return d + ' ' + l.mo[m - 1] + ' ' + y; };

// The day the owner checks the mock-up and settings [voice] facts_days: a card read longer ago shows its date as old.
const TODAY = '2026-11-17', FACTS_DAYS = 180;
const daysTo = iso => Math.round((Date.parse(TODAY) - Date.parse(iso)) / 864e5);
// Light and heavy use in hours a month (spec "The services table"); a sample key's length; the samples typed on the lines.
const USE_H = {light:15, heavy:60}, KEY_LEN = 56, READ = '2026-11-10', OLD_READ = '2026-04-02';
const AZURE = 'https://westeurope.api.cognitive.microsoft.com', OR_MODEL = 'mistralai/voxtral-small-24b-2507', LAN = 'http://192.168.1.20:8000/v1/audio/transcriptions';

// The services table's rows, Groq first, with the spec's facts read on 2026-11-10. site is the facts' page, keys where a
// key is made; p is one price or a range, free marks a free use that covers it; test says a cheap key check exists;
// untested marks a connection not yet checked with a real key; sw is a sample account switch's name, read later.
const SVC = [
  {id:'groq', name:'Groq', site:'groq.com/pricing', keys:'console.groq.com/keys', test:true, where:'us', trains:'no', keeps:'none',
    free:['day', {h:8, r:20}], cur:'usd', light:{free:true, p:[0.60, 1.67]}, heavy:{free:true, p:[2.40, 6.66]}},
  {id:'openai', name:'OpenAI', site:'openai.com/api/pricing', keys:'platform.openai.com/api-keys', test:true, where:'us', trains:'optIn', keeps:'notSaid',
    free:['none'], cur:'usd', light:{p:[4.05]}, heavy:{p:[16.20]}},
  {id:'mistral', name:'Mistral', site:'mistral.ai/pricing', keys:'console.mistral.ai', test:true, untested:true, where:'eu', trains:'freeMay', keeps:'notSaid',
    free:['unclear'], cur:'usd', light:{p:[3.15]}, heavy:{p:[12.60]}},
  {id:'together', name:'Together', site:'together.ai/pricing', keys:'api.together.ai', test:true, where:'us', trains:'notSaid', keeps:'untilSwitch',
    free:['none'], cur:'usd', light:{p:[1.35]}, heavy:{p:[5.40]}},
  {id:'lemonfox', name:'Lemonfox', site:'lemonfox.ai', keys:'lemonfox.ai', test:true, where:'notSaid', trains:'notSaid', keeps:'afterProc',
    free:['firstMonth'], cur:'usd', light:{p:[5]}, heavy:{p:[10]}},
  {id:'custom', test:false, addr:true, where:'yours', trains:'yours', keeps:'yours', free:['yours'], light:'yours', heavy:'yours'},
  {id:'openrouter', name:'OpenRouter', site:'openrouter.ai', keys:'openrouter.ai/keys', test:true, untested:true, keep0:true, where:'host', trains:'host', keeps:'routerNone',
    free:['signup'], cur:'usd', light:{from:0.18, fee:5.5}, heavy:{from:0.72, fee:5.5}},
  {id:'deepinfra', name:'DeepInfra', site:'deepinfra.com/pricing', keys:'deepinfra.com', test:true, untested:true, where:'notSaid', trains:'no', keeps:'memory',
    free:['unverified'], cur:'usd', light:{from:0.18}, heavy:{from:0.72}},
  {id:'azure', name:'Azure AI Speech', site:'azure.microsoft.com', keys:'portal.azure.com', test:true, addr:true, where:'region', trains:'notSaid', keeps:'none',
    free:['monthCard', {h:5}], cur:'usd', light:{p:[5.40]}, heavy:{p:[21.60]}},
  {id:'mai', name:'MAI-Transcribe-2', site:'azure.microsoft.com', keys:'portal.azure.com', test:true, untested:true, addr:true, where:'region', trains:'notSaid', keeps:'none',
    free:['none'], cur:'usd', light:{p:[1.50], until:'2026-12-31'}, heavy:{p:[6.00], until:'2026-12-31'}},
  {id:'eleven', name:'ElevenLabs', site:'elevenlabs.io/pricing', keys:'elevenlabs.io', test:true, untested:true, warn:'eleven', sw:'Use my data for training',
    where:'notSaid', trains:'byDefault', keeps:'notSaid', free:['monthSmall'], cur:'usd', light:{p:[3.30]}, heavy:{p:[13.20]}}];
// A row the table does not offer (Gemini, until its check passes) never shows, so the sample leaves it out.
const byId = id => SVC.find(s => s.id === id);
const svcName = s => s.id === 'custom' ? l.custom : s.name;
// Cloud models for every language of the keyboard, shown while their service is ready: Groq's two, the big Nemotron
// through DeepInfra (spec), and the "Other model" id typed for OpenRouter.
const CM = {'gq-turbo':['Whisper Large v3 Turbo', 'groq'], 'gq-large':['Whisper Large v3', 'groq'], 'di-nemo':['Nemotron 3.5 Transcribe', 'deepinfra'],
  'or-typed':[OR_MODEL, 'openrouter']};
Object.keys(CM).forEach(id => { M[id] = [CM[id][0], 0]; });

// Sample state: the user already switched on Groq (tested), DeepInfra (a key, not yet tested) and OpenRouter (no key yet).
S.cloud = true;
S.svc = Object.fromEntries(SVC.map(s => [s.id, {st:'off', key:0, test:'', will:'ok', on:'', keep0:true}]));
Object.assign(S.svc.groq, {st:'ready', key:KEY_LEN, test:'ok', on:READ});
Object.assign(S.svc.openrouter, {st:'key', on:'2026-11-12'});
Object.assign(S.svc.deepinfra, {st:'ready', key:48, on:'2026-11-12'});
S.typed = {addr:{custom:LAN}, model:{}}; S.aim = ''; S.card = {id:'groq', ask:true, read:''}; S.cur = 'groq'; S.ret = 'cloud';
if (['svccard','svcpage'].includes(S.panel)) S.panel = 'cloud';

// A card's sentences, from the row's flags and numbers.
function freeText(s) { const [k, o = {}] = s.free; return t(l.fx.free[k], {h:cn().h(o.h), r:cn().r(o.r)}); }
function price(p, cur) {
  const x = l.fx.price, m = v => money(v, cur);
  if (p === 'yours') return x.yours;
  let s = p.from !== undefined ? t(x.from, {a:m(p.from)}) : p.p.length > 1 ? t(x.range, {a:m(p.p[0]), b:m(p.p[1])}) : t(x.one, {a:m(p.p[0])});
  if (p.free) s = t(x.freeOr, {z:m(0), p:s});
  if (p.until) s = t(x.until, {p:s, date:dateOf(p.until)});
  if (p.fee) s = t(x.fee, {p:s, f:pct(p.fee)});
  return s;
}
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
function useEngine(v) {
  if (v === 'cloud') { S.cloud = true; Object.assign(S.svc.groq, {st:'ready', key:KEY_LEN, test:'ok', on:S.svc.groq.on || READ}); }
  const want = S.models.find(m => m.id === (v === 'cloud' ? 'gq-turbo' : S.models[0].id));
  S.models.forEach(m => { if (m.st === 'use') m.st = 'ready'; });
  if (want.st === 'ready') want.st = 'use';
}
function addCloud() {
  S.models.splice(1, 0, ...Object.keys(CM).map(id => ({id, st:'ready', def:false, p:0, paused:false, cloud:CM[id][1]})));
  if (S.engine === 'cloud') useEngine('cloud');
}

// The bar's cloud mark, the home row's value, the Models page's Local | Cloud chips, a cloud model's row and its bands.
function cloudChip(kind) {
  const u = inUse();
  if (!u.cloud) return '';
  const name = byId(u.cloud).name;
  return '<span class="lc cm" role="img" aria-label="' + t(kind === 'send' ? l.onBarSend : l.onBar, {name}) + '">' + ic('cloud') + '<span>' + name + '</span></span>';
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

// The Cloud page: the master switch, then the services, five a page; a ticked row's acts in the footer.
function cloudPage() {
  const on = S.cloud, s = S.tick.length ? SVC[S.tick[0]] : null, st = s ? S.svc[s.id].st : '', live = on && !!s;
  const rows = SVC.map((v, i) => {
    const sub = v.id === 'custom' ? l.customSub : t(l.freeLine, {v:freeText(v)});
    const inner = tickBox(i, true) + cell(svcName(v) + untestedTag(v), sub) + '<div class="marks">' + (v.warn ? mark(l.mTrains, 'triangle-alert', 'warn') : '') + stateMark(v.id) + '</div>';
    return on ? tickRow([CK, '1fr', 'auto'], inner, i) : row([CK, '1fr', 'auto'], inner, '', 'flat off');
  });
  const band = '<div class="pn swb' + A + '" data-a="cloudsw" role="switch" aria-checked="' + on + '" tabindex="0">' + ic('cloud') + '<span class="x"' + D + '>' + l.master + '</span>' +
    '<span class="onoff' + A + '"' + D + '>' + (on ? l.on : l.off) + '</span><span class="sw' + (on ? ' on' : '') + '"></span></div>';
  return {icon:'cloud', title:l.cloud, sub:l.cloudSub, top:band + (on ? '' : note('power', l.masterOff)), body:list(rows),
    foot:btn('facts', 'file', l.facts, '', !live) + btn('svc', 'key', l.keyModel, '', !live || st === 'off') + btn('svcoff', 'power', l.swOff, '', !live || st === 'off') + gap +
      btn('swon', 'power', l.swOn, 'enter', !live || st !== 'off')};
}

// A service's card: its facts with the date read, a link to its page, a warning band when it trains by default.
function cardPage() {
  const s = byId(S.card.id), v = S.svc[s.id], ask = S.card.ask, read = S.card.read || READ, old = daysTo(read) > FACTS_DAYS, fx = l.fx;
  const rows = [['where', fx.where[s.where]], ['trains', fx.trains[s.trains], ['byDefault', 'freeMay'].includes(s.trains)], ['keeps', fx.keeps[s.keeps]], ['free', freeText(s)],
    ['light', price(s.light, s.cur), false, USE_H.light], ['heavy', price(s.heavy, s.cur), false, USE_H.heavy]];
  const facts = '<div class="facts' + A + '"' + D + '>' + rows.map(([k, val, warm, h]) => '<span class="fk">' + t(l['c_' + k], {h:cn().hs(h)}) + '</span>' +
    '<span class="fv' + (warm ? ' warn' : '') + '">' + (warm ? ic('triangle-alert') : '') + '<span>' + val + '</span></span>').join('') + '</div>';
  const host = s.site ? s.site.split('/')[0] : '', name = s.id === 'custom' ? l.customTo : s.name;
  const when = !host ? '' : '<div class="ro' + (old ? ' old' : '') + A + '">' + ic(old ? 'triangle-alert' : 'history') + '<span class="x"' + D + '>' + t(l.readOn, {date:dateOf(read)}) +
    (old ? ' ' + l.old : '') + '</span>' + btn('link', 'external-link', t(l.link, {site:bdi(host)}), '', false, t(l.linkName, {site:s.site})) + '</div>';
  const untested = s.untested ? '<div class="ro' + A + '">' + ic('activity') + '<span class="x"' + D + '>' + l.untestedLine + '</span></div>' : '';
  const warn = s.warn ? '<div class="pn warn' + A + '">' + ic('triangle-alert') + '<span class="x"' + D + '>' + t(l.warns[s.warn], {sw:bdi(s.sw)}) + '</span></div>' : '';
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
    (s.addr ? lrow(pline('addr', l.addrCap, addr, s.id === 'custom' ? l.addrHintC : l.addrHint)) : '') +
    (addr && !addr.startsWith('https://') ? '<div class="res bad' + A + '">' + ic('triangle-alert') + '<span' + D + '>' + t(l.addrBad, {https:bdi('https://')}) + '</span></div>' : '') +
    lrow(pline('model', l.modelCap, typed, l.modelHint)) + na +
    (s.keep0 ? '<div class="swr" data-a="keep0" role="switch" aria-checked="' + v.keep0 + '" tabindex="0"><span class="x' + A + '"' + D + '>' + l.keepNothing + '</span>' +
      '<span class="onoff' + A + '"' + D + '>' + (v.keep0 ? l.on : l.off) + '</span><span class="sw' + (v.keep0 ? ' on' : '') + '"></span></div>' : '') +
    (s.keys ? '<div class="lrow ko' + A + '">' + ic('key') + '<span class="x"' + D + '>' + t(l.makeKey, {site:bdi(s.keys)}) + '</span>' + btn('link', 'external-link', l.open, '', false, t(l.linkName, {site:s.keys})) + '</div>' : '');
  return {icon:'key', title:svcName(s), sub:l[MARK[v.st][0]], body:'<div class="pb"><div class="lines">' + lines + '</div></div>',
    foot:btn('facts', 'file', l.facts) + btn('rmkey', 'trash-2', l.remove, 'danger', !v.key) + '<span class="x' + A + '"' + D + '>' + l.keySealed + '</span>' + btn('svcoff', 'power', l.swOff)};
}
Object.assign(PAGE, {cloud:cloudPage, svccard:cardPage, card:cardPage, cardwarn:cardPage, cardold:cardPage, svcpage:svcPage, key:svcPage, keybad:svcPage, address:svcPage, model:svcPage,
  slow:() => PAGE.models(), mcloud:cloudModels, paste:() => PAGE.options()});

// Navigation: the controls' presets, and Back from a card or a service's page to where it was opened.
function openCard(id, ask, ret, read) { S.card = {id, ask, read:read || ''}; S.ret = ret; }
function openSvc(id) { S.cur = id; S.ret = 'cloud'; }
const PRESET = {
  models:() => { S.weak = false; }, slow:() => { S.weak = true; }, paste:() => { S.opt.insert = 1; S.pg = Math.floor(OPTS.findIndex(o => o[0] === 'insert') / 5); },
  card:() => openCard('groq', true, 'cloud'), cardwarn:() => openCard('eleven', true, 'cloud'), cardold:() => openCard('groq', true, 'cloud', OLD_READ),
  key:() => { openSvc('groq'); Object.assign(S.svc.groq, {st:'ready', key:KEY_LEN, test:'ok', will:'ok', on:S.svc.groq.on || READ}); },
  keybad:() => { openSvc('groq'); Object.assign(S.svc.groq, {st:'key', key:KEY_LEN, test:'bad', will:'bad', on:S.svc.groq.on || READ}); },
  address:() => { openSvc('azure'); if (S.svc.azure.st === 'off') Object.assign(S.svc.azure, {st:'key', on:TODAY}); S.typed.addr.azure = AZURE; S.aim = 'addr'; },
  model:() => { openSvc('openrouter'); Object.assign(S.svc.openrouter, {st:'ready', key:KEY_LEN, test:'', will:'ok', keep0:true, on:S.svc.openrouter.on || TODAY});
    S.typed.model.openrouter = OR_MODEL; S.aim = 'model'; }};
function cloudGo(p) { S.aim = ''; if (PRESET[p]) PRESET[p](); if (p === 'svcpage') S.ret = 'cloud'; fallback(); }
const SUB = ['card','cardwarn','cardold','key','keybad','address','model','svccard','svcpage'];
const tickedSvc = () => SVC[S.tick[0]];
const curSvc = () => S.panel === 'cloud' ? tickedSvc() : byId(S.cur);
let ctimer = 0;
const backBase = ACT.back, useBase = ACT.use;
Object.assign(ACT, {
  back:() => SUB.includes(S.panel) ? go(S.ret || 'cloud') : backBase(),
  use:() => { useBase(); S.engine = inUse().cloud ? 'cloud' : 'local'; },
  cloudsw:() => { S.cloud = !S.cloud; S.tick = []; fallback(); },
  swon:() => { openCard(tickedSvc().id, true, 'cloud'); go('svccard'); },
  facts:() => { const s = curSvc(); openCard(s.id, S.svc[s.id].st === 'off', S.panel); go('svccard'); },
  cardno:() => go(S.ret),
  confirm:() => { const v = S.svc[S.card.id]; Object.assign(v, {st:v.key ? 'ready' : 'key', on:TODAY}); openSvc(S.card.id); go('svcpage'); },
  svc:() => { openSvc(tickedSvc().id); go('svcpage'); },
  svcoff:() => { Object.assign(S.svc[curSvc().id], {st:'off', key:0, test:''}); fallback(); if (S.panel !== 'cloud') go('cloud'); },
  paste:() => { Object.assign(S.svc[S.cur], {key:KEY_LEN, test:'', will:'ok', st:'ready'}); S.aim = ''; fallback(); },
  test:() => {
    const v = S.svc[S.cur]; v.test = 'testing'; clearTimeout(ctimer);
    ctimer = setTimeout(() => { const ok = v.will === 'ok'; Object.assign(v, {test:ok ? 'ok' : 'bad', st:ok ? 'ready' : 'key'}); fallback(); render(); }, PACE.test);
  },
  rmkey:() => { Object.assign(S.svc[S.cur], {key:0, test:'', st:'key'}); fallback(); },
  side:x => go(x === 'cloud' ? 'mcloud' : S.weak ? 'slow' : 'models'),
  aim:x => { S.aim = S.aim === x ? '' : x; }, keep0:() => { const v = S.svc[S.cur]; v.keep0 = !v.keep0; fallback(); }, link:() => {}});

// The controls on top, in English as on the other mock-ups, and what each new state shows.
CTL.cloud = [['cloud','Cloud page'],['card','Card: Groq'],['cardwarn','Card: ElevenLabs'],['cardold','Card: old facts'],['key','Key works'],['keybad','Key refused'],
  ['address','Azure address'],['model','Other model'],['mcloud','Models: Cloud'],['slow','Models, slow PC'],['paste','Options: Paste']];
CTL.engine = [['local','Local model'],['cloud','Groq (cloud)']];
SAY.home += ' Part 2 adds Cloud services as the second row, so Options moves to page 2.';
SAY.options += ' Part 2: Insert the text, Type it or Paste at once, as two filter chips (page 3).';
SAY.models += ' Part 2: Local | Cloud at the top, as the clipboard panel’s filter chips; this Local side lists only the models on this PC.';
Object.assign(SAY, {
  preparing:'<strong>Preparing.</strong> No bar yet: from the Dictation click until the microphone is truly open, only the Dictation key glows amber, as a studio’s amber stand-by light comes before the red on-air light. Narrator hears “{name}”. Click the amber key to cancel with nothing sent; otherwise the bar opens straight in the red Listening state. Click the Dictation key, or Listen again, to start it.',
  cloudbar:' <strong>Cloud mark:</strong> a cloud and the service’s name beside the language, so you always know where the voice goes; nothing leaves before Stop.',
  cloud:' <strong>Cloud services.</strong> The master switch, then the services five a page, Groq first; tick one, then act in the footer (the clipboard panel’s pattern). Marks: Off, Needs a key, Ready (a key set but not yet tested is Ready); the Untested tag beside a name marks a connection never checked with a real key. A row the table does not offer never shows. The sample user already switched on Groq, DeepInfra and OpenRouter; in the product the switch starts off.',
  card:' <strong>Service card, before switching on.</strong> Fixed rows, as an app store’s privacy label: where the voice goes, training, what is kept, free use, light and heavy use, the date read and a link to the service’s page.',
  cardwarn:' <strong>Card with a warning.</strong> ElevenLabs trains on the voice by default, so a band in the warning colours (Windows 11 InfoBar) names the account switch; that name is a sample until its page is read.',
  cardold:' <strong>Old facts.</strong> Read ' + daysTo(OLD_READ) + ' days before the check, past facts_days (' + FACTS_DAYS + '), so the date takes the warning colour and asks to check the service’s page.',
  svccard:' <strong>Service card.</strong> The facts with their date and a link to the service’s page.',
  key:' <strong>Key line.</strong> The panel’s own line, as Quick-fill’s: the key as dots with Paste and Test, Remove key in the footer. Test said Key works.',
  keybad:' <strong>Key refused.</strong> Test answered no, in the warning colours; Paste a new key, then Test again.',
  address:' <strong>Address line.</strong> Azure’s resource address taking the letters: ring, caret and accent line (WinUI’s focused TextBox).',
  model:' <strong>Model line.</strong> OpenRouter’s Other model taking a model id. With its keep-nothing switch on, a model with no such host is Unavailable, with the reason, here and on the Models page; switch it off and the model is Ready.',
  svcpage:' <strong>Key and model.</strong> The key line, Test, the model line and where to make a key; Custom also has its address line, which must be https.',
  slow:' <strong>Models on a weak PC.</strong> The Local side: the default is Not for this PC, so the band says local models will be very slow here and leads to Cloud services.',
  paste:' <strong>Insert the text: Paste at once.</strong> Two filter chips, Type it the default; while Paste is chosen, its one-line hint takes the row below.',
  mcloud:' <strong>Models, Cloud side.</strong> Its own list, five a page: only the models of the services you switched on, each with the cloud mark and its service; one still needing a key says so and Use stays dimmed. The band leads to Cloud services.'});

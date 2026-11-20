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

// Texts, as built in crates/kx-ui/i18n/cloud/ (2026-11-20). fx holds a card's fact sentences by group and the services
// table's fact ids, filled with the row's numbers so no price or claim is written twice.
const CT = {
en:{cloud:'Cloud services', cloudSub:'Your own key · nothing leaves before Stop', master:'Use cloud services', masterOff:'Cloud services are off: none is offered and nothing is sent.',
  freeLine:'Free: {v}', custom:'Custom address', customTo:'the address you type', customSub:'An HTTPS address, a model and a key you type', noKeep:'No host of this model promises to keep nothing.',
  mOff:'Off', mKey:'Needs a key', mUntested:'Untested', mReady:'Ready', mNa:'Unavailable', mTrains:'Trains by default', mFree:'Trains on the free allowance',
  facts:'Facts', keyModel:'Key and model', swOff:'Switch off', swOn:'Switch on',
  cardSub:'Read this before you switch it on', cardOn:'Switched on {date}', c_where:'Where your voice goes', c_trains:'Training', c_keeps:'What is kept', c_free:'Free allowance',
  c_light:'Light use · {h}', c_heavy:'Heavy use · {h}', c_bills:'Least billed', c_rule:'Terms of use', readOn:'Facts read on {date}.', old:'They may be out of date; check the service’s page.',
  link:'Open {site}', linkName:'Open {site} in the browser', confirm:'Confirm', confirmHint:'Your voice goes to {name} only after Stop, with your own key.',
  untestedLine:'Untested: not yet checked with a real key.',
  warns:{trainsProfile:'Trains its models on your voice by default. To stop it, turn off this switch in your profile menu: {sw}',
    freeAllowance:'While you use the free allowance, Google uses your voice to improve its products, and human reviewers may read what you say.'},
  keyCap:'Key', keyHint:'Type or paste your key', paste:'Paste', test:'Test', testing:'Testing…', ok:'Key works', bad:'Key refused. Check it on the service’s page.',
  remove:'Remove key', firstTest:'No test call here: the first dictation tests the key.', keySealed:'The key is kept encrypted on this PC and never shown.',
  addrCap:'Address', addrHint:'Your service’s address, such as {ex}', addrBad:'The address must use HTTPS, such as {ex}',
  modelCap:'Other model', modelHint:'A model id from the service’s page', keepNothing:'Only hosts that keep nothing', makeKey:'Make a key at {site}',
  cloudModelSub:'In the cloud, with your key', sides:'Where the model runs', sideLocal:'Local', sideCloud:'Cloud',
  noneOn:'No cloud service is switched on yet.', cloudLine:'Models of the services you switched on; nothing leaves before Stop.', slow:'Local models for {lang} will be very slow on this PC.',
  onBar:'Your voice goes to {name} at Stop', onBarSend:'Sending your voice to {name}',
  mo:['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'],
  fx:{where:{us:'United States', eu:'European Union', typed:'The address you type', host:'Depends on the host that serves the model', unstated:'Not stated on its pages'},
    trains:{never:'Not used for training', unknown:'Unknown; it depends on the service', unstated:'Not stated on its pages', host:'Depends on the host that serves the model',
      default:'Used for training by default', freeAllowance:'Used for training while you use the free allowance'},
    keeps:{logs:'Nothing by default, but checks may keep requests up to {d}; this account switch turns that off: {sw}', unknown:'Unknown; it depends on the service',
      deleted:'Nothing: deleted right after processing', host:'Depends on the host; a switch, on to start, sends only to hosts that keep nothing',
      history:'Your history, by default; keeping nothing is for Enterprise plans only', paidLimited:'On the paid tier: your requests, for a limited time to check for abuse',
      unstated:'Not stated on its pages'},
    free:{daily:'{h} a day, {r} a minute', unknown:'Unknown; it depends on the service', firstMonth:'The first month', small:'A small allowance for new accounts',
      minutesMonth:'About {m} a month', noCard:'No card needed', credit:'{c} of credit that never expires, no card needed'},
    price:{freeOr:'Free within the free allowance, else {lo} to {hi} by model', unknown:'Unknown; it depends on the service', plain:'{p}', paidPlain:'{p} on the paid tier',
      rangeFee:'{lo} to {hi} by model, plus a {fee}% fee on credit'},
    bills:{least:'At least {s} a request, so short pieces cost more'},
    rule:{nonCommercial:'The free plan is for non-commercial use only, with attribution', eea:'Paid use only in the EEA, Switzerland and the UK; users must be 18 or older'}}},
fr:{cloud:'Services cloud', cloudSub:'Votre propre clé · rien ne part avant l’arrêt', master:'Utiliser les services cloud', masterOff:'Services cloud désactivés : aucun n’est proposé, rien n’est envoyé.',
  freeLine:'Gratuit : {v}', custom:'Adresse personnalisée', customTo:'l’adresse que vous saisissez', customSub:'Une adresse HTTPS, un modèle et une clé à saisir', noKeep:'Aucun hébergeur ne s’engage à ne rien garder.',
  mOff:'Désactivé', mKey:'Clé requise', mUntested:'Non testé', mReady:'Prêt', mNa:'Indisponible', mTrains:'Entraînement activé', mFree:'Entraînement avec le quota gratuit',
  facts:'Informations', keyModel:'Clé et modèle', swOff:'Désactiver', swOn:'Activer',
  cardSub:'À lire avant d’activer', cardOn:'Activé le {date}', c_where:'Où va votre voix', c_trains:'Entraînement', c_keeps:'Conservation', c_free:'Quota gratuit',
  c_light:'Usage léger · {h}', c_heavy:'Usage intensif · {h}', c_bills:'Facturation minimale', c_rule:'Conditions d’utilisation', readOn:'Informations relevées le {date}.', old:'Elles datent peut-être ; vérifiez la page du service.',
  link:'Ouvrir {site}', linkName:'Ouvrir {site} dans le navigateur', confirm:'Confirmer', confirmHint:'Votre voix ne part vers {name} qu’après l’arrêt, avec votre propre clé.',
  untestedLine:'Non testé : pas encore vérifié avec une vraie clé.',
  warns:{trainsProfile:'Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage dans le menu de votre profil : {sw}',
    freeAllowance:'Tant que vous utilisez le quota gratuit, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites.'},
  keyCap:'Clé', keyHint:'Tapez ou collez votre clé', paste:'Coller', test:'Tester', testing:'Test en cours…', ok:'La clé fonctionne', bad:'Clé refusée. Vérifiez-la sur la page du service.',
  remove:'Retirer la clé', firstTest:'Pas de test possible ici : la première dictée testera la clé.', keySealed:'La clé est chiffrée sur ce PC et jamais affichée.',
  addrCap:'Adresse', addrHint:'L’adresse de votre service, par exemple {ex}', addrBad:'L’adresse doit utiliser HTTPS, par exemple {ex}',
  modelCap:'Autre modèle', modelHint:'L’identifiant du modèle, sur la page du service', keepNothing:'Seulement des hébergeurs qui ne gardent rien', makeKey:'Créer une clé sur {site}',
  cloudModelSub:'Dans le cloud, avec votre clé', sides:'Où le modèle s’exécute', sideLocal:'Sur ce PC', sideCloud:'Dans le cloud',
  noneOn:'Aucun service cloud n’est encore activé.', cloudLine:'Modèles des services que vous avez activés ; rien ne part avant l’arrêt.', slow:'Les modèles locaux en {lang} seront très lents sur ce PC.', langIn:{en:'anglais', fr:'français', ar:'arabe'},
  onBar:'Votre voix partira chez {name} à l’arrêt', onBarSend:'Envoi de votre voix à {name}',
  mo:['janv.','févr.','mars','avr.','mai','juin','juil.','août','sept.','oct.','nov.','déc.'],
  fx:{where:{us:'États-Unis', eu:'Union européenne', typed:'L’adresse que vous saisissez', host:'Dépend de l’hébergeur qui sert le modèle', unstated:'Non précisé sur ses pages'},
    trains:{never:'Non utilisée pour l’entraînement', unknown:'Inconnu ; cela dépend du service', unstated:'Non précisé sur ses pages', host:'Dépend de l’hébergeur qui sert le modèle',
      default:'Utilisée pour l’entraînement par défaut', freeAllowance:'Utilisée pour l’entraînement tant que vous utilisez le quota gratuit'},
    keeps:{logs:'Rien par défaut, mais les requêtes peuvent être gardées jusqu’à {d} pour contrôle ; ce réglage du compte l’empêche : {sw}', unknown:'Inconnu ; cela dépend du service',
      deleted:'Rien : tout est supprimé juste après le traitement', host:'Dépend de l’hébergeur ; un réglage, activé d’emblée, n’envoie qu’aux hébergeurs qui ne gardent rien',
      history:'Votre historique, par défaut ; ne rien garder est réservé aux offres Enterprise', paidLimited:'Au niveau payant : vos requêtes, pendant un temps limité pour détecter les abus',
      unstated:'Non précisé sur ses pages'},
    free:{daily:'{h} par jour, {r} par minute', unknown:'Inconnu ; cela dépend du service', firstMonth:'Le premier mois', small:'Un petit quota pour les nouveaux comptes',
      minutesMonth:'Environ {m} par mois', noCard:'Sans carte bancaire', credit:'{c} de crédit sans expiration, sans carte'},
    price:{freeOr:'Gratuit dans la limite du quota gratuit, sinon de {lo} à {hi} selon le modèle', unknown:'Inconnu ; cela dépend du service', plain:'{p}', paidPlain:'{p} au niveau payant',
      rangeFee:'De {lo} à {hi} selon le modèle, plus {fee} % de frais sur le crédit'},
    bills:{least:'Au moins {s} par requête ; les morceaux courts coûtent donc plus cher'},
    rule:{nonCommercial:'L’offre gratuite est réservée à un usage non commercial, avec mention de la source', eea:'Usage payant seulement dans l’EEE, en Suisse et au Royaume-Uni ; utilisateurs de 18 ans ou plus'}}},
ar:{cloud:'الخدمات السحابية', cloudSub:'بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف', master:'استخدام الخدمات السحابية', masterOff:'الخدمات السحابية معطّلة: لا تُعرض أي خدمة ولا يُرسَل شيء.',
  freeLine:'مجانًا: {v}', custom:'عنوان مخصّص', customTo:'العنوان الذي تكتبه', customSub:'عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك', noKeep:'لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.',
  mOff:'معطّلة', mKey:'تحتاج إلى مفتاح', mUntested:'غير مُختبَرة', mReady:'جاهزة', mNa:'غير متاحة', mTrains:'التدريب مفعّل افتراضيًا', mFree:'التدريب ضمن الحصة المجانية',
  facts:'المعلومات', keyModel:'المفتاح والنموذج', swOff:'تعطيل', swOn:'تفعيل',
  cardSub:'اقرأ هذا قبل التفعيل', cardOn:'فُعِّلت في {date}', c_where:'وجهة صوتك', c_trains:'التدريب', c_keeps:'ما يُحتفَظ به', c_free:'الحصة المجانية',
  c_light:'استخدام خفيف · {h}', c_heavy:'استخدام مكثّف · {h}', c_bills:'الحد الأدنى للفوترة', c_rule:'شروط الاستخدام', readOn:'قُرئت هذه المعلومات في {date}.', old:'قد تكون قديمة، فراجع صفحة الخدمة.',
  link:'فتح {site}', linkName:'فتح {site} في المتصفح', confirm:'تأكيد', confirmHint:'لا يُرسَل صوتك إلى {name} إلا بعد الإيقاف، وبمفتاحك الخاص.',
  untestedLine:'غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.',
  warns:{trainsProfile:'تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: {sw}',
    freeAllowance:'ما دمت ضمن الحصة المجانية، تستخدم Google صوتك لتحسين منتجاتها، وقد يطّلع مراجعون بشريون على ما تقوله.'},
  keyCap:'المفتاح', keyHint:'اكتب مفتاحك أو الصقه', paste:'لصق', test:'اختبار', testing:'جارٍ الاختبار…', ok:'المفتاح يعمل', bad:'رُفض المفتاح. تحقّق منه في صفحة الخدمة.',
  remove:'إزالة المفتاح', firstTest:'لا يوجد اختبار مسبق لهذه الخدمة، فسيُختبر المفتاح عند أول إملاء.', keySealed:'يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.',
  addrCap:'العنوان', addrHint:'عنوان خدمتك، مثل {ex}', addrBad:'يجب أن يستخدم العنوان HTTPS، مثل {ex}',
  modelCap:'نموذج آخر', modelHint:'معرّف النموذج من صفحة الخدمة', keepNothing:'الاقتصار على مضيفين لا يحتفظون بشيء', makeKey:'إنشاء مفتاح في {site}',
  cloudModelSub:'في السحابة، بمفتاحك', sides:'مكان تشغيل النموذج', sideLocal:'محلية', sideCloud:'سحابية',
  noneOn:'لم تُفعَّل أي خدمة سحابية بعد.', cloudLine:'نماذج الخدمات التي فعّلتها، ولا يُرسَل شيء قبل الإيقاف.', slow:'ستكون النماذج المحلية للغة {lang} بطيئة جدًا على هذا الحاسوب.',
  onBar:'يُرسَل صوتك إلى {name} عند الإيقاف', onBarSend:'جارٍ إرسال صوتك إلى {name}',
  mo:['يناير','فبراير','مارس','أبريل','مايو','يونيو','يوليو','أغسطس','سبتمبر','أكتوبر','نوفمبر','ديسمبر'],
  fx:{where:{us:'الولايات المتحدة', eu:'الاتحاد الأوروبي', typed:'العنوان الذي تكتبه', host:'يتوقف على المضيف الذي يشغّل النموذج', unstated:'غير مذكور في صفحات الخدمة'},
    trains:{never:'لا يُستخدم للتدريب', unknown:'غير معروف، ويتوقف على الخدمة', unstated:'غير مذكور في صفحات الخدمة', host:'يتوقف على المضيف الذي يشغّل النموذج',
      default:'يُستخدم للتدريب افتراضيًا', freeAllowance:'يُستخدم للتدريب ما دمت ضمن الحصة المجانية'},
    keeps:{logs:'لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى {d}، ويمكن إيقاف ذلك من هذا الخيار في حسابك: {sw}', unknown:'غير معروف، ويتوقف على الخدمة',
      deleted:'لا شيء: يُحذف كل شيء فور المعالجة', host:'يتوقف على المضيف، وهناك خيار مفعّل من البداية لا يرسل إلا إلى المضيفين الذين لا يحتفظون بشيء',
      history:'سجلّك افتراضيًا، وعدم الاحتفاظ بأي شيء متاح لخطط المؤسسات فقط', paidLimited:'في المستوى المدفوع: طلباتك لمدة محدودة لكشف إساءة الاستخدام',
      unstated:'غير مذكور في صفحات الخدمة'},
    free:{daily:'{h} يوميًا، و{r} في الدقيقة', unknown:'غير معروف، ويتوقف على الخدمة', firstMonth:'الشهر الأول', small:'حصة صغيرة للحسابات الجديدة',
      minutesMonth:'نحو {m} شهريًا', noCard:'دون الحاجة إلى بطاقة دفع', credit:'رصيد دائم بقيمة {c}، دون بطاقة دفع'},
    price:{freeOr:'مجاني ضمن الحصة المجانية، وإلا فمن {lo} إلى {hi} حسب النموذج', unknown:'غير معروف، ويتوقف على الخدمة', plain:'{p}', paidPlain:'{p} في المستوى المدفوع',
      rangeFee:'من {lo} إلى {hi} حسب النموذج، مع رسوم بنسبة {fee}% عند شراء الرصيد'},
    bills:{least:'يُحتسَب لكل طلب ما لا يقل عن {s}، فتكلّف المقاطع القصيرة أكثر'},
    rule:{nonCommercial:'الخطة المجانية للاستخدام غير التجاري فقط، مع نسب المحتوى إلى الخدمة',
      eea:'الاستخدام المدفوع فقط في المنطقة الاقتصادية الأوروبية وسويسرا والمملكة المتحدة، ويجب ألا يقل عمر المستخدم عن 18 عامًا'}}}
};
Object.keys(CT).forEach(k => Object.assign(L[k], CT[k]));

// Counts and money: CLDR plural forms as voice-bar.js's AR table; the decimal mark is voice-bar.js's nf.
const COUNT = {
  en:{h:n => n + (n === 1 ? ' hour' : ' hours'), r:n => n + (n === 1 ? ' request' : ' requests'), m:n => n + (n === 1 ? ' minute' : ' minutes'),
    d:n => n + (n === 1 ? ' day' : ' days'), s:n => n + (n === 1 ? ' second' : ' seconds'), hm:n => n + (n === 1 ? ' hour a month' : ' hours a month'),
    svc:n => n === 0 ? 'No service ready' : n + (n === 1 ? ' service ready' : ' services ready')},
  fr:{h:n => n + (n > 1 ? ' heures' : ' heure'), r:n => n + (n > 1 ? ' requêtes' : ' requête'), m:n => n + (n > 1 ? ' minutes' : ' minute'),
    d:n => n + (n > 1 ? ' jours' : ' jour'), s:n => n + (n > 1 ? ' secondes' : ' seconde'), hm:n => n + (n > 1 ? ' heures par mois' : ' heure par mois'),
    svc:n => n === 0 ? 'Aucun service prêt' : n + (n > 1 ? ' services prêts' : ' service prêt')},
  ar:{h:n => arP(n, AR.hours), r:n => arP(n, ['طلب واحد','طلبان','{n} طلبات','{n} طلبًا','{n} طلب']), m:n => arP(n, ['دقيقة واحدة','دقيقتين','{n} دقائق','{n} دقيقة','{n} دقيقة']),
    d:n => arP(n, ['يوم واحد','يومين','{n} أيام','{n} يومًا','{n} يوم']), s:n => arP(n, ['ثانية واحدة','ثانيتين','{n} ثوانٍ','{n} ثانية','{n} ثانية']),
    hm:n => arP(n, ['ساعة واحدة شهريًا','ساعتان شهريًا','{n} ساعات شهريًا','{n} ساعة شهريًا','{n} ساعة شهريًا']),
    svc:n => n === 0 ? 'لا توجد خدمة جاهزة' : arP(n, ['خدمة واحدة جاهزة','خدمتان جاهزتان','{n} خدمات جاهزة','{n} خدمةً جاهزةً','{n} خدمة جاهزة'])}};
AR.hours = ['ساعة واحدة','ساعتان','{n} ساعات','{n} ساعة','{n} ساعة'];
const cn = () => COUNT[S.lang];
// Arabic dollars take their count's form for a whole amount, and the plain form for any other (CLDR "other").
const AR_USD = ['دولار واحد','دولاران','{n} دولارات','{n} دولارًا','{n} دولار'];
const CUR = {usd:{en:a => '$' + a, fr:a => a + ' $', ar:(a, v) => Number.isInteger(v) ? arP(v, AR_USD) : a + ' دولار'}};
const money = (v, cur) => CUR[cur][S.lang](Number.isInteger(v) ? String(v) : nf(v, 2), v);
const dateOf = iso => { const [y, m, d] = iso.split('-').map(Number); return d + ' ' + l.mo[m - 1] + ' ' + y; };

// The day the owner checks the mock-up and settings [voice] facts_days: a card read longer ago shows its date as old.
const TODAY = '2026-11-20', FACTS_DAYS = 180;
const daysTo = iso => Math.round((Date.parse(TODAY) - Date.parse(iso)) / 864e5);
// Light and heavy use in hours a month (services.toml light_hours, heavy_hours); a sample key's length; the samples typed
// on the lines: a good Custom address, one the line refuses, and an Other model id for OpenRouter.
const USE_H = {light:15, heavy:60}, KEY_LEN = 56, READ = '2026-11-17', OLD_READ = '2026-04-02';
const HOME = 'https://speech.example.net/v1/audio/transcriptions', LAN = 'http://192.168.1.20:8000/v1/audio/transcriptions', OR_MODEL = 'mistralai/voxtral-small-24b-2507';

// The offered rows of services.toml and services-own.toml, in the Cloud page's order, with the facts read on 2026-11-17
// (AssemblyAI's on 2026-11-19). A fact is its id, or its id with its numbers; site is the facts' page, keys where a key is
// made; test says a key check without sound exists; untested marks a row no real key has checked, every row until the
// owner's hands-on check; tw puts the training fact in the warning tone; warn names the card's warning, sw its switch.
const SVC = [
  {id:'groq', name:'Groq', site:'console.groq.com/docs/your-data', keys:'console.groq.com/keys', test:true, untested:true, cur:'usd', where:'us', trains:'never',
    keeps:['logs', {d:30, sw:'Settings → Data Controls'}], free:['daily', {h:8, r:20}], light:['freeOr', {lo:0.60, hi:1.67}], heavy:['freeOr', {lo:2.40, hi:6.66}],
    bills:['least', {s:10}]},
  {id:'lemonfox', name:'Lemonfox', site:'www.lemonfox.ai', keys:'www.lemonfox.ai/apis/keys', test:false, untested:true, cur:'usd', where:'unstated', trains:'unstated',
    keeps:'deleted', free:'firstMonth', light:['plain', {p:5}], heavy:['plain', {p:10}]},
  {id:'custom', test:false, untested:true, addr:true, where:'typed', trains:'unknown', keeps:'unknown', free:'unknown', light:'unknown', heavy:'unknown'},
  {id:'openrouter', name:'OpenRouter', site:'openrouter.ai/docs/guides/features/zdr', keys:'openrouter.ai/settings/keys', test:true, untested:true, keep0:true, cur:'usd',
    where:'host', trains:'host', keeps:'host', free:'small', light:['rangeFee', {lo:0.16, hi:4.05, fee:5.5}], heavy:['rangeFee', {lo:0.65, hi:16.20, fee:5.5}]},
  {id:'elevenlabs', name:'ElevenLabs', site:'elevenlabs.io/privacy-policy', keys:'elevenlabs.io/app/settings/api-keys', test:true, untested:true, cur:'usd',
    warn:'trainsProfile', sw:'Terms and privacy → Data use → Improve the models for everyone', tw:true,
    where:'us', trains:'default', keeps:'history', free:['minutesMonth', {m:30}], light:['plain', {p:6}], heavy:['plain', {p:22}], rule:'nonCommercial'},
  {id:'gemini', name:'Gemini', site:'aistudio.google.com/docs/pricing', keys:'aistudio.google.com/apikey', test:true, untested:true, cur:'usd', warn:'freeAllowance', tw:true,
    where:'unstated', trains:'freeAllowance', keeps:'paidLimited', free:'noCard', light:['paidPlain', {p:4.50}], heavy:['paidPlain', {p:18}], rule:'eea'},
  {id:'assemblyai', name:'AssemblyAI', site:'www.assemblyai.com/pricing', keys:'www.assemblyai.com/dashboard/api-keys', read:'2026-11-19', test:false, untested:true, cur:'usd',
    where:'eu', trains:'never', keeps:'unstated', free:['credit', {c:50}], light:['plain', {p:4.50}], heavy:['plain', {p:18}]}];
// A hidden row (OpenAI, Mistral, Together, DeepInfra: offered = false, ADR-0050) never shows, so the sample leaves it out.
const byId = id => SVC.find(s => s.id === id);
const svcName = s => s.id === 'custom' ? l.custom : s.name;
// Each warning's short mark on its row.
const WARN_MARK = {trainsProfile:'mTrains', freeAllowance:'mFree'};
// Cloud models for every language of the keyboard, shown while their service is on: Groq's two, Lemonfox's Whisper,
// ElevenLabs' Scribe v3 and the "Other model" id typed for OpenRouter.
const CM = {'gq-turbo':['Whisper Large v3 Turbo', 'groq'], 'gq-large':['Whisper Large v3', 'groq'], 'lf-large':['Whisper Large v3', 'lemonfox'],
  'el-scribe':['Scribe v3', 'elevenlabs'], 'or-typed':[OR_MODEL, 'openrouter']};
Object.keys(CM).forEach(id => { M[id] = [CM[id][0], 0]; });

// Sample state: the user already switched on Groq (tested), Lemonfox (a key; it has no test call) and OpenRouter (no key).
S.cloud = true;
S.svc = Object.fromEntries(SVC.map(s => [s.id, {st:'off', key:0, test:'', will:'ok', on:'', keep0:true}]));
Object.assign(S.svc.groq, {st:'ready', key:KEY_LEN, test:'ok', on:READ});
Object.assign(S.svc.openrouter, {st:'key', on:'2026-11-18'});
Object.assign(S.svc.lemonfox, {st:'ready', key:48, on:'2026-11-18'});
S.typed = {addr:{custom:LAN}, model:{}}; S.aim = ''; S.card = {id:'groq', ask:true, read:''}; S.cur = 'groq'; S.ret = 'cloud';
if (['svccard','svcpage'].includes(S.panel)) S.panel = 'cloud';

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
  const html = barBase(), u = S.models && inUse(), s = u && u.cloud && byId(u.cloud);
  if (S.bar !== 'listening' || !s || !s.warn) return html;
  const line = '<div class="vw' + A + '"' + D + '>' + t(l.warns[s.warn], {sw:bdi(s.sw || '')}) + '</div>';
  return html.replace('<div class="vbar', '<div class="vbar wide warns').replace(/<\/div>$/, line + '</div>');
};
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
    const inner = tickBox(i, true) + cell(svcName(v) + untestedTag(v), sub) + '<div class="marks">' + (v.warn ? mark(l[WARN_MARK[v.warn]], 'triangle-alert', 'warn') : '') + stateMark(v.id) + '</div>';
    return on ? tickRow([CK, '1fr', 'auto'], inner, i) : row([CK, '1fr', 'auto'], inner, '', 'flat off');
  });
  const band = '<div class="pn swb' + A + '" data-a="cloudsw" role="switch" aria-checked="' + on + '" tabindex="0">' + ic('cloud') + '<span class="x"' + D + '>' + l.master + '</span>' +
    '<span class="onoff' + A + '"' + D + '>' + (on ? l.on : l.off) + '</span><span class="sw' + (on ? ' on' : '') + '"></span></div>';
  return {icon:'cloud', title:l.cloud, sub:l.cloudSub, top:band + (on ? '' : note('power', l.masterOff)), body:list(rows),
    foot:btn('facts', 'file', l.facts, '', !live) + btn('svc', 'key', l.keyModel, '', !live || st === 'off') + btn('svcoff', 'power', l.swOff, '', !live || st === 'off') + gap +
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
  side:x => go(x === 'cloud' ? 'mcloud' : S.weak ? 'slow' : 'models'),
  aim:x => { S.aim = S.aim === x ? '' : x; }, keep0:() => { const v = S.svc[S.cur]; v.keep0 = !v.keep0; fallback(); }, link:() => {}});

// The controls on top, in English as on the other mock-ups, and what each new state shows.
CTL.cloud = [['cloud','Cloud page'],['card','Card: Groq'],['cardwarn','Card: ElevenLabs'],['cardgem','Card: Gemini'],['cardold','Card: old facts'],['key','Key works'],
  ['keybad','Key refused'],['address','Custom address'],['model','Other model'],['mcloud','Models: Cloud'],['slow','Models, slow PC'],['paste','Options: Paste']];
CTL.engine = [['local','Local model'],['cloud','Groq (cloud)'],['elevenlabs','ElevenLabs, first dictation']];
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
  mcloud:' <strong>Models, Cloud side.</strong> Its own list, five a page: only the models of the services you switched on, each with the cloud mark and its service; one still needing a key says so and Use stays dimmed. The band leads to Cloud services.'});

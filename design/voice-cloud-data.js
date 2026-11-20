// Part 2 of the voice typing mock-up (design/voice-bar.html): the cloud texts, counts, sample facts, service rows and
// sample state. voice-cloud.js, which loads after it, draws them. Reference only, never shipped.
'use strict';

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
  useLocal:'Use the local model', openSvc:'Open service',
  notes:{key:'The service refused the key. Check it on the key page.', terms:'The service’s terms changed. Confirm its facts again to send.',
    account:'The service asks for a step on its site, such as a verification, a payment method or a spending cap.',
    allowance:'The free allowance is used up. It renews at {at}.', off:'The service is off, so nothing was sent.'},
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
  useLocal:'Utiliser le modèle local', openSvc:'Ouvrir le service',
  notes:{key:'Le service a refusé la clé. Vérifiez-la sur la page de la clé.', terms:'Les conditions du service ont changé. Confirmez de nouveau ses informations pour envoyer.',
    account:'Le service demande une étape sur son site, comme une vérification, un moyen de paiement ou un plafond de dépenses.',
    allowance:'Le quota gratuit est épuisé. Il se renouvelle à {at}.', off:'Le service est désactivé ; rien n’a été envoyé.'},
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
  useLocal:'استخدام النموذج المحلي', openSvc:'فتح الخدمة',
  notes:{key:'رفضت الخدمة المفتاح. تحقّق منه في صفحة المفتاح.', terms:'تغيّرت شروط الخدمة. أكّد معلوماتها مرة أخرى لإرسال الصوت.',
    account:'تطلب الخدمة خطوةً على موقعها، مثل التحقق أو وسيلة دفع أو حدٍّ للإنفاق.',
    allowance:'نفدت الحصة المجانية، وتتجدد في الساعة {at}.', off:'الخدمة معطّلة، فلم يُرسَل شيء.'},
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

// Counts and money: CLDR plural forms as voice-bar-data.js's AR table; the decimal mark is its nf.
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
// Arabic dollars take their count's form for a whole amount, and the plain form for any other (CLDR "other").
const AR_USD = ['دولار واحد','دولاران','{n} دولارات','{n} دولارًا','{n} دولار'];
const CUR = {usd:{en:a => '$' + a, fr:a => a + ' $', ar:(a, v) => Number.isInteger(v) ? arP(v, AR_USD) : a + ' دولار'}};

// The day the owner checks the mock-up and settings [voice] facts_days: a card read longer ago shows its date as old.
const TODAY = '2026-11-20', FACTS_DAYS = 180;
// Light and heavy use in hours a month (services.toml light_hours, heavy_hours); a sample key's length; the samples typed
// on the lines: a good Custom address, one the line refuses, and an Other model id for OpenRouter.
const USE_H = {light:15, heavy:60}, KEY_LEN = 56, READ = '2026-11-17', OLD_READ = '2026-04-02';
const HOME = 'https://speech.example.net/v1/audio/transcriptions', LAN = 'http://192.168.1.20:8000/v1/audio/transcriptions', OR_MODEL = 'mistralai/voxtral-small-24b-2507';
// The clock time a used free allowance renews at, on its note (owner, 2026-11-20: a clock time, never a wait).
const RENEW_AT = '14:30';

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
// Each warning's short mark on its row.
const WARN_MARK = {trainsProfile:'mTrains', freeAllowance:'mFree'};
// Cloud models for every language of the keyboard, shown while their service is on: Groq's two, Lemonfox's Whisper,
// ElevenLabs' Scribe v3 and the "Other model" id typed for OpenRouter.
const CM = {'gq-turbo':['Whisper Large v3 Turbo', 'groq'], 'gq-large':['Whisper Large v3', 'groq'], 'lf-large':['Whisper Large v3', 'lemonfox'],
  'el-scribe':['Scribe v3', 'elevenlabs'], 'or-typed':[OR_MODEL, 'openrouter']};
Object.keys(CM).forEach(id => { M[id] = [CM[id][0], 0]; });

// Sample state: the user already switched on Groq (tested), Lemonfox (a key; it has no test call) and OpenRouter (no key).
// ElevenLabs is Off but keeps its sealed key, as after declined newer terms, so its row offers Remove key.
S.cloud = true;
S.svc = Object.fromEntries(SVC.map(s => [s.id, {st:'off', key:0, test:'', will:'ok', on:'', keep0:true}]));
Object.assign(S.svc.groq, {st:'ready', key:KEY_LEN, test:'ok', on:READ});
Object.assign(S.svc.openrouter, {st:'key', on:'2026-11-18'});
Object.assign(S.svc.lemonfox, {st:'ready', key:48, on:'2026-11-18'});
S.svc.elevenlabs.key = KEY_LEN;
S.typed = {addr:{custom:LAN}, model:{}}; S.aim = ''; S.card = {id:'groq', ask:true, read:''}; S.cur = 'groq'; S.ret = 'cloud';
if (['svccard','svcpage'].includes(S.panel)) S.panel = 'cloud';

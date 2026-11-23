// Voice typing mock-up (design/voice-bar.html): texts in three languages, number forms, sample data, the controls'
// words and the sample state. voice-bar.js and voice-bar-scene.js, which load after it, draw them. Reference only.
'use strict';

// Texts. EN, FR and AR words of the design's table are used as written; the rest are new and listed in the report.
const L = {
en:{code:'EN', own:'English', ar:false,
  listening:'Listening…', transcribing:'Transcribing…', namePrep:'Dictation, preparing the microphone', stop:'Stop', cancel:'Cancel', close:'Close', gear:'Dictation settings',
  copy:'Copy', copied:'Copied', undo:'Undo insert', again:'Try again', anyway:'Transcribe anyway',
  cancelled:'Cancelled. Nothing was typed.', nothing:'Nothing heard. Is the microphone on?', listenAgain:'Listen again',
  noModel:'No model for English yet', download:'Download', pwLine:'Typed into a password box. Not kept, no Copy.',
  said:'Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week.',
  title:'Dictation', back:'Back', langLine:'Language: English', models:'Models', device:'Device', compare:'Compare models', history:'History', options:'Options',
  pageBack:'Previous page', pageOn:'Next page', of:'{a}<br>of {b}',
  homeDevice:'Automatic (processor)', homeCompare:'Last run: 12 Nov', homeOptions:'All at defaults', changed:'Changed: {n}',
  againHint:'Transcribes the last recording again with the choices here.',
  forLang:'For English', best:'Recommended for English:', inUse:'In use', def:'Default', ready:'Ready', notDl:'Not downloaded', notHere:'Not for this PC', other:'Other file, not checked',
  unfitWhy:'Needs {need} of memory, {free} free', folderLine:'Models folder: {path} · {free} free', change:'Change', open:'Open',
  use:'Use for this language', delFile:'Delete file', pause:'Pause', resume:'Resume', got:'{a} of {b}', paused:'Paused · {a} of {b}',
  devHint:'Cards are checked before use', auto:'Automatic', autoSub:'The processor until a card passes its check', cpu:'Processor', cpuSub:'{n} cores · {mem} memory, {free} free',
  cardSub:'Graphics card · {mem}, {free} free', builtSub:'Built-in graphics · {mem} shared, {free} free', checking:'Checking: warming up the card',
  passed:'Passed · {x} faster', failed:'Check failed', failWhy:'Its text differed by {a}; {b} allowed.', devUse:'Use', check:'Check',
  intro:'Read this text aloud once. Each ready model then writes it, one after another.',
  read:'The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly.',
  start:'Start reading', step:'Model {a} of {b}', writing:'Writing…', doneIn:'Done · {t}', waiting:'Waiting',
  cols:['Model','Words wrong','Wait','Memory'], wrongOf:'{a} of {b}', kept:'Only these numbers are kept, not the recording or the texts.', againBtn:'Compare again', resultLine:'12 Nov · Automatic (processor)',
  histSub:'{n} · newest first', deleteAll:'Delete all', pin:'Pin', unpin:'Unpin', del:'Delete', insert:'Insert ({n})', insertOne:'Insert',
  ask:'Delete {n}? Pinned ones stay.', yes:'Yes', no:'No', days:{today:'Today', yesterday:'Yesterday', wed:'Wed', mon:'Mon'}, langs:{en:'English', fr:'French', ar:'Arabic'},
  forAll:'For every language', applies:'Changes apply at once.', on:'On', off:'Off', less:'Less', more:'More', autoN:'Auto ({n})',
  places:['Above the keyboard', 'Bottom of the screen', 'Top of the screen'], inserts:['Type it', 'Paste at once'],
  pasteHint:'Uses the clipboard, then puts yours back; a password box is always typed.',
  opt:{sounds:'Start and stop sounds', silenceStop:'Stop after silence', silenceS:'Silence before stopping', doneS:'Text stays on the bar', place:'Bar position',
    history:'Keep a history of texts', historyMax:'Texts kept', keepVoice:'Keep recordings', voiceMax:'Space for recordings', idle:'Model leaves memory after',
    recordMax:'Longest dictation', nothingS:'“Nothing heard” hint after', debounce:'Ignore a second click within', insert:'Insert the text', suffix:'Space after the text', typeGap:'Pause between letters',
    threads:'Processor threads', waveMs:'Wave smoothing', minFree:'Room left on the models drive', cardMargin:'Room left on a card', cardGain:'A card must be faster by', cardDiff:'A card’s text may differ by'},
  app:['Notes', 'Dear Maya,', 'The meeting moved to Thursday at ten.', 'Sign in', 'Email', 'Password']},
fr:{code:'FR', own:'Français', ar:false,
  listening:'Écoute…', transcribing:'Transcription…', namePrep:'Dictée, préparation du micro', stop:'Arrêter', cancel:'Annuler', close:'Fermer', gear:'Réglages de la dictée',
  copy:'Copier', copied:'Copié', undo:'Annuler l’insertion', again:'Réessayer', anyway:'Transcrire quand même',
  cancelled:'Annulé. Rien n’a été tapé.', nothing:'Rien entendu. Le micro est-il allumé\u202F?', listenAgain:'Écouter de nouveau',
  noModel:'Aucun modèle pour le français', download:'Télécharger', pwLine:'Tapé dans un champ de mot de passe. Non conservé, sans copie.',
  said:'Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine.',
  title:'Dictée', back:'Retour', langLine:'Langue\u00A0: français', models:'Modèles', device:'Unité de calcul', compare:'Comparer les modèles', history:'Historique', options:'Options',
  pageBack:'Page précédente', pageOn:'Page suivante', of:'{a}<br>sur {b}',
  homeDevice:'Automatique (processeur)', homeCompare:'Dernière comparaison\u00A0: 12 nov.', homeOptions:'Toutes par défaut', changed:'Modifiées\u00A0: {n}',
  againHint:'Retranscrit le dernier enregistrement avec les choix faits ici.',
  forLang:'Pour le français', best:'Conseillé pour le français\u00A0:', inUse:'Utilisé', def:'Par défaut', ready:'Prêt', notDl:'Non téléchargé', notHere:'Ne convient pas à ce PC', other:'Autre fichier, non vérifié',
  unfitWhy:'Demande {need} de mémoire, {free} libres', folderLine:'Dossier des modèles\u00A0: {path} · {free} libres', change:'Changer', open:'Ouvrir',
  use:'Utiliser pour cette langue', delFile:'Supprimer le fichier', pause:'Pause', resume:'Reprendre', got:'{a} sur {b}', paused:'En pause · {a} sur {b}',
  devHint:'Les cartes sont testées avant usage', auto:'Automatique', autoSub:'Le processeur tant qu’aucune carte n’a réussi son test', cpu:'Processeur', cpuSub:'{n} cœurs · {mem} de mémoire, {free} libres',
  cardSub:'Carte graphique · {mem}, {free} libres', builtSub:'Carte graphique intégrée · {mem} partagés, {free} libres', checking:'Test\u00A0: préchauffage de la carte',
  passed:'Réussi · {x} plus rapide', failed:'Test échoué', failWhy:'Son texte différait de {a}\u202F; {b} permis.', devUse:'Utiliser', check:'Tester',
  intro:'Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.',
  read:'Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte.',
  start:'Commencer la lecture', step:'Modèle {a} sur {b}', writing:'Écriture…', doneIn:'Terminé · {t}', waiting:'En attente',
  cols:['Modèle','Mots erronés','Attente','Mémoire'], wrongOf:'{a} sur {b}', kept:'Seuls ces chiffres sont gardés, ni l’enregistrement ni les textes.', againBtn:'Comparer à nouveau', resultLine:'12 nov. · Automatique (processeur)',
  histSub:'{n} · les plus récents d’abord', deleteAll:'Tout supprimer', pin:'Épingler', unpin:'Désépingler', del:'Supprimer', insert:'Insérer ({n})', insertOne:'Insérer',
  ask:'Supprimer {n}\u202F? Les textes épinglés restent.', yes:'Oui', no:'Non', days:{today:'Aujourd’hui', yesterday:'Hier', wed:'Mer.', mon:'Lun.'}, langs:{en:'Anglais', fr:'Français', ar:'Arabe'},
  forAll:'Pour toutes les langues', applies:'Les changements s’appliquent tout de suite.', on:'Activé', off:'Désactivé', less:'Moins', more:'Plus', autoN:'Auto ({n})',
  places:['Au-dessus du clavier', 'En bas de l’écran', 'En haut de l’écran'], inserts:['Le saisir', 'Le coller en une fois'],
  pasteHint:'Passe par le presse-papiers, puis y remet votre contenu ; un champ de mot de passe est toujours saisi.',
  opt:{sounds:'Sons de début et de fin', silenceStop:'Arrêt après un silence', silenceS:'Silence avant l’arrêt', doneS:'Durée d’affichage du texte', place:'Position de la barre',
    history:'Garder l’historique des textes', historyMax:'Textes conservés', keepVoice:'Garder les enregistrements', voiceMax:'Place pour les enregistrements', idle:'Le modèle quitte la mémoire après',
    recordMax:'Durée maximale d’une dictée', nothingS:'Avis «\u00A0Rien entendu\u00A0» après', debounce:'Ignorer un second clic pendant', insert:'Insérer le texte', suffix:'Espace après le texte', typeGap:'Pause entre les caractères',
    threads:'Fils d’exécution du processeur', waveMs:'Lissage de l’onde', minFree:'Espace laissé sur le disque des modèles', cardMargin:'Mémoire laissée sur la carte', cardGain:'Gain minimal d’une carte', cardDiff:'Écart permis pour une carte'},
  app:['Notes', 'Bonjour Maya,', 'La réunion est déplacée à jeudi à dix heures.', 'Connexion', 'E-mail', 'Mot de passe']},
ar:{code:'ع', own:'العربية', ar:true,
  listening:'جارٍ الاستماع…', transcribing:'جارٍ تحويل الكلام إلى نص…', namePrep:'إملاء، جارٍ تجهيز الميكروفون', stop:'إيقاف', cancel:'إلغاء', close:'إغلاق', gear:'إعدادات الإملاء',
  copy:'نسخ', copied:'تم النسخ', undo:'تراجع عن الإدراج', again:'إعادة المحاولة', anyway:'تحويل التسجيل إلى نص',
  cancelled:'أُلغي الإملاء، ولم يُكتب شيء.', nothing:'لم يُسمع شيء. هل يعمل الميكروفون؟', listenAgain:'إعادة الاستماع',
  noModel:'لا يوجد نموذج للعربية بعد', download:'تنزيل', pwLine:'كُتب في حقل كلمة مرور، فلا يُحفَظ ولا يُنسَخ.',
  said:'شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل.',
  title:'إملاء', back:'رجوع', langLine:'اللغة: العربية', models:'النماذج', device:'وحدة المعالجة', compare:'مقارنة النماذج', history:'السجل', options:'الخيارات',
  pageBack:'الصفحة السابقة', pageOn:'الصفحة التالية', of:'{a}<br>من {b}',
  homeDevice:'تلقائي (المعالج)', homeCompare:'آخر مقارنة: 12 نوفمبر', homeOptions:'جميعها على الوضع الافتراضي', changed:'المعدَّلة: {n}',
  againHint:'يعيد تحويل آخر تسجيل إلى نص بالاختيارات المحددة هنا.',
  forLang:'للغة العربية', best:'النموذج المُوصى به للعربية:', inUse:'قيد الاستخدام', def:'افتراضي', ready:'جاهز', notDl:'غير مُنزَّل', notHere:'لا يناسب هذا الحاسوب', other:'ملف آخر، لم يُتحقَّق منه',
  unfitWhy:'يحتاج إلى {need} من الذاكرة، والمتوفر {free}', folderLine:'مجلد النماذج: {path} · {free} متوفرة', change:'تغيير', open:'فتح',
  use:'استخدامه لهذه اللغة', delFile:'حذف الملف', pause:'إيقاف مؤقت', resume:'استئناف', got:'{a} من {b}', paused:'متوقف مؤقتًا · {a} من {b}',
  devHint:'تُفحَص البطاقات قبل استخدامها', auto:'تلقائي', autoSub:'يُستخدم المعالج إلى أن تنجح إحدى البطاقات في الفحص', cpu:'المعالج', cpuSub:'{n} أنوية · ذاكرة {mem}، المتوفر {free}',
  cardSub:'بطاقة الرسوميات · {mem}، المتوفر {free}', builtSub:'رسوميات مدمجة · {mem} مشتركة، المتوفر {free}', checking:'جارٍ الفحص: تهيئة البطاقة',
  passed:'نجح الفحص · أسرع بـ {x}', failed:'فشل الفحص', failWhy:'اختلف نصّها بـ {a}، والمسموح {b}.', devUse:'استخدام', check:'فحص',
  intro:'اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.',
  read:'يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة.',
  start:'بدء القراءة', step:'النموذج {a} من {b}', writing:'جارٍ الكتابة…', doneIn:'انتهى · {t}', waiting:'في الانتظار',
  cols:['النموذج','كلمات خاطئة','الانتظار','الذاكرة'], wrongOf:'{a} من {b}', kept:'تُحفَظ هذه الأرقام فقط، لا التسجيل ولا النصوص.', againBtn:'إعادة المقارنة', resultLine:'12 نوفمبر · تلقائي (المعالج)',
  histSub:'{n} · الأحدث أولًا', deleteAll:'حذف الكل', pin:'تثبيت', unpin:'إلغاء التثبيت', del:'حذف', insert:'إدراج ({n})', insertOne:'إدراج',
  ask:'هل تريد حذف {n}؟ تبقى النصوص المثبّتة.', yes:'نعم', no:'لا', days:{today:'اليوم', yesterday:'أمس', wed:'الأربعاء', mon:'الاثنين'}, langs:{en:'الإنجليزية', fr:'الفرنسية', ar:'العربية'},
  forAll:'لجميع اللغات', applies:'تُطبَّق التغييرات فورًا.', on:'مفعّل', off:'معطّل', less:'أقل', more:'أكثر', autoN:'تلقائي ({n})',
  places:['فوق لوحة المفاتيح', 'أسفل الشاشة', 'أعلى الشاشة'], inserts:['كتابته', 'لصقه دفعة واحدة'],
  pasteHint:'يستخدم الحافظة ثم يعيد إليها ما كان فيها؛ أما حقل كلمة المرور فيُكتب فيه النص دائمًا.',
  opt:{sounds:'صوتا البدء والإيقاف', silenceStop:'الإيقاف بعد الصمت', silenceS:'مدة الصمت قبل الإيقاف', doneS:'مدة بقاء النص على الشريط', place:'موضع الشريط',
    history:'الاحتفاظ بسجل النصوص', historyMax:'عدد النصوص المحفوظة', keepVoice:'الاحتفاظ بالتسجيلات', voiceMax:'المساحة المخصصة للتسجيلات', idle:'يُفرَّغ النموذج من الذاكرة بعد',
    recordMax:'أقصى مدة للإملاء', nothingS:'تنبيه «لم يُسمع شيء» بعد', debounce:'تجاهل النقرة الثانية خلال', insert:'إدراج النص', suffix:'مسافة بعد النص', typeGap:'الفاصل بين الأحرف',
    threads:'خيوط المعالجة', waveMs:'تنعيم الموجة', minFree:'المساحة المتروكة على قرص النماذج', cardMargin:'الذاكرة المتروكة على البطاقة', cardGain:'أدنى تسريع مطلوب من البطاقة', cardDiff:'الفرق المسموح في نص البطاقة'},
  app:['ملاحظات', 'عزيزتي مايا،', 'نُقل الاجتماع إلى يوم الخميس الساعة العاشرة.', 'تسجيل الدخول', 'البريد الإلكتروني', 'كلمة المرور']}
};

// Numbers: decimal marks as the clipboard's num.decimal; Arabic plural forms by CLDR (one, two, few, many, other).
const DEC = {en:'.', fr:',', ar:','};
const nf = (x, d) => (d === undefined ? String(x) : x.toFixed(d)).replace('.', DEC[S.lang]);
const arP = (n, f) => (n === 1 ? f[0] : n === 2 ? f[1] : n % 100 >= 3 && n % 100 <= 10 ? f[2] : n % 100 >= 11 ? f[3] : f[4]).replace('{n}', n);
const AR = {sec:['ثانية واحدة','ثانيتان','{n} ثوانٍ','{n} ثانية','{n} ثانية'], min:['دقيقة واحدة','دقيقتان','{n} دقائق','{n} دقيقة','{n} دقيقة'],
  texts:['نص واحد','نصان','{n} نصوص','{n} نصًا','{n} نص'], words:['كلمة واحدة','كلمتان','{n} كلمات','{n} كلمة','{n} كلمة'],
  ready:['نموذج واحد جاهز','نموذجان جاهزان','{n} نماذج جاهزة','{n} نموذجًا جاهزًا','{n} نموذج جاهز'], times:['مرة واحدة','مرتان','{n} مرات','{n} مرة','{n} مرة']};
const FM = {
  en:{mb:n => n + ' MB', gb:x => x + ' GB', s:x => x + ' s', sec:n => n + ' s', min:n => n + ' min', ms:n => n + ' ms', times:v => nf(v) + '×',
    texts:n => n + (n === 1 ? ' text' : ' texts'), words:n => n + (n === 1 ? ' word' : ' words'), ready:n => n + (n === 1 ? ' ready model' : ' ready models')},
  fr:{mb:n => n + ' Mo', gb:x => x + ' Go', s:x => x + ' s', sec:n => n + ' s', min:n => n + ' min', ms:n => n + ' ms', times:v => nf(v) + '×',
    texts:n => n + (n > 1 ? ' textes' : ' texte'), words:n => n + (n > 1 ? ' mots' : ' mot'), ready:n => n + (n > 1 ? ' modèles prêts' : ' modèle prêt')},
  ar:{mb:n => n + ' ميغابايت', gb:x => x + ' غيغابايت', s:x => x + ' ثانية', sec:n => arP(n, AR.sec), min:n => arP(n, AR.min), ms:n => n + ' ميلي ثانية',
    times:v => Number.isInteger(v) ? arP(v, AR.times) : nf(v) + ' مرة', texts:n => arP(n, AR.texts), words:n => arP(n, AR.words), ready:n => arP(n, AR.ready)}
};

// Sample data: names and sizes from the spec's models table; words wrong, waits and memory follow the trial in docs/items/P11.md.
const M = {parakeet:['Parakeet TDT 0.6b v3', 740], canary:['Canary 1b v2', 1140], small:['Whisper Small', 194], turbo:['Whisper Large v3 Turbo', 620],
  large:['Whisper Large v3', 1160], nemo:['Nemotron 3.5 ASR 0.6B', 751], qwen6:['Qwen3-ASR 0.6B', 850], qwen17:['Qwen3-ASR 1.7B', 2190],
  cohere:['Cohere Transcribe', 2410], cohereAr:['Cohere Transcribe Arabic', 2410], other:['ggml-medium.bin', 1530]};
// Each language's order (the first is its default) and each row's state: use, ready, dl (downloading), none, unfit, other.
const LIST = {
  en:[['parakeet','use'],['turbo','ready'],['cohere','none'],['large','dl'],['qwen17','unfit'],['canary','ready'],['small','ready'],['qwen6','ready'],['cohereAr','none'],['nemo','ready'],['other','other']],
  fr:[['parakeet','use'],['cohere','none'],['turbo','ready'],['qwen17','unfit'],['large','dl'],['canary','ready'],['small','ready'],['qwen6','ready'],['nemo','ready'],['other','other']],
  ar:[['cohereAr','use'],['large','dl'],['turbo','ready'],['qwen17','unfit'],['nemo','ready'],['small','ready'],['cohere','none'],['qwen6','ready'],['other','other']]};
const RESULT = {
  en:{total:28, rows:[['small',0,2.1,.5],['turbo',0,4.0,.5],['canary',1,1.8,1.2],['qwen6',1,2.5,.9],['parakeet',2,1.0,.8],['nemo',3,.9,.7]]},
  fr:{total:28, rows:[['parakeet',0,1.0,.8],['turbo',1,4.1,.5],['canary',2,1.9,1.2],['small',4,2.2,.5],['qwen6',5,2.6,.9],['nemo',9,1.0,.7]]},
  ar:{total:19, rows:[['cohereAr',0,2.4,1.7],['turbo',4,8.5,.5],['nemo',5,1.0,.7],['small',8,2.1,.5],['qwen6',13,2.6,.9]]}};
const HIST = [
  ['en','Thanks for the update. I will send the signed form on Thursday morning.','today','14:32','parakeet',false],
  ['fr','Bonjour Maya, la réunion est déplacée à jeudi à dix heures.','today','11:05','parakeet',true],
  ['ar','شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.','today','09:40','cohereAr',false],
  ['en','Remind me to call the pharmacy before six.','yesterday','18:12','parakeet',true],
  ['fr','Merci d’apporter le formulaire signé et les deux copies.','yesterday','16:48','parakeet',false],
  ['ar','يُرجى تأكيد موعد الزيارة يوم الأحد في العاشرة صباحًا.','yesterday','10:03','cohereAr',false],
  ['en','The evening train is quieter, so the ride is calmer.','wed','20:15','turbo',false],
  ['fr','Le train du soir est plus calme et les places sont faciles à trouver.','wed','19:02','canary',false],
  ['en','Please book a table for four on Saturday at eight.','wed','12:30','parakeet',false],
  ['ar','وصل الطلب اليوم، وسأتصل بك غدًا لتحديد موعد التسليم.','mon','17:45','turbo',false],
  ['fr','Peux-tu m’envoyer la liste des courses avant midi\u202F?','mon','09:20','parakeet',false],
  ['en','Draft: the new keyboard layout works well with one finger.','mon','08:55','parakeet',false]];
const DEV = [{id:'auto', icon:'arrow-left-right'}, {id:'cpu', icon:'cpu', cores:8, mem:16, free:9.4},
  {id:'nv', icon:'gpu', name:'NVIDIA GeForce RTX 4060 Laptop GPU', mem:8, free:7.1, gain:9},
  {id:'amd', icon:'gpu', name:'AMD Radeon 780M Graphics', built:true, mem:4, free:2.6, diff:3}];
// Options: the [voice] settings a user sets, with the plan's ranges; defaults from the design's settings table.
const OPTS = [['sounds','sw'], ['silenceStop','sw'], ['silenceS','st',{min:1, max:30, fmt:'sec', dep:'silenceStop'}], ['doneS','st',{min:2, max:60, fmt:'sec'}], ['place','ch'],
  ['history','sw'], ['historyMax','st',{list:[1,10,20,50,100,200,500], fmt:'texts', dep:'history'}], ['keepVoice','sw'],
  ['voiceMax','st',{list:[64,128,256,512,1024,2048,4096,8192,16384,32768,65536], fmt:'bin', dep:'keepVoice'}], ['idle','st',{list:[1,2,5,10,15,30,60,120], fmt:'min'}],
  ['recordMax','st',{list:[10,30,60,120,300,600,1200,1800,3600], fmt:'dur'}], ['nothingS','st',{min:2, max:60, fmt:'sec'}], ['debounce','st',{list:[100,200,300,400,600,800,1000,1500,2000], fmt:'ms'}],
  ['insert','two',{texts:'inserts', hint:'pasteHint'}], ['suffix','sw'], ['typeGap','st',{list:[0,5,10,20,50,100,200], fmt:'ms'}], ['threads','st',{min:0, max:64, fmt:'threads'}], ['waveMs','st',{list:[0,100,200,300,500,1000,2000], fmt:'ms'}],
  ['minFree','st',{list:[256,512,1024,2048,4096,8192,16384,32768,65536], fmt:'bin'}], ['cardMargin','st',{list:[0,256,512,1024,2048,4096,8192], fmt:'bin'}],
  ['cardGain','st',{list:[1,1.25,1.5,2,3,5,10], fmt:'times'}], ['cardDiff','st',{min:0, max:5, fmt:'words'}]];
const OPT0 = {sounds:true, silenceStop:false, silenceS:3, doneS:9, place:0, history:true, historyMax:50, keepVoice:false, voiceMax:1024, idle:5, recordMax:600, nothingS:5,
  debounce:400, insert:0, suffix:true, typeGap:10, threads:0, waveMs:300, minFree:2048, cardMargin:512, cardGain:1.5, cardDiff:1};
// The keyboard's legends, as selection-chips.html draws them.
const KB = {
  en:{words:['the','and','please'], sp:'English (United States)', lang:'EN', keys:['Esc','Del','Caps','Ctrl','Alt','AltGr'],
    rows:[['`','1','2','3','4','5','6','7','8','9','0','-','='],['q','w','e','r','t','y','u','i','o','p','[',']','\\'],['a','s','d','f','g','h','j','k','l',';',"'"],['<','z','x','c','v','b','n','m',',','.','/']]},
  fr:{words:['le','et','merci'], sp:'Français (France)', lang:'FR', keys:['Échap','Suppr','Verr. Maj','Ctrl','Alt','Alt Gr'],
    rows:[['²','&','é','"',"'",'(','-','è','_','ç','à',')','='],['a','z','e','r','t','y','u','i','o','p','^','$','*'],['q','s','d','f','g','h','j','k','l','m','ù'],['<','w','x','c','v','b','n',',',';',':','!']]},
  ar:{words:['في','من','على'], sp:'العربية', lang:'ع', keys:['Esc','Del','Caps','Ctrl','Alt','AltGr'],
    rows:[['ذ','1','2','3','4','5','6','7','8','9','0','-','='],['ض','ص','ث','ق','ف','غ','ع','ه','خ','ح','ج','د','\\'],['ش','س','ي','ب','ل','ا','ت','ن','م','ك','ط'],['|','ئ','ء','ؤ','ر','لا','ى','ة','و','ز','ظ']]}};

// The controls on top, in English, as on the other mock-ups.
const CTL = {
  bar:[['preparing','Preparing (no bar)'],['listening','Listening'],['transcribing','Transcribing'],['text','Text'],['password','Password box'],['cancelled','Cancelled'],['nothing','Nothing heard'],['nomodel','No model yet']],
  panel:[['none','Closed'],['home','Home'],['models','Models'],['device','Device'],['compare','Compare'],['comparing','Comparing'],['results','Results'],['history','History'],['options','Options']],
  theme:[['native','Native Adaptive'],['et66','ET66'],['dolch','Modern Dolch']], mode:[['light','Light'],['dark','Soft dark']], lang:[['en','EN'],['fr','FR'],['ar','ع']],
  size:[['1','S'],['1.25','M'],['1.5','L']], still:[['0','Full'],['1','Reduced']], view:[['kb','With keyboard'],['close','Close-up']]};
const SAY = {
  listening:'<strong>Listening.</strong> Cancel at the left end, Stop at the right end, the live wave between them (Wispr Flow’s bar, ChatGPT’s microphone). The microphone key glows while the microphone is open.',
  transcribing:'<strong>Transcribing.</strong> The shimmer says the words are on their way (Gemini’s and ChatGPT’s loading shimmer). Stop stays where it was, dimmed, so nothing moves under your finger.',
  text:'<strong>Text.</strong> The text is already typed; the bar shows its first lines with Copy and Undo insert, then fades after 9 seconds (a snackbar with its action).',
  password:'<strong>Password box.</strong> The text is typed, but the bar shows dots and offers no Copy, and History keeps nothing.',
  cancelled:'<strong>Cancelled.</strong> Nothing was typed; the recording waits until the bar closes, so it can still be transcribed.',
  nothing:'<strong>Note: nothing heard.</strong> One line says what happened, with one action (Windows 11 InfoBar).',
  nomodel:'<strong>Note: no model yet.</strong> Download fetches the recommended model and opens the Models page.',
  none:'<strong>Bar closed.</strong> Pick a bar state above to show it again.',
  home:' <strong>Voice panel.</strong> Each row shows its value and opens its page; Try again works while a text is on the bar.',
  models:' <strong>Models.</strong> Tick a row, then act in the footer, as in the clipboard panel; the download row has its own Pause and Cancel (Microsoft Store).',
  device:' <strong>Device.</strong> Automatic, the processor and each card with its memory (Windows Settings › Graphics); a card is checked before use.',
  compare:' <strong>Compare models, step 1.</strong> Read the set text aloud once.',
  comparing:' <strong>Step 2.</strong> Each ready model writes the recording in turn, with Cancel.',
  results:' <strong>Step 3.</strong> The trial table’s columns; tick a model and use it for this language.',
  history:' <strong>History.</strong> Tick texts in order, then Insert, Copy, Pin or Delete; only Delete all asks first.',
  options:' <strong>Options.</strong> Switches with On or Off, minus and plus steppers, no slider (Windows Settings).',
  zoom:' Close-up shows it larger.',
  still:' <strong>Reduced motion:</strong> the wave is a plain level bar (Windows Sound settings) and the shimmer stands still.'};

// The mock-up's own pacing in ms (writing, a Copied flash, preparing, a key's test), not the product's timings.
const PACE = {write:1800, flash:1200, prep:1600, test:1200};

// State, kept between visits like selection-chips.html.
const S = {theme:'native', mode:'light', lang:'en', size:1.25, still:matchMedia('(prefers-reduced-motion: reduce)').matches, view:'kb', bar:'listening', panel:'none',
  pg:0, tick:[], ask:false, pw:false, undone:false, copied:false, hcopied:false, extra:'', devUse:'auto', check:null, passed:{}, failed:{amd:true}, opt:{...OPT0}, t0:performance.now(),
  engine:'local', weak:false};
const KEEP = ['theme','mode','lang','size','still','view','bar','panel','engine','weak'];
try { const k = JSON.parse(localStorage.getItem('kx-voice') || '{}'); KEEP.forEach(n => { if (n in k) S[n] = k[n]; }); } catch (e) {}
const save = () => { try { localStorage.setItem('kx-voice', JSON.stringify(Object.fromEntries(KEEP.map(n => [n, S[n]])))); } catch (e) {} };
function fresh() {
  S.models = LIST[S.lang].map(([id, st], i) => ({id, st, def:i === 0, p:st === 'dl' ? .46 : 0, paused:false}));
  addCloud();
  S.hist = HIST.map(h => ({lang:h[0], text:h[1], day:h[2], time:h[3], model:h[4], pin:h[5]}));
  S.tick = []; S.pg = 0; S.ask = false; S.undone = false;
}

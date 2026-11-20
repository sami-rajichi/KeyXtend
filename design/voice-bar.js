// Voice typing mock-up (design/voice-bar.html): texts in three languages, sample data and drawing. Reference only.
// Part 2's cloud services live in voice-cloud-data.js and voice-cloud.js, which load after this file and before boot().
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
const f = () => FM[S.lang];
const fileSize = mb => mb >= 1000 ? f().gb(nf(+(mb / 1000).toFixed(2))) : f().mb(mb);
const binSize = mb => mb >= 1024 ? f().gb(nf(mb / 1024)) : f().mb(mb);
const gb1 = x => f().gb(nf(x, 1));
const secs = x => f().s(nf(x, 1));
const t = (s, v = {}) => s.replace(/\{(\w+)\}/g, (_, k) => v[k] === undefined ? '' : v[k]);

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
  return g('Bar', 'bar', CTL.bar, S.bar) + g('Voice panel', 'panel', CTL.panel, S.panel) + g('Cloud (part 2)', 'panel', CTL.cloud, S.panel) + g('Dictating with', 'engine', CTL.engine, S.engine) +
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
// The mock-up's own pacing in ms (writing, a Copied flash, preparing, a key's test), not the product's timings.
const PACE = {write:1800, flash:1200, prep:1600, test:1200};
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

### Textes français de la dictée (P11), chargés dans le même paquet que fr.ftl ; les trois fichiers ont les mêmes clés.
### Les formes du pluriel se choisissent selon $form, le nombre tel que le lisent les règles CLDR ; [0] est le nombre 0.

## Dans l’ordre : la barre (nom, réglages, états, boutons, l’avis tant que rien n’est entendu, les points du mot de passe,
## le chronomètre), chaque note par identifiant, l’en-tête et l’accueil du panneau, Modèles, Unité de calcul, Comparer
## les modèles, Historique, puis Options (chaque option par sa clé de réglage, tirets pour les soulignés), les positions
## de la barre et les unités. $lang est le nom de la langue dans cette langue.
voice =
    .title = Dictée
    .gear = Réglages de la dictée
    .listening = Écoute…
    .transcribing = Transcription…
    .preparing = Dictée, préparation du micro
    .nothing = Rien entendu pour l’instant
    .stop = Arrêter
    .cancel = Annuler
    .close = Fermer
    .copy = Copier
    .copied = Copié
    .undo = Annuler l’insertion
    .again = Réessayer
    .anyway = Transcrire quand même
    .listen = Écouter de nouveau
    .download = Télécharger
    .download-size = Télécharger · { $size }
    .mask = ••••••••
    .timer = { $minutes }:{ $seconds }
    .note-nothing = Rien entendu. Le micro est-il allumé ?
    .note-cancelled = Annulé. Rien n’a été tapé.
    .note-no-mic = Aucun micro n’a été trouvé.
    .note-mic-busy = Une autre application utilise le micro.
    .note-mic-refused = Les paramètres de confidentialité bloquent le micro.
    .note-unplugged = Le micro a été débranché.
    .note-no-model = Aucun modèle en { $lang } pour l’instant
    .note-language = Le modèle choisi n’écrit pas en { $lang }. Choisissez-en un autre dans les réglages.
    .note-checking = Le fichier choisi est encore en cours de vérification. Réessayez dans un instant.
    .note-no-language = Aucun modèle de la liste n’écrit en { $lang }.
    .note-no-fit = Pas assez de mémoire libre pour ce modèle. Fermez des applications ou choisissez un modèle plus petit.
    .note-fault = La dictée s’est arrêtée après une erreur. Réessayez.
    .note-pieces = La dictée s’est arrêtée en cours de route. Voici le texte déjà écrit.
    .note-too-long = Arrêt à la durée maximale ; le texte a été tapé.
    .note-no-field = Le texte n’a pas été tapé en entier. Copiez-le ici ou depuis l’historique.
    .note-stopped = La saisie s’est arrêtée en cours de route. Copiez le texte entier ici ou depuis l’historique.
    .note-copy-failed = Le texte n’a pas pu être copié. Réessayez.
    .note-password = Tapé dans un champ de mot de passe. Non conservé, sans copie.
    .note-fell = La carte graphique a échoué ; le processeur a écrit ce texte.
    .note-print = Un téléchargement n’a pas réussi sa vérification ; il a été supprimé.
    .note-taken = Un autre fichier du dossier des modèles porte déjà ce nom ; rien n’a été téléchargé.
    .note-full = Pas assez d’espace sur le disque ; rien n’a été téléchargé.
    .note-folder = Le dossier des modèles est inutilisable. Changez-le sur la page Modèles.
    .note-net = Le téléchargement n’a pas abouti. Réessayez plus tard.
    .back = Retour
    .lang = Langue : { $lang }
    .models = Modèles
    .device = Unité de calcul
    .compare = Comparer les modèles
    .history = Historique
    .options = Options
    .no-model = Aucun modèle prêt
    .auto-cpu = Automatique (processeur)
    .auto-on = Automatique ({ $device })
    .last-run = Dernière comparaison : { $date }
    .never = Aucune comparaison pour l’instant
    .texts = { $form ->
        [one] { $count } texte
       *[other] { $count } textes
    }
    .defaults = Toutes par défaut
    .changed = Modifiées : { $count }
    .again-hint = Retranscrit le dernier enregistrement avec les choix faits ici.
    .for-lang = En { $lang }
    .best = Conseillé en { $lang } :
    .untested = Cet ordre n’a pas été testé en { $lang }.
    .slow = Les modèles locaux en { $lang } seront très lents sur ce PC.
    .mark-used = Utilisé
    .mark-default = Par défaut
    .mark-ready = Prêt
    .mark-download = Non téléchargé
    .mark-not-here = Ne convient pas à ce PC
    .mark-other = Autre fichier, non vérifié
    .unfit = Demande { $need } de mémoire, { $free } libres
    .got = { $done } sur { $total }
    .paused = En pause · { $done } sur { $total }
    .pause = Pause
    .resume = Reprendre
    .use = Utiliser pour cette langue
    .delete-file = Supprimer le fichier
    .folder = Dossier des modèles : { $path } · { $free } libres
    .change = Changer
    .open = Ouvrir
    .folder-offer = Garder les modèles dans { $path } ? { $free } libres.
    .folders = Dossier des modèles
    .drives = Lecteurs
    .here = Utiliser ce dossier
    .device-hint = Les cartes sont testées avant usage
    .auto = Automatique
    .auto-line = Le processeur tant qu’aucune carte n’a réussi son test
    .cpu = Processeur
    .cpu-line = { $cores } · { $total } de mémoire, { $free } libres
    .cores = { $form ->
        [one] { $count } cœur
       *[other] { $count } cœurs
    }
    .card-line = Carte graphique · { $total }, { $free } libres
    .built-in-line = Carte graphique intégrée · { $total } partagés, { $free } libres
    .checking = Test : préchauffage de la carte
    .faster = { $times }× plus rapide
    .mark-passed = Réussi · { $faster }
    .mark-failed = Test échoué
    .why-slow = Elle n’était que { $faster } ; il fallait au moins { $need }.
    .why-differs = Son texte différait de { $words } ; { $allowed } permis.
    .why-room = La carte n’avait pas assez de mémoire libre.
    .why-error = La carte a signalé une erreur.
    .failed = { $name } : test échoué. { $why }
    .check = Tester
    .use-device = Utiliser
    .intro = Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.
    .intro-free = Dites quelques phrases à voix haute une fois. Chaque modèle prêt les écrit ensuite, l’un après l’autre.
    .read-text = Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte.
    .ready = { $form ->
        [one] { $count } modèle prêt · { $device }
       *[other] { $count } modèles prêts · { $device }
    }
    .start = Commencer la lecture
    .recording = Lisez le texte maintenant, puis appuyez sur Arrêter dans la barre.
    .step = Modèle { $at } sur { $of }
    .mark-done = Terminé · { $wait }
    .mark-writing = Écriture…
    .mark-waiting = En attente
    .mark-run-failed = Échec
    .col-model = Modèle
    .col-wrong = Mots erronés
    .col-wait = Attente
    .col-memory = Mémoire
    .wrong-of = { $wrong } sur { $words }
    .kept = Seuls ces chiffres sont gardés, ni l’enregistrement ni les textes.
    .result-line = { $date } · { $device }
    .rerun = Comparer à nouveau
    .history-sub = { $texts } · les plus récents d’abord
    .meta = { $when } · { $lang } · { $model }
    .pin = Épingler
    .unpin = Désépingler
    .delete = Supprimer
    .delete-all = Tout supprimer
    .insert = Insérer ({ $count })
    .insert-one = Insérer
    .confirm = { $form ->
        [0] Aucun texte non épinglé à supprimer.
        [one] Supprimer { $count } texte ? Les textes épinglés restent.
       *[other] Supprimer { $count } textes ? Les textes épinglés restent.
    }
    .yes = Oui
    .no = Non
    .empty = Aucun texte pour l’instant. Dictez quelque chose et il s’affichera ici.
    .ticked = Coché { $count }
    .pinned = Épinglé
    .join = {", "}
    .for-all = Pour toutes les langues
    .applies = Les changements s’appliquent tout de suite.
    .on = Activé
    .off = Désactivé
    .minus = Moins : { $name }
    .plus = Plus : { $name }
    .opt-sounds = Sons de début et de fin
    .opt-silence-stop = Arrêt après un silence
    .opt-silence-stop-s = Silence avant l’arrêt
    .opt-done-ms = Durée d’affichage du texte
    .opt-place = Position de la barre
    .opt-history = Garder l’historique des textes
    .opt-history-max = Textes conservés
    .opt-keep-voice = Garder les enregistrements
    .opt-voice-max-mb = Place pour les enregistrements
    .opt-idle-min = Le modèle quitte la mémoire après
    .opt-record-max-s = Durée maximale d’une dictée
    .opt-nothing-s = Avis « Rien entendu » après
    .opt-debounce-ms = Ignorer un second clic pendant
    .opt-insert = Insérer le texte
    .opt-type-suffix = Espace après le texte
    .opt-type-gap-ms = Pause entre les caractères
    .opt-threads = Fils d’exécution du processeur
    .opt-wave-ms = Lissage de l’onde
    .opt-min-free-mb = Espace laissé sur le disque des modèles
    .opt-card-margin-mb = Mémoire laissée sur la carte
    .opt-card-gain = Gain minimal d’une carte
    .opt-card-diff = Écart permis pour une carte
    .place-above = Au-dessus du clavier
    .place-bottom = En bas de l’écran
    .place-top = En haut de l’écran
    .insert-type = Le saisir
    .insert-paste = Le coller en une fois
    .insert-hint = Passe par le presse-papiers, puis y remet votre contenu ; un champ de mot de passe est toujours saisi.
    .seconds = { $count } s
    .minutes = { $count } min
    .ms = { $count } ms
    .words = { $form ->
        [one] { $count } mot
       *[other] { $count } mots
    }
    .times = { $times }×
    .secs = { $s } s
    .auto-n = Auto ({ $count })
    .mb = { $mb } Mo
    .gb = { $gb } Go

## Le nom d’une langue par code ISO 639, comme l’historique nomme la langue d’un texte ; un autre code s’affiche tel quel.
langname =
    .en = Anglais
    .fr = Français
    .ar = Arabe

## Le nom d’une langue dans une phrase, par code ISO 639, en minuscules ; un autre code garde le nom donné par le système.
langin =
    .en = anglais
    .fr = français
    .ar = arabe

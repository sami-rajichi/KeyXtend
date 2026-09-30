### Textes français (ADR-0017), avec les termes de Windows ; une clé absente vient de l’anglais.
### Une espace insécable précède les deux-points, le point-virgule, le point d’exclamation et le point d’interrogation.

## Touches nommées, par identifiant de board.toml : le nom des info-bulles et des lecteurs d’écran.
key =
    .esc = Échap
    .bksp = Retour arrière
    .tab = Tabulation
    .del = Supprimer
    .caps = Verrouillage des majuscules
    .enter = Entrée
    .shift = Maj
    .rshift = Maj droite
    .up = Flèche haut
    .menu = Touche Menu
    .ctrl = Ctrl
    .win = Touche Windows
    .alt = Alt
    .lang = Langue
    .space = Espace
    .altgr = Alt Gr
    .left = Flèche gauche
    .down = Flèche bas
    .right = Flèche droite
    .rclick = Clic droit par appui long
    .grab = Saisir : glisser-déposer, sélectionner du texte
    .shortcuts = Raccourcis
    .mic = Saisie vocale
    .clip = Historique du Presse-papiers
    .snip = Capture d’écran
    .fill = Remplissage rapide
    .settings = Paramètres
    .power = Marche/Arrêt
    .dpad = Pavé de défilement

## Le mot court sur la touche, ou sous son icône en grande taille, comme sur les claviers français.
label =
    .esc = Échap
    .bksp = Retour arrière
    .tab = Tab
    .del = Suppr
    .caps = Verr. Maj
    .enter = Entrée
    .shift = Maj
    .rshift = Maj
    .up = Haut
    .menu = Menu
    .ctrl = Ctrl
    .win = Win
    .alt = Alt
    .altgr = Alt Gr
    .left = Gauche
    .down = Bas
    .right = Droite
    .rclick = Clic droit
    .grab = Saisir
    .shortcuts = Raccourcis
    .mic = Dictée
    .clip = Presse-papiers
    .snip = Capture
    .fill = Remplir
    .settings = Paramètres
    .power = Arrêt

tip =
    .later = Disponible dans une version ultérieure

## Boutons de la barre du haut, coin de redimensionnement et bulle, par identifiant [bar] d’icons.toml.
bar =
    .grip = Déplacer le clavier : cliquez, déplacez, cliquez
    .mode = Passer en mode sombre
    .mode_on = Passer en mode clair
    .theme = Changer de thème
    .smaller = Plus petit
    .bigger = Plus grand
    .minimise = Réduire en bulle
    .close = Masquer le clavier
    .corner = Redimensionner : cliquez, déplacez, cliquez
    .bubble = Afficher le clavier

tray =
    .show = Afficher le clavier
    .quit = Quitter

## Boutons d’une notification.
button =
    .retry = Réessayer
    .ok = OK

## Notifications, par clé de kx-module-api. $module est un nom de module et $keys une liste de paramètres.
notice =
    .settings-reset = Certains paramètres du module { $module } n’étaient pas valides et ont repris leur valeur par défaut : { $keys }.
    .settings-restored = Le fichier de paramètres était endommagé ; sa dernière copie valide a été restaurée.
    .settings-defaults = Aucun fichier de paramètres n’était utilisable ; les paramètres par défaut sont utilisés.
    .settings-newer = Les paramètres du module { $module } viennent d’une version plus récente. Ils sont conservés et les valeurs par défaut sont utilisées pour l’instant.
    .settings-unsaved = Impossible d’enregistrer les paramètres ; les modifications dureront jusqu’à la fermeture du clavier.
    .settings-locked = Un autre programme utilise le fichier de paramètres. Les paramètres par défaut sont utilisés et rien n’est enregistré.
    .module-failed = Le module { $module } s’est arrêté après une erreur. Tout le reste fonctionne toujours.
    .keys-not-sent = Certaines touches ne sont pas parvenues à l’application au premier plan. Vérifiez le texte saisi.
    .lang-not-switched = L’application au premier plan n’a pas pu changer de langue.

## Noms des modules, par identifiant ; un identifiant absent s’affiche tel quel.
module =
    .kernel = Général
    .keyboard = Clavier
    .layouts = Dispositions de clavier
    .ui = Apparence

## Les présentations de la touche de langue, par valeur de [keyboard] lang_key.
langkey =
    .carousel = Carrousel
    .cycle = Cycle
    .list = Liste

## La barre d’espace : la langue, puis la disposition.
lang =
    .space = { $lang } · { $layout }

## Noms des thèmes, par identifiant de themes.toml : les noms de modèles restent, les mots courants sont traduits.
theme =
    .native = Natif adaptatif
    .et66 = ET66
    .dolch = Dolch moderne

## Listes : ce qui sépare deux éléments, et la fin d’une liste qui en omet $more.
## Les formes du pluriel se choisissent selon $form, c’est-à-dire $more tel que le lisent les règles CLDR.
list =
    .join = {", "}
    .more = { $list } et { $form ->
        [one] { $more } autre
       *[other] { $more } autres
    }

### Textes français (ADR-0017), avec les termes de Windows ; une clé absente vient de l’anglais.
### Une espace insécable précède les deux-points, le point-virgule, le point d’exclamation et le point d’interrogation.

## Le nom du produit, écrit une fois et repris par les textes ci-dessous.
-app = KeyXtend

## Touches nommées, par identifiant de board.toml, puis .empty pour une touche qui ne tape rien : le nom des info-bulles et des lecteurs d’écran.
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
    .empty = Touche vide

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
    .one-lang = Une seule langue de clavier est installée
    .mouse-off = L’aide à la souris est désactivée
    .scroll-off = Le pavé de défilement est désactivé
    .rclick = Maintenez l’appui sans bouger pour faire un clic droit ; les clics brefs restent normaux
    .grab = Appuyez pour saisir, déplacez, puis cliquez pour déposer

## Les signes arabes par leur nom, que disent les lecteurs d’écran ; .join relie ceux tapés ensemble.
mark =
    .fathatan = Tanwin fatha
    .dammatan = Tanwin damma
    .kasratan = Tanwin kasra
    .fatha = Fatha
    .damma = Damma
    .kasra = Kasra
    .shadda = Chadda
    .sukun = Soukoun
    .join = {" avec "}

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

## Les tailles prédéfinies de la barre du haut, dans l’ordre de [keyboard] presets : la lettre de chaque bouton, puis son nom.
size =
    .s = P
    .m = M
    .l = G
sizename =
    .s = Petit
    .m = Moyen
    .l = Grand

## Les flèches du pavé directionnel et son bouton Arrêt, par identifiant [dpad] d’icons.toml.
dpad =
    .up = Défiler vers le haut
    .right = Défiler vers la droite
    .left = Défiler vers la gauche
    .down = Défiler vers le bas
    .stop = Arrêter le défilement

## Clair, sombre ou comme Windows, par valeur de [ui] mode.
mode =
    .light = Clair
    .dark = Sombre
    .auto = Automatique

## Titres des fenêtres, que les lecteurs d’écran annoncent.
window =
    .keyboard = Clavier { -app }
    .bubble = Clavier { -app } (réduit)
    .notice = Notification de { -app }
    .menu = Menu de { -app }
    .list = Langues du clavier
    .tip = Info-bulle de { -app }
    .pointer = Aide au pointeur

## Ce que les lecteurs d’écran annoncent pour le voile qui termine un déplacement ou un redimensionnement.
catch = Cliquez pour terminer le déplacement ou le redimensionnement

## Le badge de mode près du pointeur : son mot pendant que Saisir maintient un appui.
badge =
    .holding = Saisi

tray =
    .show = Afficher le clavier
    .quit = Quitter

## Boutons d’une notification.
button =
    .retry = Réessayer
    .ok = OK

## Le bandeau des notifications : combien d’autres attendent derrière celle qu’il montre.
## Les formes du pluriel se choisissent selon $form, qui est $more lu par les règles CLDR ; $more vaut au moins 1.
banner =
    .more = { $form ->
        [one] { $more } autre notification
       *[other] { $more } autres notifications
    }

## Listes paginées : les boutons de page, et le numéro de page entre eux.
page =
    .back = Page précédente
    .on = Page suivante
    .of = { $at } sur { $count }

## Notifications, par clé de kx-module-api. $module est un nom de module et $keys une liste de paramètres.
notice =
    .settings-reset = Certains paramètres de « { $module } » n’étaient pas valides et ont repris leur valeur par défaut : { $keys }.
    .settings-restored = Le fichier de paramètres était endommagé ; sa dernière copie valide a été restaurée.
    .settings-defaults = Aucun fichier de paramètres n’était utilisable ; les paramètres par défaut sont utilisés.
    .settings-newer = Les paramètres de « { $module } » viennent d’une version plus récente. Ils sont conservés et les valeurs par défaut sont utilisées pour l’instant.
    .settings-unsaved = Impossible d’enregistrer les paramètres ; les modifications ne dureront que jusqu’à la fermeture du clavier.
    .settings-locked = Un autre programme utilise le fichier de paramètres. Les paramètres par défaut sont utilisés et rien n’est enregistré.
    .module-failed = La partie « { $module } » s’est arrêtée après une erreur. Tout le reste fonctionne toujours.
    .keys-not-sent = Certaines touches ne sont pas parvenues à l’application au premier plan. Vérifiez le texte saisi.
    .lang-not-switched = L’application au premier plan n’a pas pu changer de langue.
    .mouse-hook-failed = L’aide à la souris n’a pas pu démarrer, car Windows a refusé qu’elle surveille la souris. Appuyez de nouveau sur la touche.
    .mouse-reset = L’aide à la souris a rencontré une erreur ; ses modes ont donc été désactivés. Appuyez sur une touche de mode pour la relancer.
    .mouse-stopped = L’aide à la souris s’est arrêtée après une erreur. Redémarrez { -app } pour l’utiliser de nouveau.
    .scroll-point = Pointez ce que vous voulez faire défiler, puis touchez une flèche.
    .scroll-none = Rien à faire défiler ici.
    .scroll-end = Déjà à la fin.
    .scroll-failed = Le pavé de défilement a rencontré une erreur. Touchez une flèche pour réessayer.
    .scroll-stopped = Le pavé de défilement s’est arrêté après une erreur. Redémarrez { -app } pour l’utiliser de nouveau.

## Noms des modules, par identifiant ; un identifiant absent s’affiche tel quel.
module =
    .kernel = { -app }
    .keyboard = Clavier
    .layouts = Dispositions de clavier
    .mouse = Aide à la souris
    .scroll = Pavé de défilement
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

## Listes : ce qui sépare deux éléments, la fin d’une liste qui en omet $more, et une liste qui n’en nomme aucun.
## Les formes du pluriel se choisissent selon $form, c’est-à-dire $more tel que le lisent les règles CLDR ; $more vaut au moins 1.
list =
    .join = {", "}
    .more = { $list } et { $form ->
        [one] { $more } autre
       *[other] { $more } autres
    }
    .count = { $form ->
        [one] { $more } paramètre
       *[other] { $more } paramètres
    }

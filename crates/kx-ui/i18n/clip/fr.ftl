### Textes français du panneau du Presse-papiers (P8), chargés dans le même paquet que fr.ftl ; les trois fichiers ont les mêmes clés.
### Les formes du pluriel se choisissent selon $form, le nombre tel que le lisent les règles CLDR ; [0] est le nombre 0 exact.

## Le panneau : titre, durée de conservation, aide de la recherche, Fermer, filtres, noms des types (ligne sans texte), pied, question.
## L’heure d’une ligne est un jour (lundi vaut 1) et l’heure sur 24 h, sinon la date ; les tailles arrivent avec deux décimales.
## La ligne d’infos omet l’application inconnue ; les lecteurs d’écran entendent .ticked, .pinned et .join après le texte d’une ligne.
clip =
    .title = Historique du Presse-papiers
    .kept = { $form ->
        [one] Conservés { $days } jour
       *[other] Conservés { $days } jours
    }
    .search = Tapez pour rechercher
    .close = Fermer
    .filter-all = Tout
    .filter-text = Texte
    .filter-images = Images
    .filter-files = Fichiers
    .filter-media = Médias
    .kind-text = Texte
    .kind-link = Lien
    .kind-image = Image
    .kind-file = Fichier
    .kind-media = Média
    .preview = Aperçu
    .pin = Épingler
    .unpin = Désépingler
    .delete = Supprimer
    .delete-all = Tout supprimer
    .insert = Insérer ({ $count })
    .insert-one = Insérer
    .back = Retour
    .confirm = { $form ->
        [0] Aucun élément non épinglé à supprimer.
        [one] Supprimer { $count } élément ? Les éléments épinglés restent.
       *[other] Supprimer { $count } éléments ? Les éléments épinglés restent.
    }
    .yes = Oui
    .no = Non
    .empty = Rien n’a encore été copié. Copiez un texte, une image ou un fichier pour le voir ici.
    .today = Aujourd’hui
    .yesterday = Hier
    .day-1 = Lun.
    .day-2 = Mar.
    .day-3 = Mer.
    .day-4 = Jeu.
    .day-5 = Ven.
    .day-6 = Sam.
    .day-7 = Dim.
    .when = { $word } { $time }
    .time = { $hour }:{ $minute }
    .date = { $day }/{ $month }/{ $year }
    .size = { $mb } Mo
    .size-gb = { $gb } Go
    .size-small = < { $mb } Mo
    .size-unknown = Taille inconnue
    .storage = Historique : { $used } sur { $cap } · { $free } libres sur { $drive }
    .open-folder = Ouvrir le dossier
    .meta = { $app } · { $when } · { $size }
    .meta-bare = { $when } · { $size }
    .chars = { $form ->
        [one] { $count } caractère
       *[other] { $count } caractères
    }
    .expires = Expire ce soir
    .note-too-big = Cette copie dépassait la taille maximale ; elle n’a pas été conservée.
    .note-disk = Le disque est presque plein ; les nouvelles copies ne sont pas conservées.
    .note-pinned-full = Les éléments épinglés remplissent l’historique ; les nouvelles copies ne sont pas conservées. Désépinglez-en pour faire de la place.
    .note-busy = Le Presse-papiers était occupé ; rien n’a été inséré. Réessayez.
    .note-paste-failed = Le collage n’a pas abouti ; l’insertion s’est arrêtée. Vérifiez ce qui a été inséré.
    .ticked = Coché { $count }
    .pinned = Épinglé
    .join = {", "}

## Les nombres : la marque entre la partie entière et les décimales, et l’espace fine insécable entre chaque groupe de trois chiffres (CLDR).
num =
    .decimal = {","}
    .group = {"\u202F"}

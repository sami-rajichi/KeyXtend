### Textes français du panneau Remplissage rapide (P9), chargés dans le même paquet que fr.ftl ; les trois fichiers ont les mêmes clés.
### Les formes du pluriel se choisissent selon $form, c’est-à-dire $count lu par les règles CLDR ; [0] est le nombre 0 exact.

## Les mots du panneau, puis les filtres, types et champs du tableau par identifiant, les boutons de ligne, de champ et de panneau,
## les pages Nouveau, Modifier et Sauvegarde, la page du mot de passe selon sa demande, et les notes ; dates et heures sont celles du Presse-papiers.
fill =
    .title = Remplissage rapide
    .search = Tapez pour rechercher
    .empty = Rien n’est enregistré pour l’instant. Cliquez sur Nouveau pour enregistrer votre premier élément.
    .none = Aucun élément ne correspond.
    .mask = ••••••••
    .join = {" · "}
    .fields = { $form ->
        [0] { $kind } · aucun champ
        [one] { $kind } · { $count } champ
       *[other] { $kind } · { $count } champs
    }
    .values = { $form ->
        [one] { $count } valeur
       *[other] { $count } valeurs
    }
    .waiting = Sauvegarde en attente de { $place }
    .filter-all = Tout
    .filter-people = Personnes
    .filter-logins = Comptes
    .filter-wifi = Wi-Fi
    .filter-cards = Cartes
    .filter-notes = Notes
    .filter-labels = Par champ
    .kind-person = Personne
    .kind-login = Compte
    .kind-wifi = Wi-Fi
    .kind-card = Carte
    .kind-note = Note
    .kind-other = Autre
    .label-full-name = Nom complet
    .label-email = E-mail
    .label-phone = Téléphone
    .label-address = Adresse
    .label-city = Ville
    .label-postcode = Code postal
    .label-country = Pays
    .label-birthday = Date de naissance
    .label-user = Identifiant
    .label-password = Mot de passe
    .label-pin = Code PIN
    .label-id-number = Numéro d’identité
    .label-number = Numéro
    .label-website = Site web
    .label-network = Réseau
    .label-text = Texte
    .user = Identifiant
    .password = Mot de passe
    .both = Les deux
    .insert = Insérer
    .open = Ouvrir
    .remove = Retirer
    .pick = Choisir
    .add = Ajouter
    .value = Valeur
    .label = Nom du champ
    .show = Afficher
    .hide = Masquer
    .hidden = Masqué
    .paste = Coller
    .make = Générer
    .new = Nouveau
    .backup = Sauvegarde
    .close = Fermer
    .back = Retour
    .edit = Modifier
    .name = Nom
    .add-field = Ajouter un champ
    .own-label = Nom de votre choix
    .save = Enregistrer
    .cancel = Annuler
    .delete = Supprimer l’élément
    .back-up-now = Sauvegarder maintenant
    .add-place = Ajouter un emplacement
    .change-password = Changer le mot de passe
    .bring-back = Restaurer
    .import = Importer depuis KeePass
    .other-folder = Un autre dossier
    .here = Utiliser ce dossier
    .done = Terminé
    .new-title = Nouvel élément
    .edit-title = Modifier l’élément
    .kinds = Que voulez-vous enregistrer ?
    .confirm = Supprimer cet élément ? Il ne pourra pas être récupéré.
    .checks = Tests de mémorisation
    .no-places = Aucune sauvegarde pour l’instant. Ajoutez un emplacement pour garder une copie verrouillée de vos éléments.
    .updated-today = Sauvegardé aujourd’hui à { $time }
    .updated-yesterday = Sauvegardé hier à { $time }
    .updated-date = Sauvegardé le { $date }
    .place-waiting = En attente
    .remove-place = Retirer { $place } ? Le fichier de sauvegarde qui s’y trouve est conservé.
    .replace = Remplacer la sauvegarde ? Elle contient peut-être les éléments que cet ordinateur n’a pas pu lire.
    .places = Choisissez où garder votre sauvegarde.
    .folders-folder = Choisissez un dossier pour votre sauvegarde.
    .folders-backup = Trouvez votre fichier de sauvegarde.
    .folders-keepass = Choisissez un fichier KeePass.
    .drives = Ce PC
    .password-new = { $form ->
        [one] Choisissez un mot de passe de sauvegarde d’au moins { $count } caractère. Si vous l’oubliez, la sauvegarde ne pourra pas être ouverte.
       *[other] Choisissez un mot de passe de sauvegarde d’au moins { $count } caractères. Si vous l’oubliez, la sauvegarde ne pourra pas être ouverte.
    }
    .password-check = { $form ->
        [one] Tapez votre mot de passe de sauvegarde pour vérifier que vous vous en souvenez. Il reste { $count } essai.
       *[other] Tapez votre mot de passe de sauvegarde pour vérifier que vous vous en souvenez. Il reste { $count } essais.
    }
    .password-open = Tapez le mot de passe de cette sauvegarde pour restaurer vos éléments.
    .password-own = Tapez le mot de passe de cette sauvegarde pour restaurer vos éléments. La sauvegarde sera ensuite réenregistrée avec votre mot de passe de sauvegarde actuel.
    .password-foreign = Tapez le mot de passe de la sauvegarde de l’autre ordinateur.
    .password-keepass = Tapez le mot de passe du fichier KeePass.
    .found = Votre sauvegarde du { $date } a été trouvée sur { $place }. La restaurer ?
    .foreign = Une sauvegarde d’un autre ordinateur, datée du { $date }, se trouve dans { $place }. Importer ses éléments ?
    .foreign-undated = Une sauvegarde d’un autre ordinateur se trouve dans { $place }. Importer ses éléments ?
    .note-typed = Saisi dans l’application au premier plan.
    .note-not-finished = Le remplissage ne s’est pas terminé. Vérifiez le texte saisi.
    .note-paste-empty = Le Presse-papiers ne contient aucun texte à coller.
    .note-paste-off = Pour coller, l’historique du Presse-papiers doit être activé.
    .note-paste-busy = Le Presse-papiers est occupé ; réessayez.
    .note-wrong-password = Ce mot de passe ne permet pas de l’ouvrir. Réessayez.
    .note-damaged = Ce fichier est endommagé ou n’est pas du bon type.
    .note-unsupported = Cette version ne sait pas lire ce fichier.
    .note-too-short = Ce mot de passe est trop court.
    .note-refused = Ce texte est trop long ou contient une marque invisible ; il n’a pas été accepté.
    .note-full = Il n’y a plus de place pour d’autres éléments, ou pour d’autres champs dans cet élément.
    .note-imported = { $form ->
        [one] { $count } élément importé. Éléments ou valeurs ignorés : { $skipped }.
       *[other] { $count } éléments importés. Éléments ou valeurs ignorés : { $skipped }.
    }
    .note-remembered = C’est bien : vous vous souvenez toujours de votre mot de passe de sauvegarde.
    .note-tries-used = C’était le dernier essai. Choisissez un nouveau mot de passe de sauvegarde ; chaque sauvegarde sera ensuite enregistrée de nouveau avec celui-ci.
    .note-unread = La sauvegarde n’a pas pu être lue. Vérifiez que son emplacement est disponible.
    .note-file-locked = Ce mot de passe n’ouvre pas ce fichier ; un fichier qui exige aussi un fichier clé ne peut pas être ouvert ici.
    .note-file-too-large = Ce fichier est trop volumineux pour être importé.
    .note-file-unread = Ce fichier n’a pas pu être lu. Vérifiez qu’il est toujours là, puis réessayez.
    .note-data-folder = Une sauvegarde à cet endroit disparaîtrait avec les données de { -app } si ce PC était effacé. Choisissez un autre dossier.
    .note-read-only = Impossible d’écrire à cet emplacement. Choisissez-en un autre.
    .note-too-large = La sauvegarde est trop volumineuse.
    .note-box-failed = La sauvegarde n’a pas pu être enregistrée ; une nouvelle tentative aura lieu.
    .note-paused = Sauvegarde en pause : vos éléments n’ont pas pu être lus. Restaurez-les ou remplacez la sauvegarde.
    .note-save-failed = Cette modification n’a pas pu être enregistrée ; une nouvelle tentative aura lieu. D’ici là, elle peut être perdue si { -app } se ferme.

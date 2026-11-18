### Textes français des services cloud de la dictée (P11, ADR-0047), chargés dans le même paquet que fr.ftl.
### Les formes du pluriel se choisissent selon $form, ou selon le $<nom>-form d’une information ; [0] est le nombre 0.

## Dans l’ordre : la page Services cloud et ses boutons, la fiche d’un service, ses informations par groupe et identifiant
## (remplies avec les chiffres de la table des services), les montants, la page de la clé, la barre et ses notes,
## l’avertissement de Comparer, puis le côté Cloud de la page Modèles. $https est le début de toute adresse saisie.
cloud =
    .sub = Votre propre clé · rien ne part avant l’arrêt
    .switch = Utiliser les services cloud
    .off-line = Services cloud désactivés : aucun n’est proposé, rien n’est envoyé.
    .free = Usage gratuit : { $free }
    .custom = Adresse personnalisée
    .custom-line = Une adresse HTTPS, un modèle et une clé à saisir
    .home = Activés · { $ready }
    .ready = { $form ->
        [0] Aucun service prêt
        [one] { $count } service prêt
       *[other] { $count } services prêts
    }
    .on = Activer
    .off = Désactiver
    .facts = Informations
    .lines = Clé et modèle
    .card-sub = À lire avant d’activer
    .card-on = Activé le { $date }
    .card-where = Où va votre voix
    .card-trains = Entraînement
    .card-keeps = Conservation
    .card-free = Usage gratuit
    .card-light = Usage léger · { $hours }
    .card-heavy = Usage intensif · { $hours }
    .card-bills = Facturation minimale
    .card-hours = { $form ->
        [one] { $count } heure par mois
       *[other] { $count } heures par mois
    }
    .card-read-on = Informations relevées le { $date }.
    .card-old = Elles datent peut-être ; vérifiez la page du service.
    .card-link = Ouvrir { $site }
    .card-confirm = Confirmer
    .card-ask = Votre voix ne part vers { $name } qu’après l’arrêt, avec votre propre clé.
    .card-custom = l’adresse que vous saisissez
    .card-untested = Non testé : pas encore vérifié avec une vraie clé.
    .fact-where-us = États-Unis
    .fact-where-typed = L’adresse que vous saisissez
    .fact-trains-never = Non utilisée pour l’entraînement
    .fact-trains-unknown = Inconnu ; cela dépend du service
    .fact-keeps-logs = Rien par défaut, mais des contrôles peuvent garder les requêtes jusqu’à { $days-form ->
        [one] { $days } jour
       *[other] { $days } jours
    } ; « { $switch } » l’empêche
    .fact-keeps-unknown = Inconnu ; cela dépend du service
    .fact-free-daily = { $hours-form ->
        [one] { $hours } heure
       *[other] { $hours } heures
    } par jour, { $per_min-form ->
        [one] { $per_min } requête
       *[other] { $per_min } requêtes
    } par minute
    .fact-free-unknown = Inconnu ; cela dépend du service
    .fact-price-free-or = Gratuit dans la limite de l’usage gratuit, sinon de { $low } à { $high } selon le modèle
    .fact-price-unknown = Inconnu ; cela dépend du service
    .fact-warn-trains = Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez « { $switch } » dans votre compte.
    .fact-bills-least = Au moins { $seconds-form ->
        [one] { $seconds } seconde
       *[other] { $seconds } secondes
    } par requête ; les morceaux courts coûtent donc plus
    .money-usd = { $amount } $
    .money = { $amount } { $code }
    .key-line = Clé
    .key-hint = Tapez ou collez votre clé
    .key-sealed = La clé est chiffrée sur ce PC et jamais affichée.
    .key-link = Créer une clé sur { $site }
    .key-paste = Coller
    .key-test = Tester
    .key-remove = Retirer la clé
    .key-testing = Test en cours…
    .key-ok = La clé fonctionne
    .key-bad = Clé refusée. Vérifiez-la sur la page du service.
    .key-failed = Aucune réponse sur la clé. Vérifiez le réseau et testez de nouveau.
    .key-unfit = Ce texte n’est pas une clé utilisable ; rien n’a été gardé.
    .key-no-test = Pas de test possible ici : la première dictée testera la clé.
    .key-busy = Le presse-papiers n’a pas répondu. Réessayez « Coller ».
    .address-line = Adresse
    .address-hint = L’adresse de votre service, par exemple { $https }example.com/v1/audio/transcriptions
    .address-hint-own = L’adresse de votre propre ressource chez ce service, qui commence par { $https }
    .address-https = L’adresse doit commencer par { $https }
    .address-host = L’adresse doit contenir un nom d’hôte, pas un numéro.
    .address-domain = L’adresse doit appartenir au domaine du service.
    .address-long = L’adresse est trop longue.
    .address-path = L’adresse contient un caractère interdit, comme un espace.
    .model-line = Autre modèle
    .model-hint = L’identifiant du modèle, sur la page du service
    .model-refused = Cet identifiant est trop long ou contient un caractère interdit.
    .keep-nothing = Seulement des hébergeurs qui ne gardent rien
    .on-bar = Votre voix partira chez { $name } à l’arrêt
    .use-local = Utiliser le modèle local
    .send-warning = Comparer enverra votre voix à { $names } à l’arrêt.
    .note-net = Le service n’a pas répondu. Vérifiez le réseau et réessayez.
    .note-key = Le service a refusé la clé. Vérifiez-la sur la page de la clé.
    .note-credit = Le compte n’a plus de crédit.
    .note-allowance = L’usage gratuit est épuisé pour l’instant.
    .note-allowance-in = L’usage gratuit est épuisé. Il se renouvelle dans { $in }.
    .note-rate = Trop de requêtes. Patientez un instant et réessayez.
    .note-too-big = L’enregistrement était trop gros pour le service.
    .note-language = Le service n’écrit pas en { $lang }.
    .note-empty = Le service n’a renvoyé aucun texte.
    .note-strange = La réponse ne venait pas du service, peut-être une page de connexion. Vérifiez le réseau.
    .note-terms = Les conditions du service ont changé. Confirmez de nouveau ses informations pour envoyer.
    .note-off = Le service est désactivé ; rien n’a été envoyé.
    .note-no-local = Un champ de mot de passe ne part jamais vers le cloud, et aucun modèle local n’est prêt.
    .note-fault = La partie cloud s’est arrêtée après une erreur. Réessayez.
    .note-keep-nothing = Ce modèle n’a aucun hébergeur qui ne garde rien.
    .hours = { $count } h
    .model = Dans le cloud, avec votre clé
    .sides = Où le modèle s’exécute
    .side-local = Sur ce PC
    .side-cloud = Dans le cloud
    .band-none-on = Aucun service cloud n’est encore activé.
    .band-listed = Modèles des services que vous avez activés ; rien ne part avant l’arrêt.

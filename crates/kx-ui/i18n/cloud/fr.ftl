### Textes français des services cloud de la dictée (P11, ADR-0047), chargés dans le même paquet que fr.ftl.
### Les formes du pluriel se choisissent selon $form, ou selon le $<nom>-form d’une information ; [0] est le nombre 0.

## Dans l’ordre : la page Services cloud et ses boutons, la fiche d’un service, ses informations par groupe et identifiant
## (remplies avec les chiffres de la table des services), les montants, la page de la clé, la barre et ses notes,
## l’avertissement de Comparer, puis le côté Cloud de la page Modèles. $https est le début de toute adresse saisie.
## Chaque avertissement a sa marque courte sur la ligne du service : warn-mark-, puis l’identifiant de l’information.
cloud =
    .sub = Votre propre clé · rien ne part avant l’arrêt
    .switch = Utiliser les services cloud
    .off-line = Services cloud désactivés : aucun n’est proposé, rien n’est envoyé.
    .free = Gratuit : { $free }
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
    .card-free = Quota gratuit
    .card-light = Usage léger · { $hours }
    .card-heavy = Usage intensif · { $hours }
    .card-bills = Facturation minimale
    .card-rule = Conditions d’utilisation
    .card-hours = { $form ->
        [one] { $count } heure par mois
       *[other] { $count } heures par mois
    }
    .card-read-on = Informations relevées le { $date }.
    .card-old = Elles datent peut-être ; vérifiez la page du service.
    .card-read-old = { cloud.card-read-on } { cloud.card-old }
    .card-link = Ouvrir { $site }
    .card-confirm = Confirmer
    .card-ask = Votre voix ne part vers { $name } qu’après l’arrêt, avec votre propre clé.
    .card-custom = l’adresse que vous saisissez
    .card-untested = Non testé : pas encore vérifié avec une vraie clé.
    .fact-where-us = États-Unis
    .fact-where-typed = L’adresse que vous saisissez
    .fact-where-eu = Union européenne
    .fact-where-unstated = Non précisé sur ses pages
    .fact-where-host = Dépend de l’hébergeur qui sert le modèle
    .fact-trains-never = Non utilisée pour l’entraînement
    .fact-trains-unknown = Inconnu ; cela dépend du service
    .fact-trains-opt-in = Seulement si vous l’acceptez
    .fact-trains-switch = Non précisé ; ce réglage du compte l’arrête : { $switch }
    .fact-trains-unstated = Non précisé sur ses pages
    .fact-trains-host = Dépend de l’hébergeur qui sert le modèle
    .fact-trains-default = Utilisée pour l’entraînement par défaut
    .fact-trains-free-allowance = Utilisée pour l’entraînement tant que vous utilisez { -cloud-free-use }
    .fact-keeps-logs = Rien par défaut, mais les requêtes peuvent être gardées jusqu’à { $days-form ->
        [one] { $days } jour
       *[other] { $days } jours
    } pour contrôle ; ce réglage du compte l’empêche : { $switch }
    .fact-keeps-unknown = Inconnu ; cela dépend du service
    .fact-keeps-none = Rien
    .fact-keeps-on-request = Non précisé ; ne rien garder n’est possible que sur demande
    .fact-keeps-stored = Vos requêtes, par défaut ; ce réglage du compte l’arrête : { $switch }
    .fact-keeps-deleted = Rien : tout est supprimé juste après le traitement
    .fact-keeps-memory = Rien : gardé en mémoire le temps du traitement seulement
    .fact-keeps-host = Dépend de l’hébergeur ; un réglage, activé d’emblée, n’envoie qu’aux hébergeurs qui ne gardent rien
    .fact-keeps-history = Votre historique, par défaut ; ne rien garder est réservé aux offres Enterprise
    .fact-keeps-paid-limited = Au { -cloud-paid-tier } : vos requêtes, pendant un temps limité pour détecter les abus
    .fact-keeps-unstated = { cloud.fact-where-unstated }
    .fact-free-daily = { $hours-form ->
        [one] { $hours } heure
       *[other] { $hours } heures
    } par jour, { $per_min-form ->
        [one] { $per_min } requête
       *[other] { $per_min } requêtes
    } par minute
    .fact-free-unknown = Inconnu ; cela dépend du service
    .fact-free-none = Aucun
    .fact-free-first-buy = Aucun ; premier achat d’au moins { $least }
    .fact-free-first-month = Le premier mois
    .fact-free-small = Un petit quota pour les nouveaux comptes
    .fact-free-minutes-month = Environ { $minutes } { $minutes-form ->
        [one] minute
       *[other] minutes
    } par mois
    .fact-free-no-card = Sans carte bancaire
    .fact-free-credit = { $credit } de crédit sans expiration, sans carte
    .fact-price-free-or = Gratuit dans la limite du quota gratuit, sinon de { $low } à { $high } selon le modèle
    .fact-price-unknown = Inconnu ; cela dépend du service
    .fact-price-plain = { $price }
    .fact-price-paid-plain = { $price } au { -cloud-paid-tier }
    .fact-price-range = De { $low } à { $high } selon le modèle
    .fact-price-range-fee = De { $low } à { $high } selon le modèle, plus { $fee } % de frais sur le crédit
    .fact-warn-trains = { -cloud-trains } Pour arrêter cela, désactivez ce réglage : { $switch }
    .fact-warn-trains-profile = { -cloud-trains } Pour arrêter cela, désactivez ce réglage dans le menu de votre profil : { $switch }
    .warn-mark-trains = Entraînement activé
    .warn-mark-trains-profile = { cloud.warn-mark-trains }
    .fact-warn-free-allowance = Tant que vous utilisez { -cloud-free-use }, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites.
    .warn-mark-free-allowance = Entraînement avec { -cloud-free-use }
    .fact-bills-least = Au moins { $seconds-form ->
        [one] { $seconds } seconde
       *[other] { $seconds } secondes
    } par requête ; les morceaux courts coûtent donc plus cher
    .fact-rule-non-commercial = L’offre gratuite est réservée à un usage non commercial, avec mention de la source
    .fact-rule-eea-paid-18 = Usage payant seulement dans l’EEE, en Suisse et au Royaume-Uni ; utilisateurs de 18 ans ou plus
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
    .key-unfit = Ce texte ne peut pas être envoyé comme clé ; rien n’a donc été gardé.
    .key-no-test = Pas de test possible ici : la première dictée testera la clé.
    .key-busy = Le presse-papiers n’a pas répondu. Réessayez « Coller ».
    .address-line = Adresse
    .address-hint = L’adresse de votre service, par exemple { $https }example.com/v1/audio/transcriptions
    .address-hint-own = L’adresse HTTPS de votre propre ressource chez ce service
    .address-https = L’adresse doit utiliser HTTPS, par exemple { $https }example.com
    .address-host = L’adresse doit contenir un nom d’hôte, pas une adresse IP.
    .address-port = Le port de l’adresse n’est pas valide.
    .address-domain = L’adresse doit appartenir au domaine du service.
    .address-long = L’adresse est trop longue.
    .address-path = L’adresse contient un caractère non autorisé, comme un espace.
    .model-line = Autre modèle
    .model-hint = L’identifiant du modèle, sur la page du service
    .model-refused = Cet identifiant de modèle est trop long ou contient un caractère non autorisé.
    .line-save = Enregistrer
    .line-save-key = Enregistrer la clé
    .line-save-address = Enregistrer l’adresse
    .line-save-model = Enregistrer le modèle
    .keep-nothing = Seulement des hébergeurs qui ne gardent rien
    .on-bar = Votre voix partira vers { $name } à l’arrêt
    .use-local = Utiliser le modèle local
    .send-warning = Comparer enverra votre voix à { $names } à l’arrêt.
    .note-net = Le service n’a pas répondu. Vérifiez le réseau et réessayez.
    .note-key = Le service a refusé la clé. Vérifiez-la sur la page de la clé.
    .note-credit = Le compte n’a plus de crédit.
    .note-account = Le service demande une étape sur son site, comme une vérification, un moyen de paiement ou un plafond de dépenses.
    .note-region = Le service n’est pas proposé dans votre pays ou votre région.
    .note-allowance = Le quota gratuit est épuisé pour l’instant.
    .note-allowance-at = Le quota gratuit est épuisé. Il se renouvelle à { $at }.
    .note-allowance-on = Le quota gratuit est épuisé. Il se renouvelle le { $date } à { $at }.
    .note-rate = Trop de requêtes. Patientez un instant et réessayez.
    .note-too-big = L’enregistrement était trop gros pour le service.
    .note-language = Le service n’écrit pas en { $lang }.
    .note-empty = Le service n’a renvoyé aucun texte.
    .note-strange = La réponse ne venait pas du service, peut-être une page de connexion. Vérifiez le réseau.
    .note-terms = Les conditions du service ont changé. Confirmez de nouveau ses informations pour envoyer.
    .note-off = Le service est désactivé ; rien n’a été envoyé.
    .note-moved = La langue n’utilise plus ce service ; rien n’a été envoyé.
    .note-no-local = Ce qui est dicté dans un champ de mot de passe ne part jamais vers le cloud, et aucun modèle local n’est prêt.
    .note-fault = La partie cloud s’est arrêtée après une erreur. Réessayez.
    .note-keep-nothing = Aucun hébergeur ne s’engage à ne rien garder.
    .model = Dans le cloud, avec votre clé
    .sides = Où le modèle s’exécute
    .side-local = Sur ce PC
    .side-cloud = Dans le cloud
    .band-none-on = Aucun service cloud n’est encore activé.
    .band-listed = Modèles des services que vous avez activés ; rien ne part avant l’arrêt.

# La première phrase de chaque avertissement d’entraînement, le commun comme celui propre à un service.
-cloud-trains = Entraîne ses modèles sur votre voix par défaut.
# Le seul nom du quota gratuit dans le texte courant ; un titre, un début de phrase et « du quota gratuit » l’écrivent en entier.
-cloud-free-use = le quota gratuit
# Le seul nom du niveau payant, comme les pages de Google le nomment, sans article, dans les lignes sur l’usage payant.
-cloud-paid-tier = niveau payant

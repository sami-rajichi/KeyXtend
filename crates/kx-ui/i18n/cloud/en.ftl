### English texts of voice typing's cloud services (P11, ADR-0047), loaded into the same bundle as en.ftl.
### Plural forms select on $form, or on a fact's $<name>-form; [0] is the exact count 0.

## In order: the Cloud page and its buttons, a service's card, its facts by group and id (filled with the services
## table's numbers), money, the key page, the bar and its notes, Compare's warning, and the Models page's Cloud side.
## $https is the scheme every typed address starts with; a fact's own names from a service's pages come in as they are.
## Each warning fact has its row's short mark: warn-mark-, then the fact's id.
cloud =
    .sub = Your own key · nothing leaves before Stop
    .switch = Use cloud services
    .off-line = Cloud services are off: none is offered and nothing is sent.
    .free = Free: { $free }
    .custom = Custom address
    .custom-line = An HTTPS address, a model and a key you type
    .home = On · { $ready }
    .ready = { $form ->
        [0] No service ready
        [one] { $count } service ready
       *[other] { $count } services ready
    }
    .on = Switch on
    .off = Switch off
    .facts = Facts
    .lines = Key and model
    .card-sub = Read this before you switch it on
    .card-on = Switched on { $date }
    .card-where = Where your voice goes
    .card-trains = Training
    .card-keeps = What is kept
    .card-free = Free allowance
    .card-light = Light use · { $hours }
    .card-heavy = Heavy use · { $hours }
    .card-bills = Least billed
    .card-rule = Terms of use
    .card-hours = { $form ->
        [one] { $count } hour a month
       *[other] { $count } hours a month
    }
    .card-read-on = Facts read on { $date }.
    .card-old = They may be out of date; check the service’s page.
    .card-link = Open { $site }
    .card-confirm = Confirm
    .card-ask = Your voice goes to { $name } only after Stop, with your own key.
    .card-custom = the address you type
    .card-untested = Untested: not yet checked with a real key.
    .fact-where-us = United States
    .fact-where-typed = The address you type
    .fact-where-eu = European Union
    .fact-where-unstated = Not stated on its pages
    .fact-where-host = Depends on the host that serves the model
    .fact-trains-never = Not used for training
    .fact-trains-unknown = Unknown; it depends on the service
    .fact-trains-opt-in = Only if you opt in
    .fact-trains-switch = Not stated; this account switch stops it: { $switch }
    .fact-trains-unstated = Not stated on its pages
    .fact-trains-host = Depends on the host that serves the model
    .fact-trains-default = Used for training by default
    .fact-keeps-logs = Nothing by default, but checks may keep requests up to { $days-form ->
        [one] { $days } day
       *[other] { $days } days
    }; this account switch turns that off: { $switch }
    .fact-keeps-unknown = Unknown; it depends on the service
    .fact-keeps-none = Nothing
    .fact-keeps-on-request = Not stated; keeping nothing is only on request
    .fact-keeps-stored = Your requests, by default; this account switch stops that: { $switch }
    .fact-keeps-deleted = Nothing: deleted right after processing
    .fact-keeps-memory = Nothing: held in memory only while it works
    .fact-keeps-host = Depends on the host; a switch, on to start, sends only to hosts that keep nothing
    .fact-keeps-history = Your history, by default; keeping nothing is for Enterprise plans only
    .fact-free-daily = { $hours-form ->
        [one] { $hours } hour
       *[other] { $hours } hours
    } a day, { $per_min-form ->
        [one] { $per_min } request
       *[other] { $per_min } requests
    } a minute
    .fact-free-unknown = Unknown; it depends on the service
    .fact-free-none = None
    .fact-free-first-buy = None; first purchase at least { $least }
    .fact-free-first-month = The first month
    .fact-free-small = A small allowance for new accounts
    .fact-free-minutes-month = About { $minutes } { $minutes-form ->
        [one] minute
       *[other] minutes
    } a month
    .fact-price-free-or = Free within the free allowance, else { $low } to { $high } by model
    .fact-price-unknown = Unknown; it depends on the service
    .fact-price-plain = { $price }
    .fact-price-range = { $low } to { $high } by model
    .fact-price-range-fee = { $low } to { $high } by model, plus a { $fee }% fee on credit
    .fact-warn-trains = Trains its models on your voice by default. Before you confirm, turn off this switch in your account: { $switch }
    .warn-mark-trains = Trains by default
    .fact-bills-least = At least { $seconds-form ->
        [one] { $seconds } second
       *[other] { $seconds } seconds
    } a request, so short pieces cost more
    .fact-rule-non-commercial = The free plan is for non-commercial use only, with attribution
    .money-usd = ${ $amount }
    .money = { $amount } { $code }
    .key-line = Key
    .key-hint = Type or paste your key
    .key-sealed = The key is kept encrypted on this PC and never shown.
    .key-link = Make a key at { $site }
    .key-paste = Paste
    .key-test = Test
    .key-remove = Remove key
    .key-testing = Testing…
    .key-ok = Key works
    .key-bad = Key refused. Check it on the service’s page.
    .key-failed = No answer about the key. Check the network and test again.
    .key-unfit = This text cannot be sent as a key, so nothing was kept.
    .key-no-test = No test call here: the first dictation tests the key.
    .key-busy = The clipboard did not answer. Try Paste again.
    .address-line = Address
    .address-hint = Your service’s address, such as { $https }example.com/v1/audio/transcriptions
    .address-hint-own = The HTTPS address of your own resource on this service
    .address-https = The address must use HTTPS, such as { $https }example.com
    .address-host = The address needs a host name, not an IP address.
    .address-domain = The address must be under the service’s own domain.
    .address-long = The address is too long.
    .address-path = The address holds a character that is not allowed, such as a space.
    .model-line = Other model
    .model-hint = A model id from the service’s page
    .model-refused = This model id is too long or holds a character that is not allowed.
    .line-save = Save
    .line-save-key = Save the key
    .line-save-address = Save the address
    .line-save-model = Save the model
    .keep-nothing = Only hosts that keep nothing
    .on-bar = Your voice goes to { $name } at Stop
    .use-local = Use the local model
    .send-warning = Compare sends your voice to { $names } at Stop.
    .note-net = The service did not answer. Check the network and try again.
    .note-key = The service refused the key. Check it on the key page.
    .note-credit = The account has no credit left.
    .note-region = The service is not offered in your country or region.
    .note-allowance = The free allowance is used up for now.
    .note-allowance-in = The free allowance is used up. It renews in { $in }.
    .note-rate = Too many requests. Wait a moment and try again.
    .note-too-big = The recording was too big for the service.
    .note-language = The service does not take { $lang }.
    .note-empty = The service sent no text.
    .note-strange = The reply was not the service’s, maybe a sign-in page. Check the network.
    .note-terms = The service’s terms changed. Confirm its facts again to send.
    .note-off = The service is off, so nothing was sent.
    .note-no-local = Speech for a password box never goes to the cloud, and no local model is ready.
    .note-fault = The cloud part stopped after an error. Try again.
    .note-keep-nothing = No host of this model promises to keep nothing.
    .hours = { $count } h
    .model = In the cloud, with your key
    .sides = Where the model runs
    .side-local = Local
    .side-cloud = Cloud
    .band-none-on = No cloud service is switched on yet.
    .band-listed = Models of the services you switched on; nothing leaves before Stop.

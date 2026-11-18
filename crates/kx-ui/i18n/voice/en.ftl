### English texts of voice typing (P11), loaded into the same bundle as en.ftl; all three files hold the same keys.
### Plural forms select on $form, which is the count as the CLDR rules read it; [0] is the exact count 0.

## In order: the bar (name, gear, states, buttons, the hint while nothing is heard yet, the password dots, the timer),
## each note by hint id, the panel's header and home list, Models, Device, Compare models, History, then Options (each
## option by its setting key, hyphens for underscores), the bar's places and units. $lang is the language's own name.
voice =
    .title = Dictation
    .gear = Dictation settings
    .listening = Listening…
    .transcribing = Transcribing…
    .preparing = Dictation, preparing the microphone
    .nothing = Nothing heard yet
    .stop = Stop
    .cancel = Cancel
    .close = Close
    .copy = Copy
    .copied = Copied
    .undo = Undo insert
    .again = Try again
    .anyway = Transcribe anyway
    .listen = Listen again
    .download = Download
    .download-size = Download · { $size }
    .mask = ••••••••
    .timer = { $minutes }:{ $seconds }
    .note-nothing = Nothing heard. Is the microphone on?
    .note-cancelled = Cancelled. Nothing was typed.
    .note-no-mic = No microphone was found.
    .note-mic-busy = Another app is using the microphone.
    .note-mic-refused = The privacy settings block the microphone.
    .note-unplugged = The microphone was unplugged.
    .note-no-model = No model for { $lang } yet
    .note-language = The chosen model does not know { $lang }. Pick another in the settings.
    .note-checking = The chosen file is still being checked. Try again in a moment.
    .note-no-language = No listed model knows { $lang }.
    .note-no-fit = Not enough free memory for this model. Close some apps or pick a smaller one.
    .note-fault = Dictation stopped after an error. Try again.
    .note-pieces = Dictation stopped part way. This is the text written so far.
    .note-too-long = Stopped at the longest dictation, and the text was typed.
    .note-no-field = Not all of the text was typed. Copy it from here or from History.
    .note-stopped = Typing stopped part way. Copy the whole text from here or from History.
    .note-copy-failed = The text could not be copied. Try again.
    .note-password = Typed into a password box. Not kept, no Copy.
    .note-fell = The graphics card failed, so the processor wrote this text.
    .note-print = A download did not pass its check, so it was deleted.
    .note-taken = Another file in the models folder has this model’s name, so nothing was downloaded.
    .note-full = Not enough room on the drive, so nothing was downloaded.
    .note-folder = The models folder cannot be used. Change it on the Models page.
    .note-net = The download did not finish. Try again later.
    .back = Back
    .lang = Language: { $lang }
    .models = Models
    .cloud = Cloud services
    .device = Device
    .compare = Compare models
    .history = History
    .options = Options
    .no-model = No model ready
    .auto-cpu = Automatic (processor)
    .auto-on = Automatic ({ $device })
    .last-run = Last run: { $date }
    .never = Not run yet
    .texts = { $form ->
        [one] { $count } text
       *[other] { $count } texts
    }
    .defaults = All at defaults
    .changed = Changed: { $count }
    .again-hint = Transcribes the last recording again with the choices here.
    .for-lang = For { $lang }
    .best = Recommended for { $lang }:
    .untested = This order is not tested for { $lang }.
    .slow = Local models for { $lang } will be very slow on this PC.
    .mark-used = In use
    .mark-default = Default
    .mark-ready = Ready
    .mark-download = Not downloaded
    .mark-not-here = Not for this PC
    .mark-other = Other file, not checked
    .unfit = Needs { $need } of memory, { $free } free
    .got = { $done } of { $total }
    .paused = Paused · { $done } of { $total }
    .pause = Pause
    .resume = Resume
    .use = Use for this language
    .delete-file = Delete file
    .folder = Models folder: { $path } · { $free } free
    .change = Change
    .open = Open
    .folder-offer = Keep models in { $path }? { $free } free.
    .folders = Models folder
    .drives = Drives
    .here = Use this folder
    .device-hint = Cards are checked before use
    .auto = Automatic
    .auto-line = The processor until a card passes its check
    .cpu = Processor
    .cpu-line = { $cores } · { $total } memory, { $free } free
    .cores = { $form ->
        [one] { $count } core
       *[other] { $count } cores
    }
    .card-line = Graphics card · { $total }, { $free } free
    .built-in-line = Built-in graphics · { $total } shared, { $free } free
    .checking = Checking: warming up the card
    .faster = { $times }× faster
    .mark-passed = Passed · { $faster }
    .mark-failed = Check failed
    .why-slow = It was only { $faster }; { $need } needed.
    .why-differs = Its text differed by { $words }; { $allowed } allowed.
    .why-room = The card had too little free memory.
    .why-error = The card reported an error.
    .failed = { $name }: check failed. { $why }
    .check = Check
    .use-device = Use
    .intro = Read this text aloud once. Each ready model then writes it, one after another.
    .intro-free = Say a few sentences aloud once. Each ready model then writes them, one after another.
    .read-text = The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly.
    .ready = { $form ->
        [one] { $count } ready model · { $device }
       *[other] { $count } ready models · { $device }
    }
    .start = Start reading
    .recording = Read the text now, then press Stop on the bar.
    .step = Model { $at } of { $of }
    .mark-done = Done · { $wait }
    .mark-writing = Writing…
    .mark-waiting = Waiting
    .mark-run-failed = Failed
    .mark-off = Off
    .mark-needs-key = Needs a key
    .mark-needs-address = Needs an address
    .mark-on = Ready
    .mark-untested = Untested
    .mark-warn = Warning
    .mark-unavailable = Unavailable
    .col-model = Model
    .col-wrong = Words wrong
    .col-wait = Wait
    .col-memory = Memory
    .wrong-of = { $wrong } of { $words }
    .kept = Only these numbers are kept, not the recording or the texts.
    .result-line = { $date } · { $device }
    .rerun = Compare again
    .history-sub = { $texts } · newest first
    .meta = { $when } · { $lang } · { $model }
    .pin = Pin
    .unpin = Unpin
    .delete = Delete
    .delete-all = Delete all
    .insert = Insert ({ $count })
    .insert-one = Insert
    .confirm = { $form ->
        [0] No unpinned texts to delete.
        [one] Delete { $count } text? Pinned ones stay.
       *[other] Delete { $count } texts? Pinned ones stay.
    }
    .yes = Yes
    .no = No
    .empty = No texts yet. Dictate something, and it shows here.
    .ticked = Ticked { $count }
    .pinned = Pinned
    .join = {", "}
    .for-all = For every language
    .applies = Changes apply at once.
    .on = On
    .off = Off
    .minus = Less: { $name }
    .plus = More: { $name }
    .opt-sounds = Start and stop sounds
    .opt-silence-stop = Stop after silence
    .opt-silence-stop-s = Silence before stopping
    .opt-done-ms = Text stays on the bar
    .opt-place = Bar position
    .opt-history = Keep a history of texts
    .opt-history-max = Texts kept
    .opt-keep-voice = Keep recordings
    .opt-voice-max-mb = Space for recordings
    .opt-idle-min = Model leaves memory after
    .opt-record-max-s = Longest dictation
    .opt-nothing-s = “Nothing heard” hint after
    .opt-debounce-ms = Ignore a second click within
    .opt-insert = Insert the text
    .opt-type-suffix = Space after the text
    .opt-type-gap-ms = Pause between letters
    .opt-threads = Processor threads
    .opt-wave-ms = Wave smoothing
    .opt-min-free-mb = Room left on the models drive
    .opt-card-margin-mb = Room left on a card
    .opt-card-gain = A card must be faster by
    .opt-card-diff = A card’s text may differ by
    .place-above = Above the keyboard
    .place-bottom = Bottom of the screen
    .place-top = Top of the screen
    .insert-type = Type it
    .insert-paste = Paste at once
    .insert-hint = Uses the clipboard, then puts yours back; a password box is always typed.
    .seconds = { $count } s
    .minutes = { $count } min
    .ms = { $count } ms
    .words = { $form ->
        [one] { $count } word
       *[other] { $count } words
    }
    .times = { $times }×
    .secs = { $s } s
    .auto-n = Auto ({ $count })
    .mb = { $mb } MB
    .gb = { $gb } GB

## A language's name by ISO 639 code, as History names a text's language; another code shows itself.
langname =
    .en = English
    .fr = French
    .ar = Arabic

## A language's name inside a sentence, by ISO 639 code; another code keeps the system's own name.
langin =
    .en = { langname.en }
    .fr = { langname.fr }
    .ar = { langname.ar }

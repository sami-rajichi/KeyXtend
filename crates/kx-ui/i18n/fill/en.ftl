### English texts of the Quick-fill panel (P9), loaded into the same bundle as en.ftl; all three files hold the same keys.
### Plural forms select on $form, which is $count as the CLDR rules read it; [0] is the exact count 0.

## The panel's words, then filters, kinds and table labels by id, row, field and panel buttons by id, the New, Edit and
## backup pages, the password page by what it asks, and the notes by id; dates and times are the clipboard's.
fill =
    .title = Quick-fill
    .search = Type to search
    .empty = Nothing saved yet. Click New to save your first entry.
    .none = No entry matches.
    .mask = ••••••••
    .join = {" · "}
    .fields = { $form ->
        [0] { $kind } · no fields
        [one] { $kind } · { $count } field
       *[other] { $kind } · { $count } fields
    }
    .values = { $form ->
        [one] { $count } value
       *[other] { $count } values
    }
    .waiting = Backup waiting for { $place }
    .filter-all = All
    .filter-people = People
    .filter-logins = Logins
    .filter-wifi = Wi-Fi
    .filter-cards = Cards
    .filter-notes = Notes
    .filter-labels = By label
    .kind-person = Person
    .kind-login = Login
    .kind-wifi = Wi-Fi
    .kind-card = Card
    .kind-note = Note
    .kind-other = Other
    .label-full-name = Full name
    .label-email = Email
    .label-phone = Phone
    .label-address = Address
    .label-city = City
    .label-postcode = Postcode
    .label-country = Country
    .label-birthday = Birthday
    .label-user = User
    .label-password = Password
    .label-pin = PIN
    .label-id-number = ID number
    .label-number = Number
    .label-website = Website
    .label-network = Network
    .label-text = Text
    .user = User
    .password = Password
    .both = Both
    .insert = Insert
    .open = Open
    .remove = Remove
    .pick = Choose
    .add = Add
    .value = Value
    .label = Label
    .show = Show
    .hide = Hide
    .hidden = Hidden
    .paste = Paste
    .make = Make one
    .new = New
    .backup = Backup
    .close = Close
    .back = Back
    .edit = Edit
    .name = Name
    .add-field = Add a field
    .own-label = Your own label
    .save = Save
    .cancel = Cancel
    .delete = Delete entry
    .back-up-now = Back up now
    .add-place = Add place
    .change-password = Change password
    .bring-back = Bring back
    .import = Import from KeePass
    .other-folder = Another folder
    .here = Use this folder
    .done = Done
    .new-title = New entry
    .edit-title = Edit entry
    .kinds = What do you want to save?
    .confirm = Delete this entry? It cannot come back.
    .checks = Memory checks
    .no-places = No backup yet. Add a place to keep a locked copy of your entries.
    .updated-today = Updated today { $time }
    .updated-yesterday = Updated yesterday { $time }
    .updated-date = Updated { $date }
    .place-waiting = Waiting
    .remove-place = Remove { $place }? The backup file there stays.
    .replace = Replace the backup? It may hold the entries this computer could not read.
    .places = Choose where to keep your backup.
    .folders-folder = Choose a folder for your backup.
    .folders-backup = Find your backup file.
    .folders-keepass = Choose a KeePass file.
    .drives = This PC
    .password-new = { $form ->
        [one] Make a backup password of at least { $count } character. If you forget it, the backup cannot be opened.
       *[other] Make a backup password of at least { $count } characters. If you forget it, the backup cannot be opened.
    }
    .password-check = { $form ->
        [one] Type your backup password to check you still remember it. { $count } try left.
       *[other] Type your backup password to check you still remember it. { $count } tries left.
    }
    .password-open = Type this backup's password to bring your entries back.
    .password-own = Type this backup's password to bring your entries back. The backup is then saved again under your current backup password.
    .password-foreign = Type the password of the backup from the other computer.
    .password-keepass = Type the KeePass file's password.
    .found = Found your backup from { $date } on { $place }. Bring it back?
    .foreign = A backup from another computer is in { $place }, from { $date }. Bring its entries in?
    .foreign-undated = A backup from another computer is in { $place }. Bring its entries in?
    .note-typed = Typed into the app in front.
    .note-not-finished = The fill did not finish. Check what was typed.
    .note-paste-empty = The clipboard holds no text to paste.
    .note-paste-off = Paste needs Clipboard history to be on.
    .note-paste-busy = The clipboard is busy; try again.
    .note-wrong-password = That password does not open it. Try again.
    .note-damaged = That file is damaged or is not the right kind.
    .note-unsupported = This version cannot read that file.
    .note-too-short = That password is too short.
    .note-refused = That text is too long or holds a hidden mark, so it was not taken.
    .note-full = There is no room for more entries, or for more fields in this entry.
    .note-imported = { $form ->
        [one] { $count } entry came in. Entries or values left out: { $skipped }.
       *[other] { $count } entries came in. Entries or values left out: { $skipped }.
    }
    .note-remembered = Right: you still remember your backup password.
    .note-tries-used = That was the last try. Make a new backup password; every backup is then saved again under it.
    .note-unread = The backup could not be read. Check that its place is there.
    .note-file-locked = That password does not open this file; a file that also needs a key file cannot be opened here.
    .note-file-too-large = That file is too big to bring in.
    .note-file-unread = That file could not be read. Check that it is still there, then try again.
    .note-data-folder = A backup there would be lost with { -app }'s own data if this PC is wiped. Choose another folder.
    .note-read-only = That place cannot be written. Choose another.
    .note-too-large = The backup is too big.
    .note-box-failed = The backup could not be written; it will try again.
    .note-paused = Backup paused: your entries could not be read. Bring back, or replace the backup.
    .note-save-failed = This change could not be saved; it will try again. Until then, it could be lost if { -app } closes.

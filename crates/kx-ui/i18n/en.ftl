### English texts, the fallback for every other language (ADR-0017); all three files hold the same keys.
### A key `group.name` in code is message `group`, attribute `name`.

## The product's name, written once and shared by the texts below.
-app = KeyXtend

## Named keys by board.toml id, the shortcuts layer's Next key and the page readers hear after it ($page of $pages), then .empty for a character key that types nothing now: the name that tips and screen readers use.
key =
    .esc = Escape
    .bksp = Backspace
    .tab = Tab
    .del = Delete
    .caps = Caps Lock
    .enter = Enter
    .shift = Shift
    .rshift = Right Shift
    .up = Up arrow
    .menu = Menu key
    .ctrl = Ctrl
    .win = Windows key
    .alt = Alt
    .lang = Language
    .space = Space
    .altgr = Alt Gr
    .left = Left arrow
    .down = Down arrow
    .right = Right arrow
    .rclick = Right click by holding
    .grab = Grab: drag and drop, select text
    .shortcuts = Shortcuts
    .mic = Voice typing
    .clip = Clipboard history
    .snip = Screen snip
    .fill = Quick fill
    .settings = Settings
    .power = Power
    .dpad = Scroll pad
    .layer-next = Next page
    .layer-page = Page { $page } of { $pages }
    .empty = Empty key

## The short word on a key, or under its icon at large sizes, as real keyboards print it.
label =
    .esc = Esc
    .bksp = Back
    .tab = Tab
    .del = Del
    .caps = Caps
    .enter = Enter
    .shift = Shift
    .rshift = Shift
    .up = Up
    .menu = Menu
    .ctrl = Ctrl
    .win = Win
    .alt = Alt
    .altgr = AltGr
    .left = Left
    .down = Down
    .right = Right
    .rclick = Right click
    .grab = Grab
    .shortcuts = Shortcuts
    .mic = Voice
    .clip = Clipboard
    .snip = Snip
    .fill = Fill
    .settings = Settings
    .power = Power

## What a key says after its name; .keys is a shortcut's tip, its $name and then the $keys it sends.
tip =
    .keys = { $name } ({ $keys })
    .later = Coming in a later version
    .one-lang = Only one keyboard language is installed
    .mouse-off = Mouse assist is off
    .scroll-off = The scroll pad is off
    .no-shortcuts = No shortcuts are chosen
    .power-off = The power key is off
    .rclick = Hold still to right-click; short clicks stay normal
    .grab = Press to pick up, move, then click to drop

## The shortcuts' names for tips and screen readers, by id.
shortcut =
    .copy = Copy
    .paste = Paste
    .cut = Cut
    .select-all = Select all
    .undo = Undo
    .redo = Redo
    .copy-all = Copy all
    .cut-all = Cut all
    .delete-all = Delete all
    .find = Find
    .save = Save
    .print = Print
    .emoji = Emoji
    .switch-app = Switch app
    .show-desktop = Show desktop
    .file-explorer = File Explorer
    .lock-pc = Lock PC
    .task-manager = Task Manager
    .close-window = Close window
    .new-tab = New tab
    .close-tab = Close tab
    .reopen-tab = Reopen closed tab
    .refresh = Refresh
    .zoom-in = Zoom in
    .zoom-out = Zoom out
    .f1 = F1
    .f2 = F2
    .f3 = F3
    .f4 = F4
    .f5 = F5
    .f6 = F6
    .f7 = F7
    .f8 = F8
    .f9 = F9
    .f10 = F10
    .f11 = F11
    .f12 = F12

## Keys a shortcut's tip names when no key label does, as keyboards print them; letters, digits and F1 to F24 show as themselves.
keyname =
    .period = .
    .numplus = Num +
    .numminus = Num -

## A shortcut's tip: what joins the keys pressed together, then what joins its steps.
combo =
    .join = +
    .then = {", "}

## Arabic marks by name, which screen readers say for a key that types them; .join links marks typed together.
mark =
    .fathatan = Fathatan
    .dammatan = Dammatan
    .kasratan = Kasratan
    .fatha = Fatha
    .damma = Damma
    .kasra = Kasra
    .shadda = Shadda
    .sukun = Sukun
    .join = {" with "}

## Top-bar buttons, the resize corner and the bubble, by icons.toml [bar] id.
bar =
    .grip = Move the keyboard: click, move, click
    .mode = Switch to dark mode
    .mode_on = Switch to light mode
    .theme = Change theme
    .smaller = Smaller
    .bigger = Bigger
    .minimise = Minimise to a bubble
    .close = Hide the keyboard
    .corner = Resize: click, move, click
    .bubble = Show the keyboard

## The size presets in the top bar, in [keyboard] presets order: the letter each button shows, then its name.
size =
    .s = S
    .m = M
    .l = L
sizename =
    .s = Small
    .m = Medium
    .l = Large

## The D-pad's arrows and its Stop button, by icons.toml [dpad] id.
dpad =
    .up = Scroll up
    .right = Scroll right
    .left = Scroll left
    .down = Scroll down
    .stop = Stop scrolling

## The chip lane on the top bar: a chip's name for screen readers ($word is its text), also in forget mode.
## Then the More and Undo chips, the lane's name while it offers nothing, and a password box's note.
chip =
    .name = Suggestion: { $word }
    .forget-name = Forget: { $word }
    .more = More emails
    .undo = Undo
    .none = No suggestions
    .locked = Password box: nothing is suggested or learned

## The suggestions menu: its button, its rows with their short labels, and the Cancel that ends forget mode or closes
## the email list.
words =
    .menu = Suggestions menu
    .forget = Forget a word
    .forget-short = Forget
    .cancel = Cancel
    .pause = Pause learning
    .pause-short = Pause
    .resume = Resume learning
    .resume-short = Resume
    .off = Turn suggestions off
    .off-short = Off
    .on = Turn suggestions on
    .on-short = On

## Light, dark or following Windows, by [ui] mode value.
mode =
    .light = Light
    .dark = Dark
    .auto = Auto

## Window titles, which screen readers announce.
window =
    .keyboard = { -app } keyboard
    .bubble = { -app } keyboard (minimised)
    .notice = { -app } notice
    .menu = { -app } menu
    .list = Keyboard languages
    .tip = { -app } tip
    .pointer = Pointer helper
    .outline = Scroll outline

## What screen readers name the cover that ends a move or resize.
catch = Click to finish moving or resizing

## The mode badge beside the pointer: its word while Grab holds a press down, the scroll pad's speed, and what joins
## its parts for screen readers.
badge =
    .holding = Holding
    .slow = slow
    .fast = fast
    .join = {", "}

tray =
    .show = Show keyboard
    .quit = Quit

## Buttons on a notice.
button =
    .retry = Try again
    .ok = OK

## The notice banner: how many more notices wait behind the one it shows.
## Plural forms select on $form, which is $more as the CLDR rules read it; $more is at least 1.
banner =
    .more = { $form ->
        [one] { $more } more notice
       *[other] { $more } more notices
    }

## Paged lists: the page buttons, and the page number between them.
page =
    .back = Previous page
    .on = Next page
    .of = { $at } of { $count }

## Notices by kx-module-api key. $module is a module name, $keys a list of setting names and $lang a language name.
notice =
    .settings-reset = Some { $module } settings were not valid, so they went back to their defaults: { $keys }.
    .settings-restored = The settings file was damaged, so its last good copy was restored.
    .settings-defaults = No settings file could be used, so the default settings are in use.
    .settings-newer = The { $module } settings come from a newer version. They are kept, and the defaults are used for now.
    .settings-unsaved = Settings could not be saved, so changes last only until the keyboard closes.
    .settings-locked = Another program is using the settings file. Default settings are in use, and nothing is saved.
    .module-failed = { $module } stopped after an error. Everything else still works.
    .keys-not-sent = Some keys did not reach the app in front. Check what was typed.
    .lang-not-switched = The app in front could not switch the language.
    .mouse-hook-failed = Mouse assist could not start, as Windows refused its mouse watcher. Try the key again.
    .mouse-reset = Mouse assist hit an error, so its modes turned off. Tap a mode key to start it again.
    .mouse-stopped = Mouse assist stopped after an error. Restart { -app } to use it again.
    .scroll-point = Point at what you want to scroll, then tap an arrow.
    .scroll-none = Nothing to scroll there.
    .scroll-end = Already at the end.
    .scroll-failed = The scroll pad hit an error. Tap an arrow to try again.
    .scroll-stopped = The scroll pad stopped after an error. Restart { -app } to use it again.
    .power-failed = The system's power dialog could not open.
    .power-stopped = The power key stopped after an error. Restart { -app } to use it again.
    .words-damaged = The learned words file was damaged, so learning starts again from empty.
    .words-missing = The word list for { $lang } is missing, so suggestions come only from the words you type.
    .words-locked = Learned words could not be unlocked, so nothing learned is saved this time.

## Module names by module id; an id not listed shows itself.
module =
    .kernel = { -app }
    .keyboard = Keyboard
    .layouts = Keyboard layouts
    .mouse = Mouse assist
    .power = Power key
    .predict = Word suggestions
    .scroll = Scroll pad
    .ui = Appearance

## The language key's designs, by [keyboard] lang_key value.
langkey =
    .carousel = Carousel
    .cycle = Cycle
    .list = List

## The space bar: the language, then the layout.
lang =
    .space = { $lang } · { $layout }

## Theme names by themes.toml id: model names stay, plain words are translated.
theme =
    .native = Native Adaptive
    .et66 = ET66
    .dolch = Modern Dolch

## Lists: what joins two items, the end of a list that leaves $more items out, and a list that names none.
## Plural forms select on $form, which is $more as the CLDR rules read it; $more is at least 1.
list =
    .join = {", "}
    .more = { $list } and { $more } more
    .count = { $form ->
        [one] { $more } setting
       *[other] { $more } settings
    }

### English texts, the fallback for every other language (ADR-0017).
### A key `group.name` in code is message `group`, attribute `name`. All three files hold the same keys.

## The product's name, written once and shared by the texts below.
-app = KeyXtend

## Named keys by board.toml id: the name that tips and screen readers use.
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

## The short word on a key, or under its icon at large sizes, as real keyboards print it.
label =
    .esc = Esc
    .bksp = Backspace
    .tab = Tab
    .del = Del
    .caps = Caps Lock
    .enter = Enter
    .shift = Shift
    .rshift = Shift
    .up = Up
    .menu = Menu
    .ctrl = Ctrl
    .win = Win
    .alt = Alt
    .altgr = Alt Gr
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

tip =
    .later = Coming in a later version

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

tray =
    .show = Show keyboard
    .quit = Quit

## Buttons on a notice.
button =
    .retry = Try again
    .ok = OK

## Notices by kx-module-api key. $module is a module name and $keys a list of setting names.
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

## Module names by module id; an id not listed shows itself.
module =
    .kernel = { -app }
    .keyboard = Keyboard
    .layouts = Keyboard layouts
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

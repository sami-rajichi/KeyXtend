### English texts of the clipboard panel (P8), loaded into the same bundle as en.ftl; all three files hold the same keys.
### Plural forms select on $form, which is the count as the CLDR rules read it; [0] is the exact count 0.

## The panel: title, kept days, search hint, Close, filters, kind names (for a row with no text), footer, Delete-all question.
## A row's time is a day word or a weekday (Monday is 1) and the 24-hour time, else the date; sizes come with two decimals.
## The meta line leaves the app out when unknown; screen readers hear .ticked, .pinned and .join after a row's text.
clip =
    .title = Clipboard history
    .kept = { $form ->
        [one] Kept for { $days } day
       *[other] Kept for { $days } days
    }
    .search = Type to search
    .close = Close
    .filter-all = All
    .filter-text = Text
    .filter-images = Images
    .filter-files = Files
    .filter-media = Media
    .kind-text = Text
    .kind-link = Link
    .kind-image = Image
    .kind-file = File
    .kind-media = Media
    .preview = Preview
    .pin = Pin
    .unpin = Unpin
    .delete = Delete
    .delete-all = Delete all
    .insert = Insert ({ $count })
    .insert-one = Insert
    .back = Back
    .confirm = { $form ->
        [0] No unpinned entries to delete.
        [one] Delete { $count } entry? Pinned ones stay.
       *[other] Delete { $count } entries? Pinned ones stay.
    }
    .yes = Yes
    .no = No
    .empty = Nothing copied yet. Copy some text, an image or a file, and it shows here.
    .today = Today
    .yesterday = Yesterday
    .day-1 = Mon
    .day-2 = Tue
    .day-3 = Wed
    .day-4 = Thu
    .day-5 = Fri
    .day-6 = Sat
    .day-7 = Sun
    .when = { $word } { $time }
    .time = { $hour }:{ $minute }
    .date = { $day }/{ $month }/{ $year }
    .size = { $mb } MB
    .size-gb = { $gb } GB
    .size-small = < { $mb } MB
    .size-unknown = Size unknown
    .storage = History: { $used } of { $cap } · { $free } free on { $drive }
    .open-folder = Open folder
    .meta = { $app } · { $when } · { $size }
    .meta-bare = { $when } · { $size }
    .chars = { $form ->
        [one] { $count } character
       *[other] { $count } characters
    }
    .expires = Goes tonight
    .note-too-big = That copy was over the size limit, so it was not kept.
    .note-disk = The drive is nearly full, so new copies are not kept.
    .note-pinned-full = Pinned entries fill the history, so new copies are not kept. Unpin some to make room.
    .note-busy = The clipboard was busy, so nothing was inserted. Try again.
    .note-paste-failed = The paste did not go through, so Insert stopped. Check what went in.
    .ticked = Ticked { $count }
    .pinned = Pinned
    .join = {", "}

## Numbers: the mark between the whole part and the decimals, and the one between each three digits (CLDR).
num =
    .decimal = {"."}
    .group = {","}

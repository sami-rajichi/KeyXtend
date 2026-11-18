### نصوص الإملاء (P11) تُحمَّل في الحزمة نفسها مع ar.ftl، والملفات الثلاثة فيها المفاتيح نفسها؛ الأرقام غربية.
### تُختار صيغة الجمع حسب $form، وهو العدد كما تقرؤه قواعد CLDR؛ وصيغة الصفر للعدد 0.

## بالترتيب: الشريط (الاسم والإعدادات والحالات والأزرار وتنبيه عدم سماع شيء بعد ونقاط كلمة المرور والمؤقّت)، ثم كل ملاحظة
## حسب معرّفها، ثم رأس اللوحة وقائمتها الرئيسية، والنماذج، ووحدة المعالجة، ومقارنة النماذج، والسجل، ثم الخيارات (كل خيار حسب
## مفتاح إعداده مع شرطة مكان الشرطة السفلية)، ومواضع الشريط والوحدات. و$lang اسم اللغة بلغتها.
voice =
    .title = إملاء
    .gear = إعدادات الإملاء
    .listening = جارٍ الاستماع…
    .transcribing = جارٍ تحويل الكلام إلى نص…
    .preparing = إملاء، جارٍ تجهيز الميكروفون
    .nothing = لم يُسمع شيء بعد
    .stop = إيقاف
    .cancel = إلغاء
    .close = إغلاق
    .copy = نسخ
    .copied = تم النسخ
    .undo = تراجع عن الإدراج
    .again = إعادة المحاولة
    .anyway = تحويل التسجيل إلى نص
    .listen = إعادة الاستماع
    .download = تنزيل
    .download-size = تنزيل · { $size }
    .mask = ••••••••
    .timer = { $minutes }:{ $seconds }
    .note-nothing = لم يُسمع شيء. هل يعمل الميكروفون؟
    .note-cancelled = أُلغي الإملاء، ولم يُكتب شيء.
    .note-no-mic = لم يُعثر على ميكروفون.
    .note-mic-busy = يستخدم تطبيق آخر الميكروفون.
    .note-mic-refused = إعدادات الخصوصية تمنع الوصول إلى الميكروفون.
    .note-unplugged = فُصل الميكروفون.
    .note-no-model = لا يوجد نموذج للغة { $lang } بعد
    .note-language = النموذج المختار لا يدعم اللغة { $lang }. اختر نموذجًا آخر من الإعدادات.
    .note-checking = ما زال الملف المختار قيد الفحص. حاول مرة أخرى بعد قليل.
    .note-no-language = لا يوجد في القائمة نموذج يكتب باللغة { $lang }.
    .note-no-fit = الذاكرة المتاحة لا تكفي لهذا النموذج. أغلق بعض التطبيقات أو اختر نموذجًا أصغر.
    .note-fault = توقف الإملاء بسبب خطأ. حاول مرة أخرى.
    .note-pieces = توقف الإملاء قبل اكتماله، وهذا ما كُتب حتى الآن.
    .note-too-long = توقف الإملاء عند أقصى مدة، وكُتب النص.
    .note-no-field = لم يُكتب النص كاملًا. انسخه من هنا أو من السجل.
    .note-stopped = توقفت كتابة النص قبل اكتمالها. انسخه كاملًا من هنا أو من السجل.
    .note-copy-failed = تعذّر نسخ النص. حاول مرة أخرى.
    .note-password = كُتب في حقل كلمة مرور، فلا يُحفَظ ولا يُنسَخ.
    .note-fell = تعذّر العمل على بطاقة الرسوميات، فكتب المعالج هذا النص.
    .note-print = لم يجتز الملف المُنزَّل الفحص، فحُذف.
    .note-taken = يحمل ملف آخر في مجلد النماذج اسم هذا النموذج، فلم يُنزَّل شيء.
    .note-full = لا توجد مساحة كافية على القرص، فلم يُنزَّل شيء.
    .note-folder = تعذّر استخدام مجلد النماذج. غيِّره من صفحة النماذج.
    .note-net = لم يكتمل التنزيل. حاول مرة أخرى لاحقًا.
    .back = رجوع
    .lang = اللغة: { $lang }
    .models = النماذج
    .cloud = الخدمات السحابية
    .device = وحدة المعالجة
    .compare = مقارنة النماذج
    .history = السجل
    .options = الخيارات
    .no-model = لا يوجد نموذج جاهز
    .auto-cpu = تلقائي (المعالج)
    .auto-on = تلقائي ({ $device })
    .last-run = آخر مقارنة: { $date }
    .never = لم تُجرَ أي مقارنة بعد
    .texts = { $form ->
        [zero] لا نصوص
        [one] نص واحد
        [two] نصان
        [few] { $count } نصوص
        [many] { $count } نصًا
       *[other] { $count } نص
    }
    .defaults = جميعها على الوضع الافتراضي
    .changed = المعدَّلة: { $count }
    .again-hint = يعيد تحويل آخر تسجيل إلى نص بالاختيارات المحددة هنا.
    .for-lang = للغة { $lang }
    .best = النموذج المُوصى به للغة { $lang }:
    .untested = لم يُختبر هذا الترتيب للغة { $lang }.
    .slow = ستكون النماذج المحلية للغة { $lang } بطيئة جدًا على هذا الحاسوب.
    .mark-used = قيد الاستخدام
    .mark-default = افتراضي
    .mark-ready = جاهز
    .mark-download = غير مُنزَّل
    .mark-not-here = لا يناسب هذا الحاسوب
    .mark-other = ملف آخر، لم يُتحقَّق منه
    .unfit = يتطلب { $need } من الذاكرة، والمتوفر { $free }
    .got = { $done } من { $total }
    .paused = متوقف مؤقتًا · { $done } من { $total }
    .pause = إيقاف مؤقت
    .resume = استئناف
    .use = استخدامه لهذه اللغة
    .delete-file = حذف الملف
    .folder = مجلد النماذج: { $path } · { $free } متوفرة
    .change = تغيير
    .open = فتح
    .folder-offer = هل تُحفَظ النماذج في { $path }؟ المتوفر { $free }.
    .folders = مجلد النماذج
    .drives = محركات الأقراص
    .here = استخدام هذا المجلد
    .device-hint = تُفحَص البطاقات قبل استخدامها
    .auto = تلقائي
    .auto-line = يُستخدم المعالج إلى أن تنجح إحدى البطاقات في الفحص
    .cpu = المعالج
    .cpu-line = { $cores } · ذاكرة { $total }، المتوفر { $free }
    .cores = { $form ->
        [one] نواة واحدة
        [two] نواتان
        [few] { $count } أنوية
        [many] { $count } نواة
       *[other] { $count } نواة
    }
    .card-line = بطاقة الرسوميات · { $total }، المتوفر { $free }
    .built-in-line = رسوميات مدمجة · { $total } مشتركة، المتوفر { $free }
    .checking = جارٍ الفحص: تهيئة البطاقة
    .faster = { $form ->
        [one] أسرع مرة واحدة
        [two] أسرع مرتين
        [few] أسرع { $times } مرات
        [many] أسرع { $times } مرة
       *[other] أسرع { $times } مرة
    }
    .mark-passed = نجح الفحص · { $faster }
    .mark-failed = فشل الفحص
    .why-slow = كانت { $faster } فقط، والمطلوب { $need }.
    .why-differs = اختلفت في نصّها { $words }، والمسموح { $allowed }.
    .why-room = لم تكن الذاكرة المتاحة على البطاقة كافية.
    .why-error = أبلغت البطاقة عن خطأ.
    .failed = { $name }: فشل الفحص. { $why }
    .check = فحص
    .use-device = استخدام
    .intro = اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.
    .intro-free = قل بضع جمل بصوت عالٍ مرة واحدة، ثم يكتبها كل نموذج جاهز، واحدًا تلو الآخر.
    .read-text = يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة.
    .ready = { $form ->
        [zero] لا نماذج جاهزة · { $device }
        [one] نموذج واحد جاهز · { $device }
        [two] نموذجان جاهزان · { $device }
        [few] { $count } نماذج جاهزة · { $device }
        [many] { $count } نموذجًا جاهزًا · { $device }
       *[other] { $count } نموذج جاهز · { $device }
    }
    .start = بدء القراءة
    .recording = اقرأ النص الآن، ثم اضغط على «إيقاف» في الشريط.
    .step = النموذج { $at } من { $of }
    .mark-done = انتهى · { $wait }
    .mark-writing = جارٍ الكتابة…
    .mark-waiting = في الانتظار
    .mark-run-failed = فشل
    .mark-off = معطّلة
    .mark-needs-key = يلزم مفتاح
    .mark-needs-address = يلزم عنوان
    .mark-on = جاهزة
    .mark-untested = غير مُختبَرة
    .mark-warn = تنبيه
    .mark-unavailable = غير متاح
    .col-model = النموذج
    .col-wrong = كلمات خاطئة
    .col-wait = الانتظار
    .col-memory = الذاكرة
    .wrong-of = { $wrong } من { $words }
    .kept = تُحفَظ هذه الأرقام فقط، لا التسجيل ولا النصوص.
    .result-line = { $date } · { $device }
    .rerun = إعادة المقارنة
    .history-sub = { $texts } · الأحدث أولًا
    .meta = { $when } · { $lang } · { $model }
    .pin = تثبيت
    .unpin = إلغاء التثبيت
    .delete = حذف
    .delete-all = حذف الكل
    .insert = إدراج ({ $count })
    .insert-one = إدراج
    .confirm = { $form ->
        [zero] لا توجد نصوص غير مثبّتة لحذفها.
        [one] هل تريد حذف نص واحد؟ تبقى النصوص المثبّتة.
        [two] هل تريد حذف نصّين؟ تبقى النصوص المثبّتة.
        [few] هل تريد حذف { $count } نصوص؟ تبقى النصوص المثبّتة.
        [many] هل تريد حذف { $count } نصًا؟ تبقى النصوص المثبّتة.
       *[other] هل تريد حذف { $count } نص؟ تبقى النصوص المثبّتة.
    }
    .yes = نعم
    .no = لا
    .empty = لا توجد نصوص بعد. أملِ شيئًا ليظهر هنا.
    .ticked = محدّد رقم { $count }
    .pinned = مثبّت
    .join = {"، "}
    .for-all = لجميع اللغات
    .applies = تُطبَّق التغييرات فورًا.
    .on = مفعّل
    .off = معطّل
    .minus = أقل: { $name }
    .plus = أكثر: { $name }
    .opt-sounds = صوتا البدء والإيقاف
    .opt-silence-stop = الإيقاف بعد الصمت
    .opt-silence-stop-s = مدة الصمت قبل الإيقاف
    .opt-done-ms = مدة بقاء النص على الشريط
    .opt-place = موضع الشريط
    .opt-history = الاحتفاظ بسجل النصوص
    .opt-history-max = عدد النصوص المحفوظة
    .opt-keep-voice = الاحتفاظ بالتسجيلات
    .opt-voice-max-mb = المساحة المخصصة للتسجيلات
    .opt-idle-min = يُفرَّغ النموذج من الذاكرة بعد
    .opt-record-max-s = أقصى مدة للإملاء
    .opt-nothing-s = تنبيه «لم يُسمع شيء» بعد
    .opt-debounce-ms = تجاهل النقرة الثانية خلال
    .opt-insert = إدراج النص
    .opt-type-suffix = مسافة بعد النص
    .opt-type-gap-ms = الفاصل بين الأحرف
    .opt-threads = خيوط المعالجة
    .opt-wave-ms = تنعيم الموجة
    .opt-min-free-mb = المساحة المتروكة على قرص النماذج
    .opt-card-margin-mb = الذاكرة المتروكة على البطاقة
    .opt-card-gain = أدنى تسريع مطلوب من البطاقة
    .opt-card-diff = الفرق المسموح في نص البطاقة
    .place-above = فوق لوحة المفاتيح
    .place-bottom = أسفل الشاشة
    .place-top = أعلى الشاشة
    .insert-type = كتابته
    .insert-paste = لصقه دفعة واحدة
    .insert-hint = يستخدم الحافظة ثم يعيد إليها ما كان فيها؛ أما حقل كلمة المرور فيُكتب فيه النص دائمًا.
    .seconds = { $form ->
        [one] ثانية واحدة
        [two] ثانيتان
        [few] { $count } ثوانٍ
        [many] { $count } ثانية
       *[other] { $count } ثانية
    }
    .minutes = { $form ->
        [one] دقيقة واحدة
        [two] دقيقتان
        [few] { $count } دقائق
        [many] { $count } دقيقة
       *[other] { $count } دقيقة
    }
    .ms = { $form ->
        [one] ميلي ثانية واحدة
        [two] ميلي ثانيتان
        [few] { $count } ميلي ثوانٍ
        [many] { $count } ميلي ثانية
       *[other] { $count } ميلي ثانية
    }
    .words = { $form ->
        [one] كلمة واحدة
        [two] كلمتان
        [few] { $count } كلمات
        [many] { $count } كلمة
       *[other] { $count } كلمة
    }
    .times = { $form ->
        [one] مرة واحدة
        [two] مرتان
        [few] { $times } مرات
        [many] { $times } مرة
       *[other] { $times } مرة
    }
    .secs = { $s } ثانية
    .auto-n = تلقائي ({ $count })
    .mb = { $mb } ميغابايت
    .gb = { $gb } غيغابايت

## اسم اللغة حسب رمز ISO 639، كما يذكر السجل لغة النص؛ ويظهر أي رمز آخر كما هو.
langname =
    .en = الإنجليزية
    .fr = الفرنسية
    .ar = العربية

## اسم اللغة داخل الجملة حسب رمز ISO 639؛ ويبقى لأي رمز آخر الاسمُ الذي يعطيه النظام.
langin =
    .en = { langname.en }
    .fr = { langname.fr }
    .ar = { langname.ar }

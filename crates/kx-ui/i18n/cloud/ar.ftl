### نصوص الخدمات السحابية للإملاء (P11، ADR-0047) تُحمَّل في الحزمة نفسها مع ar.ftl؛ الأرقام غربية.
### تُختار صيغة الجمع حسب $form، أو حسب صيغة القيمة في المعلومة، مثل $days-form؛ وصيغة الصفر للعدد 0.

## بالترتيب: صفحة الخدمات السحابية وأزرارها، ثم بطاقة الخدمة ومعلوماتها حسب المجموعة والمعرّف (تُملأ بأرقام جدول الخدمات)،
## ثم المبالغ، وصفحة المفتاح، والشريط وملاحظاته، وتحذير المقارنة، وجانب السحابة في صفحة النماذج. و$https بداية كل عنوان يُكتب.
## لكل تحذير علامة قصيرة في سطر خدمته: warn-mark- ثم معرّف المعلومة.
cloud =
    .sub = بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف
    .switch = استخدام الخدمات السحابية
    .off-line = الخدمات السحابية معطّلة: لا تُعرض أي خدمة ولا يُرسَل شيء.
    .free = مجانًا: { $free }
    .custom = عنوان مخصّص
    .custom-line = عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك
    .home = مفعّلة · { $ready }
    .ready = { $form ->
        [zero] لا توجد خدمة جاهزة
        [one] خدمة واحدة جاهزة
        [two] خدمتان جاهزتان
        [few] { $count } خدمات جاهزة
        [many] { $count } خدمة جاهزة
       *[other] { $count } خدمة جاهزة
    }
    .on = تفعيل
    .off = تعطيل
    .facts = المعلومات
    .lines = المفتاح والنموذج
    .card-sub = اقرأ هذا قبل التفعيل
    .card-on = فُعِّلت في { $date }
    .card-where = وجهة صوتك
    .card-trains = التدريب
    .card-keeps = ما يُحتفَظ به
    .card-free = الحصة المجانية
    .card-light = استخدام خفيف · { $hours }
    .card-heavy = استخدام مكثّف · { $hours }
    .card-bills = الحد الأدنى للفوترة
    .card-rule = شروط الاستخدام
    .card-hours = { $form ->
        [one] ساعة واحدة شهريًا
        [two] ساعتان شهريًا
        [few] { $count } ساعات شهريًا
        [many] { $count } ساعة شهريًا
       *[other] { $count } ساعة شهريًا
    }
    .card-read-on = قُرئت هذه المعلومات في { $date }.
    .card-old = قد تكون قديمة، فراجع صفحة الخدمة.
    .card-link = فتح { $site }
    .card-confirm = تأكيد
    .card-ask = لا يُرسَل صوتك إلى { $name } إلا بعد الإيقاف، وبمفتاحك الخاص.
    .card-custom = العنوان الذي تكتبه
    .card-untested = غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.
    .fact-where-us = الولايات المتحدة
    .fact-where-typed = العنوان الذي تكتبه
    .fact-where-eu = الاتحاد الأوروبي
    .fact-where-unstated = غير مذكور في صفحات الخدمة
    .fact-where-host = يتوقف على المضيف الذي يشغّل النموذج
    .fact-trains-never = لا يُستخدم للتدريب
    .fact-trains-unknown = غير معروف، ويتوقف على الخدمة
    .fact-trains-opt-in = لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك
    .fact-trains-switch = غير مذكور، ويوقفه هذا الخيار في حسابك: { $switch }
    .fact-trains-unstated = غير مذكور في صفحات الخدمة
    .fact-trains-host = يتوقف على المضيف الذي يشغّل النموذج
    .fact-trains-default = يُستخدم للتدريب افتراضيًا
    .fact-keeps-logs = لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى { $days-form ->
        [one] يوم واحد
        [two] يومين
        [few] { $days } أيام
        [many] { $days } يومًا
       *[other] { $days } يوم
    }، ويمكن إيقاف ذلك من هذا الخيار في حسابك: { $switch }
    .fact-keeps-unknown = غير معروف، ويتوقف على الخدمة
    .fact-keeps-none = لا شيء
    .fact-keeps-on-request = غير مذكور، وعدم الاحتفاظ بأي شيء متاح عند الطلب فقط
    .fact-keeps-stored = طلباتك افتراضيًا، ويوقف ذلك هذا الخيار في حسابك: { $switch }
    .fact-keeps-deleted = لا شيء: يُحذف كل شيء فور المعالجة
    .fact-keeps-memory = لا شيء: يبقى في الذاكرة أثناء المعالجة فقط
    .fact-keeps-host = يتوقف على المضيف، وهناك خيار مفعّل من البداية لا يرسل إلا إلى المضيفين الذين لا يحتفظون بشيء
    .fact-keeps-history = سجلّك افتراضيًا، وعدم الاحتفاظ بأي شيء متاح لخطط المؤسسات فقط
    .fact-free-daily = { $hours-form ->
        [one] ساعة واحدة
        [two] ساعتان
        [few] { $hours } ساعات
        [many] { $hours } ساعة
       *[other] { $hours } ساعة
    } يوميًا، و{ $per_min-form ->
        [one] طلب واحد
        [two] طلبان
        [few] { $per_min } طلبات
        [many] { $per_min } طلبًا
       *[other] { $per_min } طلب
    } في الدقيقة
    .fact-free-unknown = غير معروف، ويتوقف على الخدمة
    .fact-free-none = لا شيء
    .fact-free-first-buy = لا شيء، وأول شراء { $least } على الأقل
    .fact-free-first-month = الشهر الأول
    .fact-free-small = حصة صغيرة للحسابات الجديدة
    .fact-free-minutes-month = نحو { $minutes-form ->
        [one] دقيقة واحدة
        [two] دقيقتين
        [few] { $minutes } دقائق
        [many] { $minutes } دقيقة
       *[other] { $minutes } دقيقة
    } شهريًا
    .fact-price-free-or = مجاني ضمن الحصة المجانية، وإلا فمن { $low-after } إلى { $high-after } حسب النموذج
    .fact-price-unknown = غير معروف، ويتوقف على الخدمة
    .fact-price-plain = { $price }
    .fact-price-range = من { $low-after } إلى { $high-after } حسب النموذج
    .fact-price-range-fee = من { $low-after } إلى { $high-after } حسب النموذج، مع رسوم بنسبة { $fee }% عند شراء الرصيد
    .fact-warn-trains = تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: { $switch }
    .fact-warn-trains-profile = تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: { $switch }
    .warn-mark-trains = التدريب مفعّل افتراضيًا
    .warn-mark-trains-profile = التدريب مفعّل افتراضيًا
    .fact-bills-least = يُحتسَب لكل طلب ما لا يقل عن { $seconds-form ->
        [one] ثانية واحدة
        [two] ثانيتين
        [few] { $seconds } ثوانٍ
        [many] { $seconds } ثانية
       *[other] { $seconds } ثانية
    }، فتكلّف المقاطع القصيرة أكثر
    .fact-rule-non-commercial = الخطة المجانية للاستخدام غير التجاري فقط، مع نسب المحتوى إلى الخدمة
    .money-usd = { $amount-form ->
        [one] دولار واحد
        [two] { $case ->
            [after] دولارين
           *[alone] دولاران
        }
        [few] { $amount } دولارات
        [many] { $amount } دولارًا
       *[other] { $amount } دولار
    }
    .money = { $amount } { $code }
    .key-line = المفتاح
    .key-hint = اكتب مفتاحك أو الصقه
    .key-sealed = يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.
    .key-link = إنشاء مفتاح في { $site }
    .key-paste = لصق
    .key-test = اختبار
    .key-remove = إزالة المفتاح
    .key-testing = جارٍ الاختبار…
    .key-ok = المفتاح يعمل
    .key-bad = رُفض المفتاح. تحقّق منه في صفحة الخدمة.
    .key-failed = لم يصل رد بشأن المفتاح. تحقّق من الشبكة واختبره مرة أخرى.
    .key-unfit = لا يمكن إرسال هذا النص بوصفه مفتاحًا، فلم يُحفَظ شيء.
    .key-no-test = لا يوجد اختبار مسبق لهذه الخدمة، فسيُختبر المفتاح عند أول إملاء.
    .key-busy = لم تستجب الحافظة. جرّب «لصق» مرة أخرى.
    .address-line = العنوان
    .address-hint = عنوان خدمتك، مثل { $https }example.com/v1/audio/transcriptions
    .address-hint-own = عنوان HTTPS لموردك الخاص لدى هذه الخدمة
    .address-https = يجب أن يستخدم العنوان HTTPS، مثل { $https }example.com
    .address-host = يجب أن يحتوي العنوان على اسم مضيف، لا على عنوان IP.
    .address-domain = يجب أن يكون العنوان ضمن نطاق الخدمة نفسها.
    .address-long = العنوان طويل جدًا.
    .address-path = يحتوي العنوان على حرف غير مسموح به، مثل المسافة.
    .model-line = نموذج آخر
    .model-hint = معرّف النموذج من صفحة الخدمة
    .model-refused = معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.
    .line-save = حفظ
    .line-save-key = حفظ المفتاح
    .line-save-address = حفظ العنوان
    .line-save-model = حفظ النموذج
    .keep-nothing = الاقتصار على مضيفين لا يحتفظون بشيء
    .on-bar = يُرسَل صوتك إلى { $name } عند الإيقاف
    .use-local = استخدام النموذج المحلي
    .send-warning = سترسل المقارنة صوتك إلى { $names } عند الإيقاف.
    .note-net = لم تستجب الخدمة. تحقّق من الشبكة وحاول مرة أخرى.
    .note-key = رفضت الخدمة المفتاح. تحقّق منه في صفحة المفتاح.
    .note-credit = لم يبقَ رصيد في الحساب.
    .note-region = الخدمة غير متاحة في بلدك أو منطقتك.
    .note-allowance = نفدت الحصة المجانية حاليًا.
    .note-allowance-in = نفدت الحصة المجانية، وتتجدد بعد مدة قدرها { $in }.
    .note-rate = الطلبات كثيرة جدًا. انتظر قليلًا وحاول مرة أخرى.
    .note-too-big = كان التسجيل أكبر مما تقبله الخدمة.
    .note-language = لا تقبل الخدمة اللغة { $lang }.
    .note-empty = لم تُرسل الخدمة أي نص.
    .note-strange = لم يأتِ الرد من الخدمة، وربما كان صفحة تسجيل دخول. تحقّق من الشبكة.
    .note-terms = تغيّرت شروط الخدمة. أكّد معلوماتها مرة أخرى لإرسال الصوت.
    .note-off = الخدمة معطّلة، فلم يُرسَل شيء.
    .note-no-local = لا يُرسَل ما يُملى في حقل كلمة المرور إلى السحابة أبدًا، ولا يوجد نموذج محلي جاهز.
    .note-fault = توقّف الجزء السحابي بسبب خطأ. حاول مرة أخرى.
    .note-keep-nothing = لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.
    .hours = { $form ->
        [one] ساعة واحدة
        [two] ساعتان
        [few] { $count } ساعات
        [many] { $count } ساعة
       *[other] { $count } ساعة
    }
    .model = في السحابة، بمفتاحك
    .sides = مكان تشغيل النموذج
    .side-local = محلية
    .side-cloud = سحابية
    .band-none-on = لم تُفعَّل أي خدمة سحابية بعد.
    .band-listed = نماذج الخدمات التي فعّلتها، ولا يُرسَل شيء قبل الإيقاف.

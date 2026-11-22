.pragma library
// Written by kx-ui's texts test; do not edit.
var views = {
  "ar": {
    "card": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Groq إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الولايات المتحدة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الولايات المتحدة"
                  }
                ],
                "text": "الولايات المتحدة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، لا يُستخدم للتدريب",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا يُستخدم للتدريب"
                  }
                ],
                "text": "لا يُستخدم للتدريب"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": null,
            "said": "قُرئت هذه المعلومات في 17/11/2026.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": null
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 22,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-assemblyai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى AssemblyAI إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الاتحاد الأوروبي",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الاتحاد الأوروبي"
                  }
                ],
                "text": "الاتحاد الأوروبي"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، لا يُستخدم للتدريب",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا يُستخدم للتدريب"
                  }
                ],
                "text": "لا يُستخدم للتدريب"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء"
                  }
                ],
                "text": "لا شيء"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح www.assemblyai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 67,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-custom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى العنوان الذي تكتبه إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، العنوان الذي تكتبه",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "العنوان الذي تكتبه"
                  }
                ],
                "text": "العنوان الذي تكتبه"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، غير معروف، ويتوقف على الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير معروف، ويتوقف على الخدمة"
                  }
                ],
                "text": "غير معروف، ويتوقف على الخدمة"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، غير معروف، ويتوقف على الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير معروف، ويتوقف على الخدمة"
                  }
                ],
                "text": "غير معروف، ويتوقف على الخدمة"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": null,
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 57,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-deepinfra": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى DeepInfra إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، غير مذكور في صفحات الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور في صفحات الخدمة"
                  }
                ],
                "text": "غير مذكور في صفحات الخدمة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، لا يُستخدم للتدريب",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا يُستخدم للتدريب"
                  }
                ],
                "text": "لا يُستخدم للتدريب"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء: يبقى في الذاكرة أثناء المعالجة فقط",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء: يبقى في الذاكرة أثناء المعالجة فقط"
                  }
                ],
                "text": "لا شيء: يبقى في الذاكرة أثناء المعالجة فقط"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح docs.deepinfra.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "DeepInfra",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 61,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-elevenlabs": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى ElevenLabs إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الولايات المتحدة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الولايات المتحدة"
                  }
                ],
                "text": "الولايات المتحدة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، يُستخدم للتدريب افتراضيًا",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "يُستخدم للتدريب افتراضيًا"
                  }
                ],
                "text": "يُستخدم للتدريب افتراضيًا"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، سجلّك افتراضيًا، وعدم الاحتفاظ بأي شيء متاح لخطط المؤسسات فقط",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "سجلّك افتراضيًا، وعدم الاحتفاظ بأي شيء متاح لخطط المؤسسات فقط"
                  }
                ],
                "text": "سجلّك افتراضيًا، وعدم الاحتفاظ بأي شيء متاح لخطط المؤسسات فقط"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح elevenlabs.io",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: "
              },
              {
                "own": true,
                "text": "Terms and privacy → Data use → Improve the models for everyone"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: Terms and privacy → Data use → Improve the models for everyone"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "ElevenLabs",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 63,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-gemini": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Gemini إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، غير مذكور في صفحات الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور في صفحات الخدمة"
                  }
                ],
                "text": "غير مذكور في صفحات الخدمة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، يُستخدم للتدريب ما دمت ضمن الحصة المجانية",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "يُستخدم للتدريب ما دمت ضمن الحصة المجانية"
                  }
                ],
                "text": "يُستخدم للتدريب ما دمت ضمن الحصة المجانية"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، في المستوى المدفوع: طلباتك لمدة محدودة لكشف إساءة الاستخدام",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "في المستوى المدفوع: طلباتك لمدة محدودة لكشف إساءة الاستخدام"
                  }
                ],
                "text": "في المستوى المدفوع: طلباتك لمدة محدودة لكشف إساءة الاستخدام"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح aistudio.google.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "ما دمت ضمن الحصة المجانية، تستخدم Google صوتك لتحسين منتجاتها، وقد يطّلع مراجعون بشريون على ما تقوله."
              }
            ],
            "text": "ما دمت ضمن الحصة المجانية، تستخدم Google صوتك لتحسين منتجاتها، وقد يطّلع مراجعون بشريون على ما تقوله."
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Gemini",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 65,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-groq": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Groq إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الولايات المتحدة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الولايات المتحدة"
                  }
                ],
                "text": "الولايات المتحدة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، لا يُستخدم للتدريب",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا يُستخدم للتدريب"
                  }
                ],
                "text": "لا يُستخدم للتدريب"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح console.groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 47,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-lemonfox": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Lemonfox إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، غير مذكور في صفحات الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور في صفحات الخدمة"
                  }
                ],
                "text": "غير مذكور في صفحات الخدمة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، غير مذكور في صفحات الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور في صفحات الخدمة"
                  }
                ],
                "text": "غير مذكور في صفحات الخدمة"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء: يُحذف كل شيء فور المعالجة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء: يُحذف كل شيء فور المعالجة"
                  }
                ],
                "text": "لا شيء: يُحذف كل شيء فور المعالجة"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح www.lemonfox.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Lemonfox",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 55,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-mistral": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Mistral إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الاتحاد الأوروبي",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الاتحاد الأوروبي"
                  }
                ],
                "text": "الاتحاد الأوروبي"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، غير مذكور، ويوقفه هذا الخيار في حسابك: Admin panel → Privacy → Anonymous improvement data",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور، ويوقفه هذا الخيار في حسابك: "
                  },
                  {
                    "own": true,
                    "text": "Admin panel → Privacy → Anonymous improvement data"
                  }
                ],
                "text": "غير مذكور، ويوقفه هذا الخيار في حسابك: Admin panel → Privacy → Anonymous improvement data"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، غير مذكور، وعدم الاحتفاظ بأي شيء متاح عند الطلب فقط",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور، وعدم الاحتفاظ بأي شيء متاح عند الطلب فقط"
                  }
                ],
                "text": "غير مذكور، وعدم الاحتفاظ بأي شيء متاح عند الطلب فقط"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح docs.mistral.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Mistral",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 51,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-openai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى OpenAI إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الولايات المتحدة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الولايات المتحدة"
                  }
                ],
                "text": "الولايات المتحدة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك"
                  }
                ],
                "text": "لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء"
                  }
                ],
                "text": "لا شيء"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح developers.openai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "OpenAI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 49,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-openrouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى OpenRouter إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، يتوقف على المضيف الذي يشغّل النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "يتوقف على المضيف الذي يشغّل النموذج"
                  }
                ],
                "text": "يتوقف على المضيف الذي يشغّل النموذج"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، يتوقف على المضيف الذي يشغّل النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "يتوقف على المضيف الذي يشغّل النموذج"
                  }
                ],
                "text": "يتوقف على المضيف الذي يشغّل النموذج"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، يتوقف على المضيف، وهناك خيار مفعّل من البداية لا يرسل إلا إلى المضيفين الذين لا يحتفظون بشيء",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "يتوقف على المضيف، وهناك خيار مفعّل من البداية لا يرسل إلا إلى المضيفين الذين لا يحتفظون بشيء"
                  }
                ],
                "text": "يتوقف على المضيف، وهناك خيار مفعّل من البداية لا يرسل إلا إلى المضيفين الذين لا يحتفظون بشيء"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح openrouter.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 59,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLong-together": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Together AI إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، غير مذكور في صفحات الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير مذكور في صفحات الخدمة"
                  }
                ],
                "text": "غير مذكور في صفحات الخدمة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك"
                  }
                ],
                "text": "لا يُستخدم للتدريب إلا إذا وافقتَ على ذلك"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، طلباتك افتراضيًا، ويوقف ذلك هذا الخيار في حسابك: Organization Settings → Privacy → Prompt storage",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "طلباتك افتراضيًا، ويوقف ذلك هذا الخيار في حسابك: "
                  },
                  {
                    "own": true,
                    "text": "Organization Settings → Privacy → Prompt storage"
                  }
                ],
                "text": "طلباتك افتراضيًا، ويوقف ذلك هذا الخيار في حسابك: Organization Settings → Privacy → Prompt storage"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح docs.together.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Together AI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 53,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-assemblyai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى AssemblyAI إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، رصيد دائم بقيمة 50 دولارًا، دون بطاقة دفع",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "رصيد دائم بقيمة 50 دولارًا، دون بطاقة دفع"
                  }
                ],
                "text": "رصيد دائم بقيمة 50 دولارًا، دون بطاقة دفع"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، 4,50 دولار",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "4,50 دولار"
                  }
                ],
                "text": "4,50 دولار"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، 18 دولارًا",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "18 دولارًا"
                  }
                ],
                "text": "18 دولارًا"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح www.assemblyai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 68,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-custom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى العنوان الذي تكتبه إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، غير معروف، ويتوقف على الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير معروف، ويتوقف على الخدمة"
                  }
                ],
                "text": "غير معروف، ويتوقف على الخدمة"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، غير معروف، ويتوقف على الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير معروف، ويتوقف على الخدمة"
                  }
                ],
                "text": "غير معروف، ويتوقف على الخدمة"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، غير معروف، ويتوقف على الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير معروف، ويتوقف على الخدمة"
                  }
                ],
                "text": "غير معروف، ويتوقف على الخدمة"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": null,
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 58,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-deepinfra": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى DeepInfra إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، لا شيء",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء"
                  }
                ],
                "text": "لا شيء"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، من 0,18 دولار إلى 1,80 دولار حسب النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "من 0,18 دولار إلى 1,80 دولار حسب النموذج"
                  }
                ],
                "text": "من 0,18 دولار إلى 1,80 دولار حسب النموذج"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، من 0,72 دولار إلى 7,20 دولار حسب النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "من 0,72 دولار إلى 7,20 دولار حسب النموذج"
                  }
                ],
                "text": "من 0,72 دولار إلى 7,20 دولار حسب النموذج"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح docs.deepinfra.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "DeepInfra",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 62,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-elevenlabs": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى ElevenLabs إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، نحو 30 دقيقة شهريًا",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "نحو 30 دقيقة شهريًا"
                  }
                ],
                "text": "نحو 30 دقيقة شهريًا"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، 6 دولارات",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "6 دولارات"
                  }
                ],
                "text": "6 دولارات"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، 22 دولارًا",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "22 دولارًا"
                  }
                ],
                "text": "22 دولارًا"
              },
              "warn": false
            },
            {
              "id": "rule",
              "label": "شروط الاستخدام",
              "said": "شروط الاستخدام، الخطة المجانية للاستخدام غير التجاري فقط، مع نسب المحتوى إلى الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الخطة المجانية للاستخدام غير التجاري فقط، مع نسب المحتوى إلى الخدمة"
                  }
                ],
                "text": "الخطة المجانية للاستخدام غير التجاري فقط، مع نسب المحتوى إلى الخدمة"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح elevenlabs.io",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: "
              },
              {
                "own": true,
                "text": "Terms and privacy → Data use → Improve the models for everyone"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: Terms and privacy → Data use → Improve the models for everyone"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "ElevenLabs",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 64,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-gemini": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Gemini إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، دون الحاجة إلى بطاقة دفع",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "دون الحاجة إلى بطاقة دفع"
                  }
                ],
                "text": "دون الحاجة إلى بطاقة دفع"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، 4,50 دولار في المستوى المدفوع",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "4,50 دولار في المستوى المدفوع"
                  }
                ],
                "text": "4,50 دولار في المستوى المدفوع"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، 18 دولارًا في المستوى المدفوع",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "18 دولارًا في المستوى المدفوع"
                  }
                ],
                "text": "18 دولارًا في المستوى المدفوع"
              },
              "warn": false
            },
            {
              "id": "rule",
              "label": "شروط الاستخدام",
              "said": "شروط الاستخدام، الاستخدام المدفوع فقط في المنطقة الاقتصادية الأوروبية وسويسرا والمملكة المتحدة، ويجب ألا يقل عمر المستخدم عن 18 عامًا",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الاستخدام المدفوع فقط في المنطقة الاقتصادية الأوروبية وسويسرا والمملكة المتحدة، ويجب ألا يقل عمر المستخدم عن 18 عامًا"
                  }
                ],
                "text": "الاستخدام المدفوع فقط في المنطقة الاقتصادية الأوروبية وسويسرا والمملكة المتحدة، ويجب ألا يقل عمر المستخدم عن 18 عامًا"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح aistudio.google.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "ما دمت ضمن الحصة المجانية، تستخدم Google صوتك لتحسين منتجاتها، وقد يطّلع مراجعون بشريون على ما تقوله."
              }
            ],
            "text": "ما دمت ضمن الحصة المجانية، تستخدم Google صوتك لتحسين منتجاتها، وقد يطّلع مراجعون بشريون على ما تقوله."
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Gemini",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 66,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-groq": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Groq إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "8 ساعات يوميًا، و20 طلبًا في الدقيقة"
                  }
                ],
                "text": "8 ساعات يوميًا، و20 طلبًا في الدقيقة"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، مجاني ضمن الحصة المجانية، وإلا فمن 0,60 دولار إلى 1,67 دولار حسب النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "مجاني ضمن الحصة المجانية، وإلا فمن 0,60 دولار إلى 1,67 دولار حسب النموذج"
                  }
                ],
                "text": "مجاني ضمن الحصة المجانية، وإلا فمن 0,60 دولار إلى 1,67 دولار حسب النموذج"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، مجاني ضمن الحصة المجانية، وإلا فمن 2,40 دولار إلى 6,66 دولار حسب النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "مجاني ضمن الحصة المجانية، وإلا فمن 2,40 دولار إلى 6,66 دولار حسب النموذج"
                  }
                ],
                "text": "مجاني ضمن الحصة المجانية، وإلا فمن 2,40 دولار إلى 6,66 دولار حسب النموذج"
              },
              "warn": false
            },
            {
              "id": "bills",
              "label": "الحد الأدنى للفوترة",
              "said": "الحد الأدنى للفوترة، يُحتسَب لكل طلب ما لا يقل عن 10 ثوانٍ، فتكلّف المقاطع القصيرة أكثر",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "يُحتسَب لكل طلب ما لا يقل عن 10 ثوانٍ، فتكلّف المقاطع القصيرة أكثر"
                  }
                ],
                "text": "يُحتسَب لكل طلب ما لا يقل عن 10 ثوانٍ، فتكلّف المقاطع القصيرة أكثر"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح console.groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 48,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-lemonfox": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Lemonfox إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، الشهر الأول",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الشهر الأول"
                  }
                ],
                "text": "الشهر الأول"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، 5 دولارات",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "5 دولارات"
                  }
                ],
                "text": "5 دولارات"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، 10 دولارات",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "10 دولارات"
                  }
                ],
                "text": "10 دولارات"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح www.lemonfox.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 21/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 21/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Lemonfox",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 56,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-mistral": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Mistral إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، لا شيء",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء"
                  }
                ],
                "text": "لا شيء"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، من 3,15 دولار إلى 3,60 دولار حسب النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "من 3,15 دولار إلى 3,60 دولار حسب النموذج"
                  }
                ],
                "text": "من 3,15 دولار إلى 3,60 دولار حسب النموذج"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، من 12,60 دولار إلى 14,40 دولار حسب النموذج",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "من 12,60 دولار إلى 14,40 دولار حسب النموذج"
                  }
                ],
                "text": "من 12,60 دولار إلى 14,40 دولار حسب النموذج"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح docs.mistral.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Mistral",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 52,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-openai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى OpenAI إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، لا شيء",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء"
                  }
                ],
                "text": "لا شيء"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، 4,05 دولار",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "4,05 دولار"
                  }
                ],
                "text": "4,05 دولار"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، 16,20 دولار",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "16,20 دولار"
                  }
                ],
                "text": "16,20 دولار"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح developers.openai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "OpenAI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 50,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-openrouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى OpenRouter إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، حصة صغيرة للحسابات الجديدة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "حصة صغيرة للحسابات الجديدة"
                  }
                ],
                "text": "حصة صغيرة للحسابات الجديدة"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، من 0,16 دولار إلى 4,05 دولار حسب النموذج، مع رسوم بنسبة 5,5% عند شراء الرصيد",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "من 0,16 دولار إلى 4,05 دولار حسب النموذج، مع رسوم بنسبة 5,5% عند شراء الرصيد"
                  }
                ],
                "text": "من 0,16 دولار إلى 4,05 دولار حسب النموذج، مع رسوم بنسبة 5,5% عند شراء الرصيد"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، من 0,65 دولار إلى 16,20 دولار حسب النموذج، مع رسوم بنسبة 5,5% عند شراء الرصيد",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "من 0,65 دولار إلى 16,20 دولار حسب النموذج، مع رسوم بنسبة 5,5% عند شراء الرصيد"
                  }
                ],
                "text": "من 0,65 دولار إلى 16,20 دولار حسب النموذج، مع رسوم بنسبة 5,5% عند شراء الرصيد"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح openrouter.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 60,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardLongPrice-together": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "تأكيد",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "لا يُرسَل صوتك إلى Together AI إلا بعد الإيقاف، وبمفتاحك الخاص.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "الحصة المجانية",
              "said": "الحصة المجانية، لا شيء، وأول شراء 15 دولارًا على الأقل",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء، وأول شراء 15 دولارًا على الأقل"
                  }
                ],
                "text": "لا شيء، وأول شراء 15 دولارًا على الأقل"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "استخدام خفيف · 15 ساعة شهريًا",
              "said": "استخدام خفيف · 15 ساعة شهريًا، 1,35 دولار",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "1,35 دولار"
                  }
                ],
                "text": "1,35 دولار"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "استخدام مكثّف · 60 ساعة شهريًا",
              "said": "استخدام مكثّف · 60 ساعة شهريًا، 5,40 دولار",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "5,40 دولار"
                  }
                ],
                "text": "5,40 دولار"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح docs.together.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 17/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "Together AI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 54,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cardOld": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "وجهة صوتك",
              "said": "وجهة صوتك، الولايات المتحدة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "الولايات المتحدة"
                  }
                ],
                "text": "الولايات المتحدة"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "التدريب",
              "said": "التدريب، غير معروف، ويتوقف على الخدمة",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "غير معروف، ويتوقف على الخدمة"
                  }
                ],
                "text": "غير معروف، ويتوقف على الخدمة"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "ما يُحتفَظ به",
              "said": "ما يُحتفَظ به، لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "لا شيء افتراضيًا، لكن قد تُحفَظ الطلبات للفحص مدةً تصل إلى 30 يومًا، ويمكن إيقاف ذلك من هذا الخيار في حسابك: Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "قد تكون قديمة، فراجع صفحة الخدمة.",
            "said": "قُرئت هذه المعلومات في 02/11/2026. قد تكون قديمة، فراجع صفحة الخدمة.",
            "text": "قُرئت هذه المعلومات في 02/11/2026."
          },
          "untested": null,
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار: Settings → Data Controls"
          }
        },
        "sub": "فُعِّلت في 10/11/2026",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 23,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "on",
                  "text": "جاهزة"
                }
              ],
              "name": "Groq",
              "tag": "غير مُختبَرة",
              "tick": true
            },
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "DeepInfra",
              "tag": null,
              "tick": false
            },
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "OpenRouter",
              "tag": null,
              "tick": false
            },
            {
              "line": "عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "يلزم عنوان"
                }
              ],
              "name": "عنوان مخصّص",
              "tag": null,
              "tick": false
            },
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "ElevenLabs",
              "tag": null,
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 19,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudLong-1": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "Groq",
              "tag": "غير مُختبَرة",
              "tick": true
            },
            {
              "line": "مجانًا: لا شيء",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "OpenAI",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: لا شيء",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "Mistral",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: لا شيء، وأول شراء 15 دولارًا على الأقل",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "Together AI",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: الشهر الأول",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "Lemonfox",
              "tag": "غير مُختبَرة",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 41,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudLong-2": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 1,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "يلزم عنوان"
                }
              ],
              "name": "عنوان مخصّص",
              "tag": "غير مُختبَرة",
              "tick": true
            },
            {
              "line": "مجانًا: حصة صغيرة للحسابات الجديدة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "OpenRouter",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: لا شيء",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "DeepInfra",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: نحو 30 دقيقة شهريًا",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "ElevenLabs",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: دون الحاجة إلى بطاقة دفع",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب ضمن الحصة المجانية"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "Gemini",
              "tag": "غير مُختبَرة",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 42,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudLong-3": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 2,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "مجانًا: رصيد دائم بقيمة 50 دولارًا، دون بطاقة دفع",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "AssemblyAI",
              "tag": "غير مُختبَرة",
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 43,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudLongOff-1": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "Groq",
              "tag": "غير مُختبَرة",
              "tick": true
            },
            {
              "line": "مجانًا: لا شيء",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "OpenAI",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: لا شيء",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "Mistral",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: لا شيء، وأول شراء 15 دولارًا على الأقل",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "Together AI",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: الشهر الأول",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "Lemonfox",
              "tag": "غير مُختبَرة",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 44,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudLongOff-2": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 1,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك",
              "marks": [
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "عنوان مخصّص",
              "tag": "غير مُختبَرة",
              "tick": true
            },
            {
              "line": "مجانًا: حصة صغيرة للحسابات الجديدة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "OpenRouter",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: لا شيء",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "DeepInfra",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: نحو 30 دقيقة شهريًا",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "ElevenLabs",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: دون الحاجة إلى بطاقة دفع",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب ضمن الحصة المجانية"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "Gemini",
              "tag": "غير مُختبَرة",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 45,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudLongOff-3": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 2,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "مجانًا: رصيد دائم بقيمة 50 دولارًا، دون بطاقة دفع",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "AssemblyAI",
              "tag": "غير مُختبَرة",
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 46,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudOff": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": "الخدمات السحابية معطّلة: لا تُعرض أي خدمة ولا يُرسَل شيء.",
          "rows": [],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": false,
            "value": "معطّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 0,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 21,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "cloudSealed": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "المفتاح والنموذج",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "تفعيل",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "on",
                  "text": "جاهزة"
                }
              ],
              "name": "Groq",
              "tag": "غير مُختبَرة",
              "tick": false
            },
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "DeepInfra",
              "tag": null,
              "tick": false
            },
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "OpenRouter",
              "tag": null,
              "tick": false
            },
            {
              "line": "عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "يلزم عنوان"
                }
              ],
              "name": "عنوان مخصّص",
              "tag": null,
              "tick": false
            },
            {
              "line": "مجانًا: 8 ساعات يوميًا، و20 طلبًا في الدقيقة",
              "marks": [
                {
                  "id": "warn",
                  "text": "التدريب مفعّل افتراضيًا"
                },
                {
                  "id": "off",
                  "text": "معطّلة"
                }
              ],
              "name": "ElevenLabs",
              "tag": null,
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "استخدام الخدمات السحابية",
            "on": true,
            "value": "مفعّل"
          }
        },
        "sub": "بمفتاحك الخاص · لا يُرسَل شيء قبل الإيقاف",
        "ticked": 1,
        "title": "الخدمات السحابية",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 20,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compare": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "6 نماذج جاهزة · تلقائي (المعالج)",
          "picks": [],
          "recording": null,
          "send": null,
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 8,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo، Groq",
              "service": "Groq",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 25,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareCloudUnticked": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo، Groq",
              "service": "Groq",
              "tick": false
            },
            {
              "name": "Whisper Large v3",
              "said": "Whisper Large v3، Groq",
              "service": "Groq",
              "tick": false
            }
          ],
          "recording": null,
          "send": null,
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 26,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareEdge": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "mistralai/voxtral-small-24b-2507",
              "said": "mistralai/voxtral-small-24b-2507، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq وLemonfox والعنوان الذي تكتبه وOpenRouter وElevenLabs وGemini وAssemblyAI عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 73,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareFew": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq وLemonfox والعنوان الذي تكتبه وOpenRouter وElevenLabs وGemini وAssemblyAI عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 74,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo، Groq",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "Whisper Large v3",
              "said": "Whisper Large v3، Groq",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "whisper-large-v3-turbo",
              "said": "whisper-large-v3-turbo، عنوان مخصّص",
              "service": "عنوان مخصّص",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq والعنوان الذي تكتبه عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 69,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareShort": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "mistralai/voxtral-mini-transcribe-3-0",
              "said": "mistralai/voxtral-mini-transcribe-3-0، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe، Gemini",
              "service": "Gemini",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq وLemonfox والعنوان الذي تكتبه وOpenRouter وElevenLabs وGemini وAssemblyAI عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 72,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareTyped": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
              "said": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm، عنوان مخصّص",
              "service": "عنوان مخصّص",
              "tick": true
            },
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe، Gemini",
              "service": "Gemini",
              "tick": true
            },
            {
              "name": "wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww",
              "said": "wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq وLemonfox والعنوان الذي تكتبه وOpenRouter وElevenLabs وGemini وAssemblyAI عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 71,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "compareWide": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "بدء القراءة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "اقرأ هذا النص بصوت عالٍ مرة واحدة، ثم يكتبه كل نموذج جاهز، واحدًا تلو الآخر.",
          "is": "ready",
          "line": "نموذجان جاهزان · تلقائي (المعالج)",
          "picks": [
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe، Gemini",
              "service": "Gemini",
              "tick": true
            },
            {
              "name": "Universal-3.6 Pro",
              "said": "Universal-3.6 Pro، AssemblyAI",
              "service": "AssemblyAI",
              "tick": true
            },
            {
              "name": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
              "said": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm، OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq وLemonfox والعنوان الذي تكتبه وOpenRouter وElevenLabs وGemini وAssemblyAI عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 70,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "comparing": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "button",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "running",
          "rows": [
            {
              "mark": {
                "id": "done",
                "text": "انتهى · 2,1 ثانية"
              },
              "name": "Whisper Small",
              "why": null
            },
            {
              "mark": {
                "id": "run-failed",
                "text": "فشل"
              },
              "name": "Whisper Large v3 Turbo",
              "why": null
            },
            {
              "mark": {
                "id": "writing",
                "text": "جارٍ الكتابة…"
              },
              "name": "Canary 1b v2",
              "why": null
            },
            {
              "mark": {
                "id": "waiting",
                "text": "في الانتظار"
              },
              "name": "Qwen3-ASR 0.6B",
              "why": null
            },
            {
              "mark": {
                "id": "waiting",
                "text": "في الانتظار"
              },
              "name": "Parakeet TDT 0.6b v3",
              "why": null
            }
          ]
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": {
          "id": "step",
          "name": "Canary 1b v2",
          "text": "النموذج 3 من 6"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 9,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "comparingCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "button",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "running",
          "rows": [
            {
              "mark": {
                "id": "run-failed",
                "text": "فشل"
              },
              "name": "Whisper Large v3 Turbo",
              "why": "رفضت الخدمة المفتاح. تحقّق منه في صفحة المفتاح."
            },
            {
              "mark": {
                "id": "writing",
                "text": "جارٍ الكتابة…"
              },
              "name": "Whisper small",
              "why": null
            }
          ]
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
        "top": {
          "id": "step",
          "name": "Whisper small",
          "text": "النموذج 2 من 2"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 27,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "device": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "check",
            "kind": "button",
            "label": "فحص",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدام",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "device",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "device",
          "rows": [
            {
              "check": null,
              "kind": "auto",
              "line": "يُستخدم المعالج إلى أن تنجح إحدى البطاقات في الفحص",
              "mark": {
                "id": "used",
                "text": "قيد الاستخدام"
              },
              "name": "تلقائي",
              "tick": false,
              "used": true
            },
            {
              "check": null,
              "kind": "cpu",
              "line": "8 أنوية · ذاكرة 16 غيغابايت، المتوفر 9,4 غيغابايت",
              "mark": null,
              "name": "المعالج",
              "tick": false,
              "used": false
            },
            {
              "check": null,
              "kind": "card",
              "line": "بطاقة الرسوميات · 8 غيغابايت، المتوفر 7,1 غيغابايت",
              "mark": {
                "id": "passed",
                "text": "نجح الفحص · أسرع 9 مرات"
              },
              "name": "NVIDIA GeForce RTX 4060 Laptop GPU",
              "tick": false,
              "used": false
            },
            {
              "check": null,
              "kind": "built-in",
              "line": "رسوميات مدمجة · 4 غيغابايت مشتركة، المتوفر 2,6 غيغابايت",
              "mark": {
                "id": "failed",
                "text": "فشل الفحص"
              },
              "name": "AMD Radeon 780M Graphics",
              "tick": false,
              "used": false
            },
            {
              "check": {
                "line": "جارٍ الفحص: تهيئة البطاقة",
                "share": 0.4
              },
              "kind": "card",
              "line": "بطاقة الرسوميات · 4 غيغابايت، المتوفر 3,8 غيغابايت",
              "mark": null,
              "name": "Intel Arc A370M",
              "tick": true,
              "used": false
            }
          ]
        },
        "sub": "تُفحَص البطاقات قبل استخدامها",
        "ticked": 1,
        "title": "وحدة المعالجة",
        "top": {
          "id": "failed",
          "name": null,
          "text": "AMD Radeon 780M Graphics: فشل الفحص. اختلفت في نصّها 3 كلمات، والمسموح كلمة واحدة."
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 7,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "folders": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "here",
            "kind": "button",
            "label": "استخدام هذا المجلد",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "folders",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "folders",
          "rows": [
            "C:\\",
            "D:\\"
          ]
        },
        "sub": "محركات الأقراص",
        "ticked": 0,
        "title": "مجلد النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 18,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "freeResults": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "rerun",
            "kind": "button",
            "label": "إعادة المقارنة",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "head": [
            "النموذج",
            "كلمات خاطئة",
            "الانتظار",
            "الذاكرة"
          ],
          "is": "results",
          "rows": [
            {
              "memory": "0,5 غيغابايت",
              "name": "Whisper Small",
              "text": "The evening train leaves at seven.",
              "tick": false,
              "wait": "2,1 ثانية",
              "wrong": null
            },
            {
              "memory": "0,5 غيغابايت",
              "name": "Whisper Large v3 Turbo",
              "text": "The evening train leaves at seven.",
              "tick": true,
              "wait": "4,0 ثانية",
              "wrong": null
            },
            {
              "memory": "1,2 غيغابايت",
              "name": "Canary 1b v2",
              "text": "The evening train leaves at seven.",
              "tick": false,
              "wait": "1,8 ثانية",
              "wrong": null
            }
          ]
        },
        "sub": "12/11/2026 · تلقائي (المعالج)",
        "ticked": 1,
        "title": "مقارنة النماذج",
        "top": {
          "id": "kept",
          "name": null,
          "text": "تُحفَظ هذه الأرقام فقط، لا التسجيل ولا النصوص."
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 11,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "history": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "حذف الكل",
          "on": true,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "button",
            "label": "نسخ",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "pin",
            "kind": "button",
            "label": "تثبيت",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "insert",
            "kind": "button",
            "label": "إدراج (2)",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "empty": null,
          "is": "history",
          "rows": [
            {
              "arabic": false,
              "id": 12,
              "meta": "اليوم 14:32 · الإنجليزية · Parakeet TDT 0.6b v3",
              "name": "Thanks for the update. I will send the signed form on Thursday.، اليوم 14:32 · الإنجليزية · Parakeet TDT 0.6b v3، محدّد رقم 2",
              "pinned": false,
              "rtl": false,
              "text": "Thanks for the update. I will send the signed form on Thursday.",
              "tick": 2
            },
            {
              "arabic": false,
              "id": 11,
              "meta": "اليوم 11:32 · الفرنسية · Parakeet TDT 0.6b v3",
              "name": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.، اليوم 11:32 · الفرنسية · Parakeet TDT 0.6b v3، محدّد رقم 1، مثبّت",
              "pinned": true,
              "rtl": false,
              "text": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
              "tick": 1
            },
            {
              "arabic": true,
              "id": 10,
              "meta": "اليوم 09:32 · العربية · Cohere Transcribe Arabic",
              "name": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.، اليوم 09:32 · العربية · Cohere Transcribe Arabic",
              "pinned": false,
              "rtl": true,
              "text": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
              "tick": null
            }
          ],
          "unpin": false
        },
        "sub": "12 نصًا · الأحدث أولًا",
        "ticked": 2,
        "title": "السجل",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 12,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "historyAsk": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "حذف الكل",
          "on": false,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "نعم",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "لا",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "هل تريد حذف 11 نصًا؟ تبقى النصوص المثبّتة.",
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "empty": null,
          "is": "history",
          "rows": [
            {
              "arabic": false,
              "id": 12,
              "meta": "اليوم 14:32 · الإنجليزية · Parakeet TDT 0.6b v3",
              "name": "Thanks for the update. I will send the signed form on Thursday.، اليوم 14:32 · الإنجليزية · Parakeet TDT 0.6b v3، محدّد رقم 2",
              "pinned": false,
              "rtl": false,
              "text": "Thanks for the update. I will send the signed form on Thursday.",
              "tick": 2
            },
            {
              "arabic": false,
              "id": 11,
              "meta": "اليوم 11:32 · الفرنسية · Parakeet TDT 0.6b v3",
              "name": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.، اليوم 11:32 · الفرنسية · Parakeet TDT 0.6b v3، محدّد رقم 1، مثبّت",
              "pinned": true,
              "rtl": false,
              "text": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
              "tick": 1
            },
            {
              "arabic": true,
              "id": 10,
              "meta": "اليوم 09:32 · العربية · Cohere Transcribe Arabic",
              "name": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.، اليوم 09:32 · العربية · Cohere Transcribe Arabic",
              "pinned": false,
              "rtl": true,
              "text": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
              "tick": null
            }
          ],
          "unpin": false
        },
        "sub": "12 نصًا · الأحدث أولًا",
        "ticked": 0,
        "title": "السجل",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 13,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "historyEmpty": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "حذف الكل",
          "on": false,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "button",
            "label": "نسخ",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "pin",
            "kind": "button",
            "label": "تثبيت",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "insert",
            "kind": "button",
            "label": "إدراج",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "empty": "لا توجد نصوص بعد. أملِ شيئًا ليظهر هنا.",
          "is": "history",
          "rows": [],
          "unpin": false
        },
        "sub": "لا نصوص · الأحدث أولًا",
        "ticked": 0,
        "title": "السجل",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 14,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "home": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": null,
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "again",
            "kind": "button",
            "label": "إعادة المحاولة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يعيد تحويل آخر تسجيل إلى نص بالاختيارات المحددة هنا.",
        "note": null,
        "page": "home",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "home",
          "rows": [
            {
              "label": "النماذج",
              "tab": "models",
              "value": "Parakeet TDT 0.6b v3"
            },
            {
              "label": "الخدمات السحابية",
              "tab": "cloud",
              "value": "معطّل"
            },
            {
              "label": "وحدة المعالجة",
              "tab": "device",
              "value": "تلقائي (المعالج)"
            },
            {
              "label": "مقارنة النماذج",
              "tab": "compare",
              "value": "آخر مقارنة: 12/11/2026"
            },
            {
              "label": "السجل",
              "tab": "history",
              "value": "12 نصًا"
            }
          ]
        },
        "sub": "اللغة: العربية",
        "ticked": 0,
        "title": "إملاء",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 6,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "models": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف الملف",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "تنزيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "تغيير",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "فتح",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "مجلد النماذج: D:\\KeyXtend\\models · 222 غيغابايت متوفرة"
          },
          "rows": [
            {
              "fetch": null,
              "line": "776 ميغابايت",
              "marks": [
                {
                  "id": "used",
                  "text": "قيد الاستخدام"
                },
                {
                  "id": "default",
                  "text": "افتراضي"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 ميغابايت",
              "marks": [
                {
                  "id": "ready",
                  "text": "جاهز"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "2,53 غيغابايت",
              "marks": [
                {
                  "id": "download",
                  "text": "غير مُنزَّل"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "إيقاف مؤقت",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "إلغاء",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 ميغابايت من 1,22 غيغابايت",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "يتطلب 3,2 غيغابايت من الذاكرة، والمتوفر 2,8 غيغابايت",
              "marks": [
                {
                  "id": "not-here",
                  "text": "لا يناسب هذا الحاسوب"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "للغة العربية",
        "ticked": 1,
        "title": "النماذج",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "النموذج المُوصى به للغة العربية:"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 1,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "modelsCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف الملف",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "تنزيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "الخدمات السحابية",
              "on": true,
              "tone": "plain"
            },
            "text": "نماذج الخدمات التي فعّلتها، ولا يُرسَل شيء قبل الإيقاف."
          },
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "في السحابة، بمفتاحك",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "ready",
                  "text": "جاهز"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.",
              "marks": [
                {
                  "id": "cloud",
                  "text": "OpenRouter"
                },
                {
                  "id": "unavailable",
                  "text": "غير متاح"
                }
              ],
              "name": "voxtral-small",
              "tick": false
            },
            {
              "fetch": null,
              "line": "في السحابة، بمفتاحك",
              "marks": [
                {
                  "id": "cloud",
                  "text": "DeepInfra"
                },
                {
                  "id": "needs-key",
                  "text": "يلزم مفتاح"
                }
              ],
              "name": "Nemotron 3.5 Transcribe",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "cloud"
          },
          "slow": null,
          "untested": null
        },
        "sub": "للغة العربية",
        "ticked": 1,
        "title": "النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 24,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "modelsLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف الملف",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "تنزيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "الخدمات السحابية",
              "on": true,
              "tone": "plain"
            },
            "text": "نماذج الخدمات التي فعّلتها، ولا يُرسَل شيء قبل الإيقاف."
          },
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "في السحابة، بمفتاحك",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "used",
                  "text": "قيد الاستخدام"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "في السحابة، بمفتاحك",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "ready",
                  "text": "جاهز"
                }
              ],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "في السحابة، بمفتاحك",
              "marks": [
                {
                  "id": "cloud",
                  "text": "عنوان مخصّص"
                },
                {
                  "id": "needs-address",
                  "text": "يلزم عنوان"
                }
              ],
              "name": "whisper-large-v3-turbo",
              "tick": false
            },
            {
              "fetch": null,
              "line": "لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.",
              "marks": [
                {
                  "id": "cloud",
                  "text": "OpenRouter"
                },
                {
                  "id": "unavailable",
                  "text": "غير متاح"
                }
              ],
              "name": "voxtral-small",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "cloud"
          },
          "slow": null,
          "untested": null
        },
        "sub": "للغة العربية",
        "ticked": 1,
        "title": "النماذج",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 40,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "modelsMore": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف الملف",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "تنزيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": {
          "id": "net",
          "text": "لم يكتمل التنزيل. حاول مرة أخرى لاحقًا."
        },
        "page": "models",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "تغيير",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "فتح",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "مجلد النماذج: D:\\KeyXtend\\models · 222 غيغابايت متوفرة"
          },
          "rows": [
            {
              "fetch": null,
              "line": "1,6 غيغابايت",
              "marks": [
                {
                  "id": "other",
                  "text": "ملف آخر، لم يُتحقَّق منه"
                }
              ],
              "name": "ggml-medium.bin",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 1,
                    "id": "resume",
                    "kind": "fetch",
                    "label": "استئناف",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 1,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "إلغاء",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": true,
                "share": 0.2
              },
              "line": "متوقف مؤقتًا · 239 ميغابايت من 1,2 غيغابايت",
              "marks": [],
              "name": "Canary 1b v2",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "النماذج",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "النموذج المُوصى به للغة العربية:"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 2,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "modelsOffer": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "نعم",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "لا",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "هل تُحفَظ النماذج في D:\\KeyXtend\\models؟ المتوفر 222 غيغابايت.",
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "776 ميغابايت",
              "marks": [
                {
                  "id": "used",
                  "text": "قيد الاستخدام"
                },
                {
                  "id": "default",
                  "text": "افتراضي"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 ميغابايت",
              "marks": [
                {
                  "id": "ready",
                  "text": "جاهز"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "2,53 غيغابايت",
              "marks": [
                {
                  "id": "download",
                  "text": "غير مُنزَّل"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "إيقاف مؤقت",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "إلغاء",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 ميغابايت من 1,22 غيغابايت",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "يتطلب 3,2 غيغابايت من الذاكرة، والمتوفر 2,8 غيغابايت",
              "marks": [
                {
                  "id": "not-here",
                  "text": "لا يناسب هذا الحاسوب"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "النماذج",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "النموذج المُوصى به للغة العربية:"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 5,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "modelsSlow": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف الملف",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "تنزيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "تغيير",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "فتح",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "مجلد النماذج: D:\\KeyXtend\\models · 222 غيغابايت متوفرة"
          },
          "rows": [
            {
              "fetch": null,
              "line": "يتطلب 5 غيغابايت من الذاكرة، والمتوفر 2,8 غيغابايت",
              "marks": [
                {
                  "id": "default",
                  "text": "افتراضي"
                },
                {
                  "id": "not-here",
                  "text": "لا يناسب هذا الحاسوب"
                }
              ],
              "name": "Cohere Transcribe Arabic",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 ميغابايت",
              "marks": [
                {
                  "id": "ready",
                  "text": "جاهز"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "local"
          },
          "slow": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "الخدمات السحابية",
              "on": true,
              "tone": "plain"
            },
            "text": "ستكون النماذج المحلية للغة العربية بطيئة جدًا على هذا الحاسوب."
          },
          "untested": "لم يُختبر هذا الترتيب للغة العربية."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "النماذج",
        "top": {
          "id": "best",
          "name": "Cohere Transcribe Arabic",
          "text": "النموذج المُوصى به للغة العربية:"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 4,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "modelsUsed": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "حذف الملف",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "تنزيل",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "تغيير",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "فتح",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "مجلد النماذج: D:\\KeyXtend\\models · 222 غيغابايت متوفرة"
          },
          "rows": [
            {
              "fetch": null,
              "line": "776 ميغابايت",
              "marks": [
                {
                  "id": "used",
                  "text": "قيد الاستخدام"
                },
                {
                  "id": "default",
                  "text": "افتراضي"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": true
            },
            {
              "fetch": null,
              "line": "650 ميغابايت",
              "marks": [
                {
                  "id": "ready",
                  "text": "جاهز"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": false
            },
            {
              "fetch": null,
              "line": "2,53 غيغابايت",
              "marks": [
                {
                  "id": "download",
                  "text": "غير مُنزَّل"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "إيقاف مؤقت",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "إلغاء",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 ميغابايت من 1,22 غيغابايت",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "يتطلب 3,2 غيغابايت من الذاكرة، والمتوفر 2,8 غيغابايت",
              "marks": [
                {
                  "id": "not-here",
                  "text": "لا يناسب هذا الحاسوب"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "محلية",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "سحابية",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "مكان تشغيل النموذج",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "للغة العربية",
        "ticked": 1,
        "title": "النماذج",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "النموذج المُوصى به للغة العربية:"
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 3,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "options": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "تُطبَّق التغييرات فورًا.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 0,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "sounds",
              "kind": "switch",
              "label": "صوتا البدء والإيقاف",
              "live": true,
              "on": true,
              "steps": null,
              "value": "مفعّل"
            },
            {
              "chips": null,
              "key": "silence_stop",
              "kind": "switch",
              "label": "الإيقاف بعد الصمت",
              "live": true,
              "on": false,
              "steps": null,
              "value": "معطّل"
            },
            {
              "chips": null,
              "key": "silence_stop_s",
              "kind": "number",
              "label": "مدة الصمت قبل الإيقاف",
              "live": false,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "أقل: مدة الصمت قبل الإيقاف",
                "plus": "أكثر: مدة الصمت قبل الإيقاف"
              },
              "value": "3 ثوانٍ"
            },
            {
              "chips": null,
              "key": "done_ms",
              "kind": "number",
              "label": "مدة بقاء النص على الشريط",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "أقل: مدة بقاء النص على الشريط",
                "plus": "أكثر: مدة بقاء النص على الشريط"
              },
              "value": "9 ثوانٍ"
            },
            {
              "chips": null,
              "key": "place",
              "kind": "choice",
              "label": "موضع الشريط",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": true,
                "minus": "أقل: موضع الشريط",
                "plus": "أكثر: موضع الشريط"
              },
              "value": "فوق لوحة المفاتيح"
            }
          ]
        },
        "sub": "لجميع اللغات",
        "ticked": 0,
        "title": "الخيارات",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 15,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "optionsInsert": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "تُطبَّق التغييرات فورًا.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 2,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "debounce_ms",
              "kind": "number",
              "label": "تجاهل النقرة الثانية خلال",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "أقل: تجاهل النقرة الثانية خلال",
                "plus": "أكثر: تجاهل النقرة الثانية خلال"
              },
              "value": "400 ميلي ثانية"
            },
            {
              "chips": [
                {
                  "label": "كتابته",
                  "on": false
                },
                {
                  "label": "لصقه دفعة واحدة",
                  "on": true
                }
              ],
              "key": "insert",
              "kind": "pair",
              "label": "إدراج النص",
              "live": true,
              "on": false,
              "steps": null,
              "value": "لصقه دفعة واحدة"
            },
            {
              "chips": null,
              "key": "insert",
              "kind": "hint",
              "label": "يستخدم الحافظة ثم يعيد إليها ما كان فيها؛ أما حقل كلمة المرور فيُكتب فيه النص دائمًا.",
              "live": true,
              "on": false,
              "steps": null,
              "value": ""
            },
            {
              "chips": null,
              "key": "type_suffix",
              "kind": "switch",
              "label": "مسافة بعد النص",
              "live": true,
              "on": true,
              "steps": null,
              "value": "مفعّل"
            },
            {
              "chips": null,
              "key": "type_gap_ms",
              "kind": "number",
              "label": "الفاصل بين الأحرف",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "أقل: الفاصل بين الأحرف",
                "plus": "أكثر: الفاصل بين الأحرف"
              },
              "value": "5 ميلي ثوانٍ"
            }
          ]
        },
        "sub": "لجميع اللغات",
        "ticked": 0,
        "title": "الخيارات",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 17,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "optionsMore": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "تُطبَّق التغييرات فورًا.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 1,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "threads",
              "kind": "number",
              "label": "خيوط المعالجة",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": true,
                "minus": "أقل: خيوط المعالجة",
                "plus": "أكثر: خيوط المعالجة"
              },
              "value": "تلقائي"
            },
            {
              "chips": null,
              "key": "type_suffix",
              "kind": "switch",
              "label": "مسافة بعد النص",
              "live": true,
              "on": true,
              "steps": null,
              "value": "مفعّل"
            },
            {
              "chips": null,
              "key": "card_gain",
              "kind": "number",
              "label": "أدنى تسريع مطلوب من البطاقة",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "أقل: أدنى تسريع مطلوب من البطاقة",
                "plus": "أكثر: أدنى تسريع مطلوب من البطاقة"
              },
              "value": "1,5 مرة"
            },
            {
              "chips": null,
              "key": "history_max",
              "kind": "number",
              "label": "عدد النصوص المحفوظة",
              "live": true,
              "on": false,
              "steps": {
                "high": true,
                "low": false,
                "minus": "أقل: عدد النصوص المحفوظة",
                "plus": "أكثر: عدد النصوص المحفوظة"
              },
              "value": "500 نص"
            },
            {
              "chips": null,
              "key": "card_diff",
              "kind": "number",
              "label": "الفرق المسموح في نص البطاقة",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "أقل: الفرق المسموح في نص البطاقة",
                "plus": "أكثر: الفرق المسموح في نص البطاقة"
              },
              "value": "كلمة واحدة"
            }
          ]
        },
        "sub": "لجميع اللغات",
        "ticked": 0,
        "title": "الخيارات",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 16,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "results": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "rerun",
            "kind": "button",
            "label": "إعادة المقارنة",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "استخدامه لهذه اللغة",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "head": [
            "النموذج",
            "كلمات خاطئة",
            "الانتظار",
            "الذاكرة"
          ],
          "is": "results",
          "rows": [
            {
              "memory": "0,5 غيغابايت",
              "name": "Whisper Small",
              "text": null,
              "tick": false,
              "wait": "2,1 ثانية",
              "wrong": "0 من 28"
            },
            {
              "memory": "0,5 غيغابايت",
              "name": "Whisper Large v3 Turbo",
              "text": null,
              "tick": true,
              "wait": "4,0 ثانية",
              "wrong": "0 من 28"
            },
            {
              "memory": "1,2 غيغابايت",
              "name": "Canary 1b v2",
              "text": null,
              "tick": false,
              "wait": "1,8 ثانية",
              "wrong": "1 من 28"
            }
          ]
        },
        "sub": "12/11/2026 · تلقائي (المعالج)",
        "ticked": 1,
        "title": "مقارنة النماذج",
        "top": {
          "id": "kept",
          "name": null,
          "text": "تُحفَظ هذه الأرقام فقط، لا التسجيل ولا النصوص."
        },
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 10,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "service": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "ok",
            "text": "المفتاح يعمل",
            "warns": false,
            "works": true
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 28,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceAllowance": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "allowance",
            "text": "نفدت الحصة المجانية، وتتجدد يوم 14/11/2026 في الساعة 23:59.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 34,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceAssembly": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "no-test",
            "text": "لا يوجد اختبار مسبق لهذه الخدمة، فسيُختبر المفتاح عند أول إملاء.",
            "warns": false,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في www.assemblyai.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 30,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceCustom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق العنوان",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ العنوان",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "العنوان",
            "dots": 0,
            "fault": "يجب أن يستخدم العنوان HTTPS، مثل https://example.com",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 0,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 31,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق العنوان",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ العنوان",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "العنوان",
            "dots": 0,
            "fault": "يحتوي العنوان على حرف غير مسموح به، مثل المسافة.",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "تطلب الخدمة خطوةً على موقعها، مثل التحقق أو وسيلة دفع أو حدٍّ للإنفاق.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 33,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "servicePasted-address": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق العنوان",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ العنوان",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "العنوان",
            "dots": 0,
            "fault": "لا يوجد نص في الحافظة.",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "تطلب الخدمة خطوةً على موقعها، مثل التحقق أو وسيلة دفع أو حدٍّ للإنفاق.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 38,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "servicePasted-key": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق العنوان",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ العنوان",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "العنوان",
            "dots": 0,
            "fault": "يحتوي العنوان على حرف غير مسموح به، مثل المسافة.",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": "لا يوجد نص في الحافظة.",
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 37,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "servicePasted-model": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق العنوان",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ العنوان",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "العنوان",
            "dots": 0,
            "fault": "يحتوي العنوان على حرف غير مسموح به، مثل المسافة.",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "تطلب الخدمة خطوةً على موقعها، مثل التحقق أو وسيلة دفع أو حدٍّ للإنفاق.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "لا يوجد نص في الحافظة.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 39,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceRate": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "rate",
            "text": "الطلبات كثيرة جدًا. انتظر قليلًا وحاول مرة أخرى.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 35,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceRouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "no-test",
            "text": "لا يوجد اختبار مسبق لهذه الخدمة، فسيُختبر المفتاح عند أول إملاء.",
            "warns": false,
            "works": false
          },
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "الاقتصار على مضيفين لا يحتفظون بشيء",
            "on": true,
            "value": "مفعّل"
          },
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "mistralai/voxtral-small",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 29,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceStrange": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "strange",
            "text": "لم يأتِ الرد من الخدمة، وربما كان صفحة تسجيل دخول. تحقّق من الشبكة.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 36,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    },
    "serviceTyped": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "رجوع",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "إغلاق",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "المعلومات",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "إزالة المفتاح",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "تعطيل",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "line": "يُحفَظ المفتاح مشفّرًا على هذا الحاسوب ولا يُعرض أبدًا.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "ok",
            "text": "المفتاح يعمل",
            "warns": false,
            "works": true
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 12,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ المفتاح",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "المفتاح",
            "dots": 56,
            "fault": null,
            "hint": "اكتب مفتاحك أو الصقه",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 20,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": true,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "لصق",
                "name": "لصق النموذج",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "حفظ",
                "name": "حفظ النموذج",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "إعدادات الإملاء في KeyXtend"
      },
      "place": "above",
      "round": 32,
      "sounds": true,
      "words": {
        "copied": "تم النسخ",
        "gear": "إعدادات الإملاء",
        "timer": "%1:%2",
        "title": "إملاء",
        "window": "شريط الإملاء في KeyXtend"
      }
    }
  },
  "en": {
    "card": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Groq only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, United States",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "United States"
                  }
                ],
                "text": "United States"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Not used for training",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not used for training"
                  }
                ],
                "text": "Not used for training"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": null,
            "said": "Facts read on 17/11/2026.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": null
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 22,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-assemblyai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to AssemblyAI only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, European Union",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "European Union"
                  }
                ],
                "text": "European Union"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Not used for training",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not used for training"
                  }
                ],
                "text": "Not used for training"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing"
                  }
                ],
                "text": "Nothing"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open www.assemblyai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 67,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-custom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to the address you type only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, The address you type",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "The address you type"
                  }
                ],
                "text": "The address you type"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Unknown; it depends on the service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Unknown; it depends on the service"
                  }
                ],
                "text": "Unknown; it depends on the service"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Unknown; it depends on the service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Unknown; it depends on the service"
                  }
                ],
                "text": "Unknown; it depends on the service"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": null,
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 57,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-deepinfra": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to DeepInfra only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, Not stated on its pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated on its pages"
                  }
                ],
                "text": "Not stated on its pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Not used for training",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not used for training"
                  }
                ],
                "text": "Not used for training"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing: held in memory only while it works",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing: held in memory only while it works"
                  }
                ],
                "text": "Nothing: held in memory only while it works"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open docs.deepinfra.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "DeepInfra",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 61,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-elevenlabs": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to ElevenLabs only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, United States",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "United States"
                  }
                ],
                "text": "United States"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Used for training by default",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Used for training by default"
                  }
                ],
                "text": "Used for training by default"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Your history, by default; keeping nothing is for Enterprise plans only",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Your history, by default; keeping nothing is for Enterprise plans only"
                  }
                ],
                "text": "Your history, by default; keeping nothing is for Enterprise plans only"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open elevenlabs.io",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch in your profile menu: "
              },
              {
                "own": true,
                "text": "Terms and privacy → Data use → Improve the models for everyone"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch in your profile menu: Terms and privacy → Data use → Improve the models for everyone"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "ElevenLabs",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 63,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-gemini": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Gemini only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, Not stated on its pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated on its pages"
                  }
                ],
                "text": "Not stated on its pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Used for training while you use the free allowance",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Used for training while you use the free allowance"
                  }
                ],
                "text": "Used for training while you use the free allowance"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, On the paid tier: your requests, for a limited time to check for abuse",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "On the paid tier: your requests, for a limited time to check for abuse"
                  }
                ],
                "text": "On the paid tier: your requests, for a limited time to check for abuse"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open aistudio.google.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "While you use the free allowance, Google uses your voice to improve its products, and human reviewers may read what you say."
              }
            ],
            "text": "While you use the free allowance, Google uses your voice to improve its products, and human reviewers may read what you say."
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Gemini",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 65,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-groq": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Groq only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, United States",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "United States"
                  }
                ],
                "text": "United States"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Not used for training",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not used for training"
                  }
                ],
                "text": "Not used for training"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open console.groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 47,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-lemonfox": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Lemonfox only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, Not stated on its pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated on its pages"
                  }
                ],
                "text": "Not stated on its pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Not stated on its pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated on its pages"
                  }
                ],
                "text": "Not stated on its pages"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing: deleted right after processing",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing: deleted right after processing"
                  }
                ],
                "text": "Nothing: deleted right after processing"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open www.lemonfox.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Lemonfox",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 55,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-mistral": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Mistral only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, European Union",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "European Union"
                  }
                ],
                "text": "European Union"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Not stated; this account switch stops it: Admin panel → Privacy → Anonymous improvement data",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated; this account switch stops it: "
                  },
                  {
                    "own": true,
                    "text": "Admin panel → Privacy → Anonymous improvement data"
                  }
                ],
                "text": "Not stated; this account switch stops it: Admin panel → Privacy → Anonymous improvement data"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Not stated; keeping nothing is only on request",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated; keeping nothing is only on request"
                  }
                ],
                "text": "Not stated; keeping nothing is only on request"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open docs.mistral.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Mistral",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 51,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-openai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to OpenAI only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, United States",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "United States"
                  }
                ],
                "text": "United States"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Only if you opt in",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Only if you opt in"
                  }
                ],
                "text": "Only if you opt in"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing"
                  }
                ],
                "text": "Nothing"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open developers.openai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "OpenAI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 49,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-openrouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to OpenRouter only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, Depends on the host that serves the model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Depends on the host that serves the model"
                  }
                ],
                "text": "Depends on the host that serves the model"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Depends on the host that serves the model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Depends on the host that serves the model"
                  }
                ],
                "text": "Depends on the host that serves the model"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Depends on the host; a switch, on to start, sends only to hosts that keep nothing",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Depends on the host; a switch, on to start, sends only to hosts that keep nothing"
                  }
                ],
                "text": "Depends on the host; a switch, on to start, sends only to hosts that keep nothing"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open openrouter.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 59,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLong-together": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Together AI only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, Not stated on its pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Not stated on its pages"
                  }
                ],
                "text": "Not stated on its pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Only if you opt in",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Only if you opt in"
                  }
                ],
                "text": "Only if you opt in"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Your requests, by default; this account switch stops that: Organization Settings → Privacy → Prompt storage",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Your requests, by default; this account switch stops that: "
                  },
                  {
                    "own": true,
                    "text": "Organization Settings → Privacy → Prompt storage"
                  }
                ],
                "text": "Your requests, by default; this account switch stops that: Organization Settings → Privacy → Prompt storage"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open docs.together.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Together AI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 53,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-assemblyai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to AssemblyAI only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, $50 of credit that never expires, no card needed",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$50 of credit that never expires, no card needed"
                  }
                ],
                "text": "$50 of credit that never expires, no card needed"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $4.50",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$4.50"
                  }
                ],
                "text": "$4.50"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $18",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$18"
                  }
                ],
                "text": "$18"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open www.assemblyai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 68,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-custom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to the address you type only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, Unknown; it depends on the service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Unknown; it depends on the service"
                  }
                ],
                "text": "Unknown; it depends on the service"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, Unknown; it depends on the service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Unknown; it depends on the service"
                  }
                ],
                "text": "Unknown; it depends on the service"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, Unknown; it depends on the service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Unknown; it depends on the service"
                  }
                ],
                "text": "Unknown; it depends on the service"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": null,
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 58,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-deepinfra": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to DeepInfra only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, None",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "None"
                  }
                ],
                "text": "None"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $0.18 to $1.80 by model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$0.18 to $1.80 by model"
                  }
                ],
                "text": "$0.18 to $1.80 by model"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $0.72 to $7.20 by model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$0.72 to $7.20 by model"
                  }
                ],
                "text": "$0.72 to $7.20 by model"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open docs.deepinfra.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "DeepInfra",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 62,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-elevenlabs": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to ElevenLabs only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, About 30 minutes a month",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "About 30 minutes a month"
                  }
                ],
                "text": "About 30 minutes a month"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $6",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$6"
                  }
                ],
                "text": "$6"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $22",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$22"
                  }
                ],
                "text": "$22"
              },
              "warn": false
            },
            {
              "id": "rule",
              "label": "Terms of use",
              "said": "Terms of use, The free plan is for non-commercial use only, with attribution",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "The free plan is for non-commercial use only, with attribution"
                  }
                ],
                "text": "The free plan is for non-commercial use only, with attribution"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open elevenlabs.io",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch in your profile menu: "
              },
              {
                "own": true,
                "text": "Terms and privacy → Data use → Improve the models for everyone"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch in your profile menu: Terms and privacy → Data use → Improve the models for everyone"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "ElevenLabs",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 64,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-gemini": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Gemini only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, No card needed",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "No card needed"
                  }
                ],
                "text": "No card needed"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $4.50 on the paid tier",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$4.50 on the paid tier"
                  }
                ],
                "text": "$4.50 on the paid tier"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $18 on the paid tier",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$18 on the paid tier"
                  }
                ],
                "text": "$18 on the paid tier"
              },
              "warn": false
            },
            {
              "id": "rule",
              "label": "Terms of use",
              "said": "Terms of use, Paid use only in the EEA, Switzerland and the UK; users must be 18 or older",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Paid use only in the EEA, Switzerland and the UK; users must be 18 or older"
                  }
                ],
                "text": "Paid use only in the EEA, Switzerland and the UK; users must be 18 or older"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open aistudio.google.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "While you use the free allowance, Google uses your voice to improve its products, and human reviewers may read what you say."
              }
            ],
            "text": "While you use the free allowance, Google uses your voice to improve its products, and human reviewers may read what you say."
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Gemini",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 66,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-groq": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Groq only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, 8 hours a day, 20 requests a minute",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "8 hours a day, 20 requests a minute"
                  }
                ],
                "text": "8 hours a day, 20 requests a minute"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, Free within the free allowance, else $0.60 to $1.67 by model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Free within the free allowance, else $0.60 to $1.67 by model"
                  }
                ],
                "text": "Free within the free allowance, else $0.60 to $1.67 by model"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, Free within the free allowance, else $2.40 to $6.66 by model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Free within the free allowance, else $2.40 to $6.66 by model"
                  }
                ],
                "text": "Free within the free allowance, else $2.40 to $6.66 by model"
              },
              "warn": false
            },
            {
              "id": "bills",
              "label": "Least billed",
              "said": "Least billed, At least 10 seconds a request, so short pieces cost more",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "At least 10 seconds a request, so short pieces cost more"
                  }
                ],
                "text": "At least 10 seconds a request, so short pieces cost more"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open console.groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 48,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-lemonfox": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Lemonfox only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, The first month",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "The first month"
                  }
                ],
                "text": "The first month"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $5",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$5"
                  }
                ],
                "text": "$5"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $10",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$10"
                  }
                ],
                "text": "$10"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open www.lemonfox.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 21/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 21/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Lemonfox",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 56,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-mistral": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Mistral only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, None",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "None"
                  }
                ],
                "text": "None"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $3.15 to $3.60 by model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$3.15 to $3.60 by model"
                  }
                ],
                "text": "$3.15 to $3.60 by model"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $12.60 to $14.40 by model",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$12.60 to $14.40 by model"
                  }
                ],
                "text": "$12.60 to $14.40 by model"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open docs.mistral.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Mistral",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 52,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-openai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to OpenAI only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, None",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "None"
                  }
                ],
                "text": "None"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $4.05",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$4.05"
                  }
                ],
                "text": "$4.05"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $16.20",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$16.20"
                  }
                ],
                "text": "$16.20"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open developers.openai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "OpenAI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 50,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-openrouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to OpenRouter only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, A small allowance for new accounts",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "A small allowance for new accounts"
                  }
                ],
                "text": "A small allowance for new accounts"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $0.16 to $4.05 by model, plus a 5.5% fee on credit",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$0.16 to $4.05 by model, plus a 5.5% fee on credit"
                  }
                ],
                "text": "$0.16 to $4.05 by model, plus a 5.5% fee on credit"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $0.65 to $16.20 by model, plus a 5.5% fee on credit",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$0.65 to $16.20 by model, plus a 5.5% fee on credit"
                  }
                ],
                "text": "$0.65 to $16.20 by model, plus a 5.5% fee on credit"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open openrouter.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 60,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardLongPrice-together": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirm",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Your voice goes to Together AI only after Stop, with your own key.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Free allowance",
              "said": "Free allowance, None; first purchase at least $15",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "None; first purchase at least $15"
                  }
                ],
                "text": "None; first purchase at least $15"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Light use · 15 hours a month",
              "said": "Light use · 15 hours a month, $1.35",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$1.35"
                  }
                ],
                "text": "$1.35"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Heavy use · 60 hours a month",
              "said": "Heavy use · 60 hours a month, $5.40",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "$5.40"
                  }
                ],
                "text": "$5.40"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open docs.together.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 17/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Together AI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 54,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cardOld": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Where your voice goes",
              "said": "Where your voice goes, United States",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "United States"
                  }
                ],
                "text": "United States"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Training",
              "said": "Training, Unknown; it depends on the service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Unknown; it depends on the service"
                  }
                ],
                "text": "Unknown; it depends on the service"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "What is kept",
              "said": "What is kept, Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "Nothing by default, but checks may keep requests up to 30 days; this account switch turns that off: Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "They may be out of date; check the service’s page.",
            "said": "Facts read on 02/11/2026. They may be out of date; check the service’s page.",
            "text": "Facts read on 02/11/2026."
          },
          "untested": null,
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. To stop it, turn off this switch: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. To stop it, turn off this switch: Settings → Data Controls"
          }
        },
        "sub": "Switched on 10/11/2026",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 23,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "on",
                  "text": "Ready"
                }
              ],
              "name": "Groq",
              "tag": "Untested",
              "tick": true
            },
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "DeepInfra",
              "tag": null,
              "tick": false
            },
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "OpenRouter",
              "tag": null,
              "tick": false
            },
            {
              "line": "An HTTPS address, a model and a key you type",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Needs an address"
                }
              ],
              "name": "Custom address",
              "tag": null,
              "tick": false
            },
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "ElevenLabs",
              "tag": null,
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 19,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudLong-1": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "Groq",
              "tag": "Untested",
              "tick": true
            },
            {
              "line": "Free: None",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "OpenAI",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: None",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "Mistral",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: None; first purchase at least $15",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "Together AI",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: The first month",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "Lemonfox",
              "tag": "Untested",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 41,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudLong-2": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 1,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "An HTTPS address, a model and a key you type",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Needs an address"
                }
              ],
              "name": "Custom address",
              "tag": "Untested",
              "tick": true
            },
            {
              "line": "Free: A small allowance for new accounts",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "OpenRouter",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: None",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "DeepInfra",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: About 30 minutes a month",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "ElevenLabs",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: No card needed",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains on the free allowance"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "Gemini",
              "tag": "Untested",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 42,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudLong-3": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 2,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Free: $50 of credit that never expires, no card needed",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "AssemblyAI",
              "tag": "Untested",
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 43,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudLongOff-1": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "Groq",
              "tag": "Untested",
              "tick": true
            },
            {
              "line": "Free: None",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "OpenAI",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: None",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "Mistral",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: None; first purchase at least $15",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "Together AI",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: The first month",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "Lemonfox",
              "tag": "Untested",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 44,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudLongOff-2": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 1,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "An HTTPS address, a model and a key you type",
              "marks": [
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "Custom address",
              "tag": "Untested",
              "tick": true
            },
            {
              "line": "Free: A small allowance for new accounts",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "OpenRouter",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: None",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "DeepInfra",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: About 30 minutes a month",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "ElevenLabs",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: No card needed",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains on the free allowance"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "Gemini",
              "tag": "Untested",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 45,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudLongOff-3": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 2,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Free: $50 of credit that never expires, no card needed",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "AssemblyAI",
              "tag": "Untested",
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 46,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudOff": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": "Cloud services are off: none is offered and nothing is sent.",
          "rows": [],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": false,
            "value": "Off"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 0,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 21,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "cloudSealed": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Key and model",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Switch on",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "on",
                  "text": "Ready"
                }
              ],
              "name": "Groq",
              "tag": "Untested",
              "tick": false
            },
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "DeepInfra",
              "tag": null,
              "tick": false
            },
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "OpenRouter",
              "tag": null,
              "tick": false
            },
            {
              "line": "An HTTPS address, a model and a key you type",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Needs an address"
                }
              ],
              "name": "Custom address",
              "tag": null,
              "tick": false
            },
            {
              "line": "Free: 8 hours a day, 20 requests a minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Trains by default"
                },
                {
                  "id": "off",
                  "text": "Off"
                }
              ],
              "name": "ElevenLabs",
              "tag": null,
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Use cloud services",
            "on": true,
            "value": "On"
          }
        },
        "sub": "Your own key · nothing leaves before Stop",
        "ticked": 1,
        "title": "Cloud services",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 20,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compare": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "6 ready models · Automatic (processor)",
          "picks": [],
          "recording": null,
          "send": null,
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 8,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, Groq",
              "service": "Groq",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 25,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareCloudUnticked": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, Groq",
              "service": "Groq",
              "tick": false
            },
            {
              "name": "Whisper Large v3",
              "said": "Whisper Large v3, Groq",
              "service": "Groq",
              "tick": false
            }
          ],
          "recording": null,
          "send": null,
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 26,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareEdge": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "mistralai/voxtral-small-24b-2507",
              "said": "mistralai/voxtral-small-24b-2507, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq, Lemonfox, the address you type, OpenRouter, ElevenLabs, Gemini and AssemblyAI at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 73,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareFew": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq, Lemonfox, the address you type, OpenRouter, ElevenLabs, Gemini and AssemblyAI at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 74,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, Groq",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "Whisper Large v3",
              "said": "Whisper Large v3, Groq",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "whisper-large-v3-turbo",
              "said": "whisper-large-v3-turbo, Custom address",
              "service": "Custom address",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq and the address you type at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 69,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareShort": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "mistralai/voxtral-mini-transcribe-3-0",
              "said": "mistralai/voxtral-mini-transcribe-3-0, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe, Gemini",
              "service": "Gemini",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq, Lemonfox, the address you type, OpenRouter, ElevenLabs, Gemini and AssemblyAI at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 72,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareTyped": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
              "said": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm, Custom address",
              "service": "Custom address",
              "tick": true
            },
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe, Gemini",
              "service": "Gemini",
              "tick": true
            },
            {
              "name": "wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww",
              "said": "wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq, Lemonfox, the address you type, OpenRouter, ElevenLabs, Gemini and AssemblyAI at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 71,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "compareWide": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Start reading",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "Read this text aloud once. Each ready model then writes it, one after another.",
          "is": "ready",
          "line": "2 ready models · Automatic (processor)",
          "picks": [
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe, Gemini",
              "service": "Gemini",
              "tick": true
            },
            {
              "name": "Universal-3.6 Pro",
              "said": "Universal-3.6 Pro, AssemblyAI",
              "service": "AssemblyAI",
              "tick": true
            },
            {
              "name": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
              "said": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq, Lemonfox, the address you type, OpenRouter, ElevenLabs, Gemini and AssemblyAI at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 70,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "comparing": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "button",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "running",
          "rows": [
            {
              "mark": {
                "id": "done",
                "text": "Done · 2.1 s"
              },
              "name": "Whisper Small",
              "why": null
            },
            {
              "mark": {
                "id": "run-failed",
                "text": "Failed"
              },
              "name": "Whisper Large v3 Turbo",
              "why": null
            },
            {
              "mark": {
                "id": "writing",
                "text": "Writing…"
              },
              "name": "Canary 1b v2",
              "why": null
            },
            {
              "mark": {
                "id": "waiting",
                "text": "Waiting"
              },
              "name": "Qwen3-ASR 0.6B",
              "why": null
            },
            {
              "mark": {
                "id": "waiting",
                "text": "Waiting"
              },
              "name": "Parakeet TDT 0.6b v3",
              "why": null
            }
          ]
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": {
          "id": "step",
          "name": "Canary 1b v2",
          "text": "Model 3 of 6"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 9,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "comparingCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "button",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "running",
          "rows": [
            {
              "mark": {
                "id": "run-failed",
                "text": "Failed"
              },
              "name": "Whisper Large v3 Turbo",
              "why": "The service refused the key. Check it on the key page."
            },
            {
              "mark": {
                "id": "writing",
                "text": "Writing…"
              },
              "name": "Whisper small",
              "why": null
            }
          ]
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
        "top": {
          "id": "step",
          "name": "Whisper small",
          "text": "Model 2 of 2"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 27,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "device": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "check",
            "kind": "button",
            "label": "Check",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "device",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "device",
          "rows": [
            {
              "check": null,
              "kind": "auto",
              "line": "The processor until a card passes its check",
              "mark": {
                "id": "used",
                "text": "In use"
              },
              "name": "Automatic",
              "tick": false,
              "used": true
            },
            {
              "check": null,
              "kind": "cpu",
              "line": "8 cores · 16 GB memory, 9.4 GB free",
              "mark": null,
              "name": "Processor",
              "tick": false,
              "used": false
            },
            {
              "check": null,
              "kind": "card",
              "line": "Graphics card · 8 GB, 7.1 GB free",
              "mark": {
                "id": "passed",
                "text": "Passed · 9× faster"
              },
              "name": "NVIDIA GeForce RTX 4060 Laptop GPU",
              "tick": false,
              "used": false
            },
            {
              "check": null,
              "kind": "built-in",
              "line": "Built-in graphics · 4 GB shared, 2.6 GB free",
              "mark": {
                "id": "failed",
                "text": "Check failed"
              },
              "name": "AMD Radeon 780M Graphics",
              "tick": false,
              "used": false
            },
            {
              "check": {
                "line": "Checking: warming up the card",
                "share": 0.4
              },
              "kind": "card",
              "line": "Graphics card · 4 GB, 3.8 GB free",
              "mark": null,
              "name": "Intel Arc A370M",
              "tick": true,
              "used": false
            }
          ]
        },
        "sub": "Cards are checked before use",
        "ticked": 1,
        "title": "Device",
        "top": {
          "id": "failed",
          "name": null,
          "text": "AMD Radeon 780M Graphics: check failed. Its text differed by 3 words; 1 word allowed."
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 7,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "folders": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "here",
            "kind": "button",
            "label": "Use this folder",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "folders",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "folders",
          "rows": [
            "C:\\",
            "D:\\"
          ]
        },
        "sub": "Drives",
        "ticked": 0,
        "title": "Models folder",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 18,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "freeResults": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "rerun",
            "kind": "button",
            "label": "Compare again",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "head": [
            "Model",
            "Words wrong",
            "Wait",
            "Memory"
          ],
          "is": "results",
          "rows": [
            {
              "memory": "0.5 GB",
              "name": "Whisper Small",
              "text": "The evening train leaves at seven.",
              "tick": false,
              "wait": "2.1 s",
              "wrong": null
            },
            {
              "memory": "0.5 GB",
              "name": "Whisper Large v3 Turbo",
              "text": "The evening train leaves at seven.",
              "tick": true,
              "wait": "4.0 s",
              "wrong": null
            },
            {
              "memory": "1.2 GB",
              "name": "Canary 1b v2",
              "text": "The evening train leaves at seven.",
              "tick": false,
              "wait": "1.8 s",
              "wrong": null
            }
          ]
        },
        "sub": "12/11/2026 · Automatic (processor)",
        "ticked": 1,
        "title": "Compare models",
        "top": {
          "id": "kept",
          "name": null,
          "text": "Only these numbers are kept, not the recording or the texts."
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 11,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "history": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "Delete all",
          "on": true,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "button",
            "label": "Copy",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "pin",
            "kind": "button",
            "label": "Pin",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "insert",
            "kind": "button",
            "label": "Insert (2)",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "empty": null,
          "is": "history",
          "rows": [
            {
              "arabic": false,
              "id": 12,
              "meta": "Today 14:32 · English · Parakeet TDT 0.6b v3",
              "name": "Thanks for the update. I will send the signed form on Thursday., Today 14:32 · English · Parakeet TDT 0.6b v3, Ticked 2",
              "pinned": false,
              "rtl": false,
              "text": "Thanks for the update. I will send the signed form on Thursday.",
              "tick": 2
            },
            {
              "arabic": false,
              "id": 11,
              "meta": "Today 11:32 · French · Parakeet TDT 0.6b v3",
              "name": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures., Today 11:32 · French · Parakeet TDT 0.6b v3, Ticked 1, Pinned",
              "pinned": true,
              "rtl": false,
              "text": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
              "tick": 1
            },
            {
              "arabic": true,
              "id": 10,
              "meta": "Today 09:32 · Arabic · Cohere Transcribe Arabic",
              "name": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع., Today 09:32 · Arabic · Cohere Transcribe Arabic",
              "pinned": false,
              "rtl": true,
              "text": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
              "tick": null
            }
          ],
          "unpin": false
        },
        "sub": "12 texts · newest first",
        "ticked": 2,
        "title": "History",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 12,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "historyAsk": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "Delete all",
          "on": false,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Yes",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "No",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Delete 11 texts? Pinned ones stay.",
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "empty": null,
          "is": "history",
          "rows": [
            {
              "arabic": false,
              "id": 12,
              "meta": "Today 14:32 · English · Parakeet TDT 0.6b v3",
              "name": "Thanks for the update. I will send the signed form on Thursday., Today 14:32 · English · Parakeet TDT 0.6b v3, Ticked 2",
              "pinned": false,
              "rtl": false,
              "text": "Thanks for the update. I will send the signed form on Thursday.",
              "tick": 2
            },
            {
              "arabic": false,
              "id": 11,
              "meta": "Today 11:32 · French · Parakeet TDT 0.6b v3",
              "name": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures., Today 11:32 · French · Parakeet TDT 0.6b v3, Ticked 1, Pinned",
              "pinned": true,
              "rtl": false,
              "text": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
              "tick": 1
            },
            {
              "arabic": true,
              "id": 10,
              "meta": "Today 09:32 · Arabic · Cohere Transcribe Arabic",
              "name": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع., Today 09:32 · Arabic · Cohere Transcribe Arabic",
              "pinned": false,
              "rtl": true,
              "text": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
              "tick": null
            }
          ],
          "unpin": false
        },
        "sub": "12 texts · newest first",
        "ticked": 0,
        "title": "History",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 13,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "historyEmpty": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "Delete all",
          "on": false,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "button",
            "label": "Copy",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "pin",
            "kind": "button",
            "label": "Pin",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "insert",
            "kind": "button",
            "label": "Insert",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "empty": "No texts yet. Dictate something, and it shows here.",
          "is": "history",
          "rows": [],
          "unpin": false
        },
        "sub": "0 texts · newest first",
        "ticked": 0,
        "title": "History",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 14,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "home": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": null,
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "again",
            "kind": "button",
            "label": "Try again",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Transcribes the last recording again with the choices here.",
        "note": null,
        "page": "home",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "home",
          "rows": [
            {
              "label": "Models",
              "tab": "models",
              "value": "Parakeet TDT 0.6b v3"
            },
            {
              "label": "Cloud services",
              "tab": "cloud",
              "value": "Off"
            },
            {
              "label": "Device",
              "tab": "device",
              "value": "Automatic (processor)"
            },
            {
              "label": "Compare models",
              "tab": "compare",
              "value": "Last run: 12/11/2026"
            },
            {
              "label": "History",
              "tab": "history",
              "value": "12 texts"
            }
          ]
        },
        "sub": "Language: English",
        "ticked": 0,
        "title": "Dictation",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 6,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "models": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete file",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Download",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Change",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Open",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Models folder: D:\\KeyXtend\\models · 222 GB free"
          },
          "rows": [
            {
              "fetch": null,
              "line": "776 MB",
              "marks": [
                {
                  "id": "used",
                  "text": "In use"
                },
                {
                  "id": "default",
                  "text": "Default"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 MB",
              "marks": [
                {
                  "id": "ready",
                  "text": "Ready"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "2.53 GB",
              "marks": [
                {
                  "id": "download",
                  "text": "Not downloaded"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "Pause",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Cancel",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 MB of 1.22 GB",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Needs 3.2 GB of memory, 2.8 GB free",
              "marks": [
                {
                  "id": "not-here",
                  "text": "Not for this PC"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "For English",
        "ticked": 1,
        "title": "Models",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Recommended for English:"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 1,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "modelsCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete file",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Download",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "Cloud services",
              "on": true,
              "tone": "plain"
            },
            "text": "Models of the services you switched on; nothing leaves before Stop."
          },
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "In the cloud, with your key",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "ready",
                  "text": "Ready"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "No host of this model promises to keep nothing.",
              "marks": [
                {
                  "id": "cloud",
                  "text": "OpenRouter"
                },
                {
                  "id": "unavailable",
                  "text": "Unavailable"
                }
              ],
              "name": "voxtral-small",
              "tick": false
            },
            {
              "fetch": null,
              "line": "In the cloud, with your key",
              "marks": [
                {
                  "id": "cloud",
                  "text": "DeepInfra"
                },
                {
                  "id": "needs-key",
                  "text": "Needs a key"
                }
              ],
              "name": "Nemotron 3.5 Transcribe",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "cloud"
          },
          "slow": null,
          "untested": null
        },
        "sub": "For English",
        "ticked": 1,
        "title": "Models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 24,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "modelsLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete file",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Download",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "Cloud services",
              "on": true,
              "tone": "plain"
            },
            "text": "Models of the services you switched on; nothing leaves before Stop."
          },
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "In the cloud, with your key",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "used",
                  "text": "In use"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "In the cloud, with your key",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "ready",
                  "text": "Ready"
                }
              ],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "In the cloud, with your key",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Custom address"
                },
                {
                  "id": "needs-address",
                  "text": "Needs an address"
                }
              ],
              "name": "whisper-large-v3-turbo",
              "tick": false
            },
            {
              "fetch": null,
              "line": "No host of this model promises to keep nothing.",
              "marks": [
                {
                  "id": "cloud",
                  "text": "OpenRouter"
                },
                {
                  "id": "unavailable",
                  "text": "Unavailable"
                }
              ],
              "name": "voxtral-small",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "cloud"
          },
          "slow": null,
          "untested": null
        },
        "sub": "For English",
        "ticked": 1,
        "title": "Models",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 40,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "modelsMore": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete file",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Download",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": {
          "id": "net",
          "text": "The download did not finish. Try again later."
        },
        "page": "models",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Change",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Open",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Models folder: D:\\KeyXtend\\models · 222 GB free"
          },
          "rows": [
            {
              "fetch": null,
              "line": "1.6 GB",
              "marks": [
                {
                  "id": "other",
                  "text": "Other file, not checked"
                }
              ],
              "name": "ggml-medium.bin",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 1,
                    "id": "resume",
                    "kind": "fetch",
                    "label": "Resume",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 1,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Cancel",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": true,
                "share": 0.2
              },
              "line": "Paused · 239 MB of 1.2 GB",
              "marks": [],
              "name": "Canary 1b v2",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Models",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Recommended for English:"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 2,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "modelsOffer": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Yes",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "No",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Keep models in D:\\KeyXtend\\models? 222 GB free.",
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "776 MB",
              "marks": [
                {
                  "id": "used",
                  "text": "In use"
                },
                {
                  "id": "default",
                  "text": "Default"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 MB",
              "marks": [
                {
                  "id": "ready",
                  "text": "Ready"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "2.53 GB",
              "marks": [
                {
                  "id": "download",
                  "text": "Not downloaded"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "Pause",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Cancel",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 MB of 1.22 GB",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Needs 3.2 GB of memory, 2.8 GB free",
              "marks": [
                {
                  "id": "not-here",
                  "text": "Not for this PC"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Models",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Recommended for English:"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 5,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "modelsSlow": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete file",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Download",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Change",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Open",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Models folder: D:\\KeyXtend\\models · 222 GB free"
          },
          "rows": [
            {
              "fetch": null,
              "line": "Needs 5 GB of memory, 2.8 GB free",
              "marks": [
                {
                  "id": "default",
                  "text": "Default"
                },
                {
                  "id": "not-here",
                  "text": "Not for this PC"
                }
              ],
              "name": "Cohere Transcribe Arabic",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 MB",
              "marks": [
                {
                  "id": "ready",
                  "text": "Ready"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "local"
          },
          "slow": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "Cloud services",
              "on": true,
              "tone": "plain"
            },
            "text": "Local models for English will be very slow on this PC."
          },
          "untested": "This order is not tested for English."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Models",
        "top": {
          "id": "best",
          "name": "Cohere Transcribe Arabic",
          "text": "Recommended for English:"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 4,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "modelsUsed": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Delete file",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Download",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Change",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Open",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Models folder: D:\\KeyXtend\\models · 222 GB free"
          },
          "rows": [
            {
              "fetch": null,
              "line": "776 MB",
              "marks": [
                {
                  "id": "used",
                  "text": "In use"
                },
                {
                  "id": "default",
                  "text": "Default"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": true
            },
            {
              "fetch": null,
              "line": "650 MB",
              "marks": [
                {
                  "id": "ready",
                  "text": "Ready"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": false
            },
            {
              "fetch": null,
              "line": "2.53 GB",
              "marks": [
                {
                  "id": "download",
                  "text": "Not downloaded"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "Pause",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Cancel",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 MB of 1.22 GB",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Needs 3.2 GB of memory, 2.8 GB free",
              "marks": [
                {
                  "id": "not-here",
                  "text": "Not for this PC"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Local",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Where the model runs",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "For English",
        "ticked": 1,
        "title": "Models",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Recommended for English:"
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 3,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "options": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Changes apply at once.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 0,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "sounds",
              "kind": "switch",
              "label": "Start and stop sounds",
              "live": true,
              "on": true,
              "steps": null,
              "value": "On"
            },
            {
              "chips": null,
              "key": "silence_stop",
              "kind": "switch",
              "label": "Stop after silence",
              "live": true,
              "on": false,
              "steps": null,
              "value": "Off"
            },
            {
              "chips": null,
              "key": "silence_stop_s",
              "kind": "number",
              "label": "Silence before stopping",
              "live": false,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Less: Silence before stopping",
                "plus": "More: Silence before stopping"
              },
              "value": "3 s"
            },
            {
              "chips": null,
              "key": "done_ms",
              "kind": "number",
              "label": "Text stays on the bar",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Less: Text stays on the bar",
                "plus": "More: Text stays on the bar"
              },
              "value": "9 s"
            },
            {
              "chips": null,
              "key": "place",
              "kind": "choice",
              "label": "Bar position",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": true,
                "minus": "Less: Bar position",
                "plus": "More: Bar position"
              },
              "value": "Above the keyboard"
            }
          ]
        },
        "sub": "For every language",
        "ticked": 0,
        "title": "Options",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 15,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "optionsInsert": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Changes apply at once.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 2,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "debounce_ms",
              "kind": "number",
              "label": "Ignore a second click within",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Less: Ignore a second click within",
                "plus": "More: Ignore a second click within"
              },
              "value": "400 ms"
            },
            {
              "chips": [
                {
                  "label": "Type it",
                  "on": false
                },
                {
                  "label": "Paste at once",
                  "on": true
                }
              ],
              "key": "insert",
              "kind": "pair",
              "label": "Insert the text",
              "live": true,
              "on": false,
              "steps": null,
              "value": "Paste at once"
            },
            {
              "chips": null,
              "key": "insert",
              "kind": "hint",
              "label": "Uses the clipboard, then puts yours back; a password box is always typed.",
              "live": true,
              "on": false,
              "steps": null,
              "value": ""
            },
            {
              "chips": null,
              "key": "type_suffix",
              "kind": "switch",
              "label": "Space after the text",
              "live": true,
              "on": true,
              "steps": null,
              "value": "On"
            },
            {
              "chips": null,
              "key": "type_gap_ms",
              "kind": "number",
              "label": "Pause between letters",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Less: Pause between letters",
                "plus": "More: Pause between letters"
              },
              "value": "5 ms"
            }
          ]
        },
        "sub": "For every language",
        "ticked": 0,
        "title": "Options",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 17,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "optionsMore": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "Changes apply at once.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 1,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "threads",
              "kind": "number",
              "label": "Processor threads",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": true,
                "minus": "Less: Processor threads",
                "plus": "More: Processor threads"
              },
              "value": "Automatic"
            },
            {
              "chips": null,
              "key": "type_suffix",
              "kind": "switch",
              "label": "Space after the text",
              "live": true,
              "on": true,
              "steps": null,
              "value": "On"
            },
            {
              "chips": null,
              "key": "card_gain",
              "kind": "number",
              "label": "A card must be faster by",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Less: A card must be faster by",
                "plus": "More: A card must be faster by"
              },
              "value": "1.5×"
            },
            {
              "chips": null,
              "key": "history_max",
              "kind": "number",
              "label": "Texts kept",
              "live": true,
              "on": false,
              "steps": {
                "high": true,
                "low": false,
                "minus": "Less: Texts kept",
                "plus": "More: Texts kept"
              },
              "value": "500 texts"
            },
            {
              "chips": null,
              "key": "card_diff",
              "kind": "number",
              "label": "A card’s text may differ by",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Less: A card’s text may differ by",
                "plus": "More: A card’s text may differ by"
              },
              "value": "1 word"
            }
          ]
        },
        "sub": "For every language",
        "ticked": 0,
        "title": "Options",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 16,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "results": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "rerun",
            "kind": "button",
            "label": "Compare again",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Use for this language",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "head": [
            "Model",
            "Words wrong",
            "Wait",
            "Memory"
          ],
          "is": "results",
          "rows": [
            {
              "memory": "0.5 GB",
              "name": "Whisper Small",
              "text": null,
              "tick": false,
              "wait": "2.1 s",
              "wrong": "0 of 28"
            },
            {
              "memory": "0.5 GB",
              "name": "Whisper Large v3 Turbo",
              "text": null,
              "tick": true,
              "wait": "4.0 s",
              "wrong": "0 of 28"
            },
            {
              "memory": "1.2 GB",
              "name": "Canary 1b v2",
              "text": null,
              "tick": false,
              "wait": "1.8 s",
              "wrong": "1 of 28"
            }
          ]
        },
        "sub": "12/11/2026 · Automatic (processor)",
        "ticked": 1,
        "title": "Compare models",
        "top": {
          "id": "kept",
          "name": null,
          "text": "Only these numbers are kept, not the recording or the texts."
        },
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 10,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "service": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "ok",
            "text": "Key works",
            "warns": false,
            "works": true
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 28,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceAllowance": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "allowance",
            "text": "The free allowance is used up. It renews on 14/11/2026 at 23:59.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 34,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceAssembly": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "no-test",
            "text": "No test call here: the first dictation tests the key.",
            "warns": false,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at www.assemblyai.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 30,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceCustom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the address",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Save",
                "name": "Save the address",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Address",
            "dots": 0,
            "fault": "The address must use HTTPS, such as https://example.com",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 0,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": "This model id is too long or holds a character that is not allowed.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 31,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the address",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Save",
                "name": "Save the address",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Address",
            "dots": 0,
            "fault": "The address holds a character that is not allowed, such as a space.",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "The service asks for a step on its site, such as a verification, a payment method or a spending cap.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": "This model id is too long or holds a character that is not allowed.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 33,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "servicePasted-address": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the address",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Save",
                "name": "Save the address",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Address",
            "dots": 0,
            "fault": "The clipboard holds no text.",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "The service asks for a step on its site, such as a verification, a payment method or a spending cap.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": "This model id is too long or holds a character that is not allowed.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 38,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "servicePasted-key": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the address",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Save",
                "name": "Save the address",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Address",
            "dots": 0,
            "fault": "The address holds a character that is not allowed, such as a space.",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": "The clipboard holds no text.",
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": "This model id is too long or holds a character that is not allowed.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 37,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "servicePasted-model": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the address",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Save",
                "name": "Save the address",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Address",
            "dots": 0,
            "fault": "The address holds a character that is not allowed, such as a space.",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "The service asks for a step on its site, such as a verification, a payment method or a spending cap.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": "The clipboard holds no text.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 39,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceRate": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "rate",
            "text": "Too many requests. Wait a moment and try again.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 35,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceRouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "no-test",
            "text": "No test call here: the first dictation tests the key.",
            "warns": false,
            "works": false
          },
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "Only hosts that keep nothing",
            "on": true,
            "value": "On"
          },
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": "No host of this model promises to keep nothing.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "mistralai/voxtral-small",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 29,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceStrange": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "strange",
            "text": "The reply was not the service’s, maybe a sign-in page. Check the network.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 36,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    },
    "serviceTyped": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Back",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Close",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Facts",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Remove key",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Switch off",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "line": "The key is kept encrypted on this PC and never shown.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "ok",
            "text": "Key works",
            "warns": false,
            "works": true
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 12,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Save",
                "name": "Save the key",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Key",
            "dots": 56,
            "fault": null,
            "hint": "Type or paste your key",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 20,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": true,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Paste",
                "name": "Paste the model",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Save",
                "name": "Save the model",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "KeyXtend dictation settings"
      },
      "place": "above",
      "round": 32,
      "sounds": true,
      "words": {
        "copied": "Copied",
        "gear": "Dictation settings",
        "timer": "%1:%2",
        "title": "Dictation",
        "window": "KeyXtend dictation bar"
      }
    }
  },
  "fr": {
    "card": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Groq qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, États-Unis",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "États-Unis"
                  }
                ],
                "text": "États-Unis"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Non utilisée pour l’entraînement",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non utilisée pour l’entraînement"
                  }
                ],
                "text": "Non utilisée pour l’entraînement"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": null,
            "said": "Informations relevées le 17/11/2026.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": null
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 22,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-assemblyai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers AssemblyAI qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Union européenne",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Union européenne"
                  }
                ],
                "text": "Union européenne"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Non utilisée pour l’entraînement",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non utilisée pour l’entraînement"
                  }
                ],
                "text": "Non utilisée pour l’entraînement"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien"
                  }
                ],
                "text": "Rien"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir www.assemblyai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 67,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-custom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers l’adresse que vous saisissez qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, L’adresse que vous saisissez",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "L’adresse que vous saisissez"
                  }
                ],
                "text": "L’adresse que vous saisissez"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Inconnu ; cela dépend du service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Inconnu ; cela dépend du service"
                  }
                ],
                "text": "Inconnu ; cela dépend du service"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Inconnu ; cela dépend du service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Inconnu ; cela dépend du service"
                  }
                ],
                "text": "Inconnu ; cela dépend du service"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": null,
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 57,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-deepinfra": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers DeepInfra qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Non précisé sur ses pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé sur ses pages"
                  }
                ],
                "text": "Non précisé sur ses pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Non utilisée pour l’entraînement",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non utilisée pour l’entraînement"
                  }
                ],
                "text": "Non utilisée pour l’entraînement"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien : gardé en mémoire le temps du traitement seulement",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien : gardé en mémoire le temps du traitement seulement"
                  }
                ],
                "text": "Rien : gardé en mémoire le temps du traitement seulement"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir docs.deepinfra.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "DeepInfra",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 61,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-elevenlabs": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers ElevenLabs qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, États-Unis",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "États-Unis"
                  }
                ],
                "text": "États-Unis"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Utilisée pour l’entraînement par défaut",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Utilisée pour l’entraînement par défaut"
                  }
                ],
                "text": "Utilisée pour l’entraînement par défaut"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Votre historique, par défaut ; ne rien garder est réservé aux offres Enterprise",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Votre historique, par défaut ; ne rien garder est réservé aux offres Enterprise"
                  }
                ],
                "text": "Votre historique, par défaut ; ne rien garder est réservé aux offres Enterprise"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir elevenlabs.io",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage dans le menu de votre profil : "
              },
              {
                "own": true,
                "text": "Terms and privacy → Data use → Improve the models for everyone"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage dans le menu de votre profil : Terms and privacy → Data use → Improve the models for everyone"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "ElevenLabs",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 63,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-gemini": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Gemini qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Non précisé sur ses pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé sur ses pages"
                  }
                ],
                "text": "Non précisé sur ses pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Utilisée pour l’entraînement tant que vous utilisez le quota gratuit",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Utilisée pour l’entraînement tant que vous utilisez le quota gratuit"
                  }
                ],
                "text": "Utilisée pour l’entraînement tant que vous utilisez le quota gratuit"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Au niveau payant : vos requêtes, pendant un temps limité pour détecter les abus",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Au niveau payant : vos requêtes, pendant un temps limité pour détecter les abus"
                  }
                ],
                "text": "Au niveau payant : vos requêtes, pendant un temps limité pour détecter les abus"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir aistudio.google.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Tant que vous utilisez le quota gratuit, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites."
              }
            ],
            "text": "Tant que vous utilisez le quota gratuit, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites."
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Gemini",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 65,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-groq": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Groq qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, États-Unis",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "États-Unis"
                  }
                ],
                "text": "États-Unis"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Non utilisée pour l’entraînement",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non utilisée pour l’entraînement"
                  }
                ],
                "text": "Non utilisée pour l’entraînement"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir console.groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 47,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-lemonfox": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Lemonfox qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Non précisé sur ses pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé sur ses pages"
                  }
                ],
                "text": "Non précisé sur ses pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Non précisé sur ses pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé sur ses pages"
                  }
                ],
                "text": "Non précisé sur ses pages"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien : tout est supprimé juste après le traitement",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien : tout est supprimé juste après le traitement"
                  }
                ],
                "text": "Rien : tout est supprimé juste après le traitement"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir www.lemonfox.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Lemonfox",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 55,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-mistral": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Mistral qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Union européenne",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Union européenne"
                  }
                ],
                "text": "Union européenne"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Non précisé ; ce réglage du compte l’arrête : Admin panel → Privacy → Anonymous improvement data",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé ; ce réglage du compte l’arrête : "
                  },
                  {
                    "own": true,
                    "text": "Admin panel → Privacy → Anonymous improvement data"
                  }
                ],
                "text": "Non précisé ; ce réglage du compte l’arrête : Admin panel → Privacy → Anonymous improvement data"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Non précisé ; ne rien garder n’est possible que sur demande",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé ; ne rien garder n’est possible que sur demande"
                  }
                ],
                "text": "Non précisé ; ne rien garder n’est possible que sur demande"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir docs.mistral.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Mistral",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 51,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-openai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers OpenAI qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, États-Unis",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "États-Unis"
                  }
                ],
                "text": "États-Unis"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Seulement si vous l’acceptez",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Seulement si vous l’acceptez"
                  }
                ],
                "text": "Seulement si vous l’acceptez"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien"
                  }
                ],
                "text": "Rien"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir developers.openai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "OpenAI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 49,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-openrouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers OpenRouter qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Dépend de l’hébergeur qui sert le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Dépend de l’hébergeur qui sert le modèle"
                  }
                ],
                "text": "Dépend de l’hébergeur qui sert le modèle"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Dépend de l’hébergeur qui sert le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Dépend de l’hébergeur qui sert le modèle"
                  }
                ],
                "text": "Dépend de l’hébergeur qui sert le modèle"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Dépend de l’hébergeur ; un réglage, activé d’emblée, n’envoie qu’aux hébergeurs qui ne gardent rien",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Dépend de l’hébergeur ; un réglage, activé d’emblée, n’envoie qu’aux hébergeurs qui ne gardent rien"
                  }
                ],
                "text": "Dépend de l’hébergeur ; un réglage, activé d’emblée, n’envoie qu’aux hébergeurs qui ne gardent rien"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir openrouter.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 59,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLong-together": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Together AI qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, Non précisé sur ses pages",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Non précisé sur ses pages"
                  }
                ],
                "text": "Non précisé sur ses pages"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Seulement si vous l’acceptez",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Seulement si vous l’acceptez"
                  }
                ],
                "text": "Seulement si vous l’acceptez"
              },
              "warn": false
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Vos requêtes, par défaut ; ce réglage du compte l’arrête : Organization Settings → Privacy → Prompt storage",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Vos requêtes, par défaut ; ce réglage du compte l’arrête : "
                  },
                  {
                    "own": true,
                    "text": "Organization Settings → Privacy → Prompt storage"
                  }
                ],
                "text": "Vos requêtes, par défaut ; ce réglage du compte l’arrête : Organization Settings → Privacy → Prompt storage"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir docs.together.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Together AI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 53,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-assemblyai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers AssemblyAI qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, 50 $ de crédit sans expiration, sans carte",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "50 $ de crédit sans expiration, sans carte"
                  }
                ],
                "text": "50 $ de crédit sans expiration, sans carte"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, 4,50 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "4,50 $"
                  }
                ],
                "text": "4,50 $"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, 18 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "18 $"
                  }
                ],
                "text": "18 $"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir www.assemblyai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 68,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-custom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers l’adresse que vous saisissez qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Inconnu ; cela dépend du service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Inconnu ; cela dépend du service"
                  }
                ],
                "text": "Inconnu ; cela dépend du service"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, Inconnu ; cela dépend du service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Inconnu ; cela dépend du service"
                  }
                ],
                "text": "Inconnu ; cela dépend du service"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, Inconnu ; cela dépend du service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Inconnu ; cela dépend du service"
                  }
                ],
                "text": "Inconnu ; cela dépend du service"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": null,
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 58,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-deepinfra": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers DeepInfra qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Aucun",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Aucun"
                  }
                ],
                "text": "Aucun"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, De 0,18 $ à 1,80 $ selon le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "De 0,18 $ à 1,80 $ selon le modèle"
                  }
                ],
                "text": "De 0,18 $ à 1,80 $ selon le modèle"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, De 0,72 $ à 7,20 $ selon le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "De 0,72 $ à 7,20 $ selon le modèle"
                  }
                ],
                "text": "De 0,72 $ à 7,20 $ selon le modèle"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir docs.deepinfra.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "DeepInfra",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 62,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-elevenlabs": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers ElevenLabs qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Environ 30 minutes par mois",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Environ 30 minutes par mois"
                  }
                ],
                "text": "Environ 30 minutes par mois"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, 6 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "6 $"
                  }
                ],
                "text": "6 $"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, 22 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "22 $"
                  }
                ],
                "text": "22 $"
              },
              "warn": false
            },
            {
              "id": "rule",
              "label": "Conditions d’utilisation",
              "said": "Conditions d’utilisation, L’offre gratuite est réservée à un usage non commercial, avec mention de la source",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "L’offre gratuite est réservée à un usage non commercial, avec mention de la source"
                  }
                ],
                "text": "L’offre gratuite est réservée à un usage non commercial, avec mention de la source"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir elevenlabs.io",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage dans le menu de votre profil : "
              },
              {
                "own": true,
                "text": "Terms and privacy → Data use → Improve the models for everyone"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage dans le menu de votre profil : Terms and privacy → Data use → Improve the models for everyone"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "ElevenLabs",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 64,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-gemini": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Gemini qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Sans carte bancaire",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Sans carte bancaire"
                  }
                ],
                "text": "Sans carte bancaire"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, 4,50 $ au niveau payant",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "4,50 $ au niveau payant"
                  }
                ],
                "text": "4,50 $ au niveau payant"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, 18 $ au niveau payant",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "18 $ au niveau payant"
                  }
                ],
                "text": "18 $ au niveau payant"
              },
              "warn": false
            },
            {
              "id": "rule",
              "label": "Conditions d’utilisation",
              "said": "Conditions d’utilisation, Usage payant seulement dans l’EEE, en Suisse et au Royaume-Uni ; utilisateurs de 18 ans ou plus",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Usage payant seulement dans l’EEE, en Suisse et au Royaume-Uni ; utilisateurs de 18 ans ou plus"
                  }
                ],
                "text": "Usage payant seulement dans l’EEE, en Suisse et au Royaume-Uni ; utilisateurs de 18 ans ou plus"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir aistudio.google.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Tant que vous utilisez le quota gratuit, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites."
              }
            ],
            "text": "Tant que vous utilisez le quota gratuit, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites."
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Gemini",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 66,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-groq": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Groq qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, 8 heures par jour, 20 requêtes par minute",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "8 heures par jour, 20 requêtes par minute"
                  }
                ],
                "text": "8 heures par jour, 20 requêtes par minute"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, Gratuit dans la limite du quota gratuit, sinon de 0,60 $ à 1,67 $ selon le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Gratuit dans la limite du quota gratuit, sinon de 0,60 $ à 1,67 $ selon le modèle"
                  }
                ],
                "text": "Gratuit dans la limite du quota gratuit, sinon de 0,60 $ à 1,67 $ selon le modèle"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, Gratuit dans la limite du quota gratuit, sinon de 2,40 $ à 6,66 $ selon le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Gratuit dans la limite du quota gratuit, sinon de 2,40 $ à 6,66 $ selon le modèle"
                  }
                ],
                "text": "Gratuit dans la limite du quota gratuit, sinon de 2,40 $ à 6,66 $ selon le modèle"
              },
              "warn": false
            },
            {
              "id": "bills",
              "label": "Facturation minimale",
              "said": "Facturation minimale, Au moins 10 secondes par requête ; les morceaux courts coûtent donc plus cher",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Au moins 10 secondes par requête ; les morceaux courts coûtent donc plus cher"
                  }
                ],
                "text": "Au moins 10 secondes par requête ; les morceaux courts coûtent donc plus cher"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir console.groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 48,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-lemonfox": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Lemonfox qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Le premier mois",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Le premier mois"
                  }
                ],
                "text": "Le premier mois"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, 5 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "5 $"
                  }
                ],
                "text": "5 $"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, 10 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "10 $"
                  }
                ],
                "text": "10 $"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir www.lemonfox.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 21/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 21/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Lemonfox",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 56,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-mistral": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Mistral qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Aucun",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Aucun"
                  }
                ],
                "text": "Aucun"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, De 3,15 $ à 3,60 $ selon le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "De 3,15 $ à 3,60 $ selon le modèle"
                  }
                ],
                "text": "De 3,15 $ à 3,60 $ selon le modèle"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, De 12,60 $ à 14,40 $ selon le modèle",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "De 12,60 $ à 14,40 $ selon le modèle"
                  }
                ],
                "text": "De 12,60 $ à 14,40 $ selon le modèle"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir docs.mistral.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Mistral",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 52,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-openai": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers OpenAI qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Aucun",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Aucun"
                  }
                ],
                "text": "Aucun"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, 4,05 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "4,05 $"
                  }
                ],
                "text": "4,05 $"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, 16,20 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "16,20 $"
                  }
                ],
                "text": "16,20 $"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir developers.openai.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "OpenAI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 50,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-openrouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers OpenRouter qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Un petit quota pour les nouveaux comptes",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Un petit quota pour les nouveaux comptes"
                  }
                ],
                "text": "Un petit quota pour les nouveaux comptes"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, De 0,16 $ à 4,05 $ selon le modèle, plus 5,5 % de frais sur le crédit",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "De 0,16 $ à 4,05 $ selon le modèle, plus 5,5 % de frais sur le crédit"
                  }
                ],
                "text": "De 0,16 $ à 4,05 $ selon le modèle, plus 5,5 % de frais sur le crédit"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, De 0,65 $ à 16,20 $ selon le modèle, plus 5,5 % de frais sur le crédit",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "De 0,65 $ à 16,20 $ selon le modèle, plus 5,5 % de frais sur le crédit"
                  }
                ],
                "text": "De 0,65 $ à 16,20 $ selon le modèle, plus 5,5 % de frais sur le crédit"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir openrouter.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 60,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardLongPrice-together": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Confirmer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Votre voix ne part vers Together AI qu’après l’arrêt, avec votre propre clé.",
        "note": null,
        "page": "card",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "facts": [
            {
              "id": "free",
              "label": "Quota gratuit",
              "said": "Quota gratuit, Aucun ; premier achat d’au moins 15 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Aucun ; premier achat d’au moins 15 $"
                  }
                ],
                "text": "Aucun ; premier achat d’au moins 15 $"
              },
              "warn": false
            },
            {
              "id": "light",
              "label": "Usage léger · 15 heures par mois",
              "said": "Usage léger · 15 heures par mois, 1,35 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "1,35 $"
                  }
                ],
                "text": "1,35 $"
              },
              "warn": false
            },
            {
              "id": "heavy",
              "label": "Usage intensif · 60 heures par mois",
              "said": "Usage intensif · 60 heures par mois, 5,40 $",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "5,40 $"
                  }
                ],
                "text": "5,40 $"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir docs.together.ai",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 17/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Together AI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 54,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cardOld": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "card",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "facts": [
            {
              "id": "where",
              "label": "Où va votre voix",
              "said": "Où va votre voix, États-Unis",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "États-Unis"
                  }
                ],
                "text": "États-Unis"
              },
              "warn": false
            },
            {
              "id": "trains",
              "label": "Entraînement",
              "said": "Entraînement, Inconnu ; cela dépend du service",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Inconnu ; cela dépend du service"
                  }
                ],
                "text": "Inconnu ; cela dépend du service"
              },
              "warn": true
            },
            {
              "id": "keeps",
              "label": "Conservation",
              "said": "Conservation, Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : Settings → Data Controls",
              "value": {
                "parts": [
                  {
                    "own": false,
                    "text": "Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : "
                  },
                  {
                    "own": true,
                    "text": "Settings → Data Controls"
                  }
                ],
                "text": "Rien par défaut, mais les requêtes peuvent être gardées jusqu’à 30 jours pour contrôle ; ce réglage du compte l’empêche : Settings → Data Controls"
              },
              "warn": false
            }
          ],
          "is": "card",
          "read": {
            "link": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir groq.com",
              "on": true,
              "tone": "plain"
            },
            "old": "Elles datent peut-être ; vérifiez la page du service.",
            "said": "Informations relevées le 02/11/2026. Elles datent peut-être ; vérifiez la page du service.",
            "text": "Informations relevées le 02/11/2026."
          },
          "untested": null,
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage : Settings → Data Controls"
          }
        },
        "sub": "Activé le 10/11/2026",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 23,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "on",
                  "text": "Prêt"
                }
              ],
              "name": "Groq",
              "tag": "Non testé",
              "tick": true
            },
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "DeepInfra",
              "tag": null,
              "tick": false
            },
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "OpenRouter",
              "tag": null,
              "tick": false
            },
            {
              "line": "Une adresse HTTPS, un modèle et une clé à saisir",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Adresse requise"
                }
              ],
              "name": "Adresse personnalisée",
              "tag": null,
              "tick": false
            },
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "ElevenLabs",
              "tag": null,
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 19,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudLong-1": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "Groq",
              "tag": "Non testé",
              "tick": true
            },
            {
              "line": "Gratuit : Aucun",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "OpenAI",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Aucun",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "Mistral",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Aucun ; premier achat d’au moins 15 $",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "Together AI",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Le premier mois",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "Lemonfox",
              "tag": "Non testé",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 41,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudLong-2": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 1,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Une adresse HTTPS, un modèle et une clé à saisir",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Adresse requise"
                }
              ],
              "name": "Adresse personnalisée",
              "tag": "Non testé",
              "tick": true
            },
            {
              "line": "Gratuit : Un petit quota pour les nouveaux comptes",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "OpenRouter",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Aucun",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "DeepInfra",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Environ 30 minutes par mois",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "ElevenLabs",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Sans carte bancaire",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement avec le quota gratuit"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "Gemini",
              "tag": "Non testé",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 42,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudLong-3": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 2,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Gratuit : 50 $ de crédit sans expiration, sans carte",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "AssemblyAI",
              "tag": "Non testé",
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 43,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudLongOff-1": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "Groq",
              "tag": "Non testé",
              "tick": true
            },
            {
              "line": "Gratuit : Aucun",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "OpenAI",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Aucun",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "Mistral",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Aucun ; premier achat d’au moins 15 $",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "Together AI",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Le premier mois",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "Lemonfox",
              "tag": "Non testé",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 44,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudLongOff-2": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 1,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Une adresse HTTPS, un modèle et une clé à saisir",
              "marks": [
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "Adresse personnalisée",
              "tag": "Non testé",
              "tick": true
            },
            {
              "line": "Gratuit : Un petit quota pour les nouveaux comptes",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "OpenRouter",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Aucun",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "DeepInfra",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Environ 30 minutes par mois",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "ElevenLabs",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : Sans carte bancaire",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement avec le quota gratuit"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "Gemini",
              "tag": "Non testé",
              "tick": false
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 45,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudLongOff-3": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 2,
          "count": 3
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Gratuit : 50 $ de crédit sans expiration, sans carte",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "AssemblyAI",
              "tag": "Non testé",
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 46,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudOff": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": "Services cloud désactivés : aucun n’est proposé, rien n’est envoyé.",
          "rows": [],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": false,
            "value": "Désactivé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 0,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 21,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "cloudSealed": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "lines",
            "kind": "service",
            "label": "Clé et modèle",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "on",
            "kind": "service",
            "label": "Activer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "cloud",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "cloud",
          "off": null,
          "rows": [
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "on",
                  "text": "Prêt"
                }
              ],
              "name": "Groq",
              "tag": "Non testé",
              "tick": false
            },
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "DeepInfra",
              "tag": null,
              "tick": false
            },
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "OpenRouter",
              "tag": null,
              "tick": false
            },
            {
              "line": "Une adresse HTTPS, un modèle et une clé à saisir",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Adresse requise"
                }
              ],
              "name": "Adresse personnalisée",
              "tag": null,
              "tick": false
            },
            {
              "line": "Gratuit : 8 heures par jour, 20 requêtes par minute",
              "marks": [
                {
                  "id": "warn",
                  "text": "Entraînement activé"
                },
                {
                  "id": "off",
                  "text": "Désactivé"
                }
              ],
              "name": "ElevenLabs",
              "tag": null,
              "tick": true
            }
          ],
          "switch": {
            "kind": "cloud",
            "label": "Utiliser les services cloud",
            "on": true,
            "value": "Activé"
          }
        },
        "sub": "Votre propre clé · rien ne part avant l’arrêt",
        "ticked": 1,
        "title": "Services cloud",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 20,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compare": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "6 modèles prêts · Automatique (processeur)",
          "picks": [],
          "recording": null,
          "send": null,
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 8,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, Groq",
              "service": "Groq",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 25,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareCloudUnticked": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, Groq",
              "service": "Groq",
              "tick": false
            },
            {
              "name": "Whisper Large v3",
              "said": "Whisper Large v3, Groq",
              "service": "Groq",
              "tick": false
            }
          ],
          "recording": null,
          "send": null,
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 26,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareEdge": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "mistralai/voxtral-small-24b-2507",
              "said": "mistralai/voxtral-small-24b-2507, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq, Lemonfox, l’adresse que vous saisissez, OpenRouter, ElevenLabs, Gemini et AssemblyAI à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 73,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareFew": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq, Lemonfox, l’adresse que vous saisissez, OpenRouter, ElevenLabs, Gemini et AssemblyAI à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 74,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, Groq",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "Whisper Large v3",
              "said": "Whisper Large v3, Groq",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "whisper-large-v3-turbo",
              "said": "whisper-large-v3-turbo, Adresse personnalisée",
              "service": "Adresse personnalisée",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq et l’adresse que vous saisissez à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 69,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareShort": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "mistralai/voxtral-mini-transcribe-3-0",
              "said": "mistralai/voxtral-mini-transcribe-3-0, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe, Gemini",
              "service": "Gemini",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq, Lemonfox, l’adresse que vous saisissez, OpenRouter, ElevenLabs, Gemini et AssemblyAI à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 72,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareTyped": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
              "said": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm, Adresse personnalisée",
              "service": "Adresse personnalisée",
              "tick": true
            },
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe, Gemini",
              "service": "Gemini",
              "tick": true
            },
            {
              "name": "wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww",
              "said": "wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq, Lemonfox, l’adresse que vous saisissez, OpenRouter, ElevenLabs, Gemini et AssemblyAI à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 71,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "compareWide": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "",
            "kind": "compare",
            "label": "Commencer la lecture",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "intro": "Lisez ce texte à voix haute une fois. Chaque modèle prêt l’écrit ensuite, l’un après l’autre.",
          "is": "ready",
          "line": "2 modèles prêts · Automatique (processeur)",
          "picks": [
            {
              "name": "Voxtral Mini Transcribe 3",
              "said": "Voxtral Mini Transcribe 3, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Whisper Large v3 Turbo",
              "said": "Whisper Large v3 Turbo, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            },
            {
              "name": "Gemini 3.5 Transcribe",
              "said": "Gemini 3.5 Transcribe, Gemini",
              "service": "Gemini",
              "tick": true
            },
            {
              "name": "Universal-3.6 Pro",
              "said": "Universal-3.6 Pro, AssemblyAI",
              "service": "AssemblyAI",
              "tick": true
            },
            {
              "name": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm",
              "said": "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm, OpenRouter",
              "service": "OpenRouter",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq, Lemonfox, l’adresse que vous saisissez, OpenRouter, ElevenLabs, Gemini et AssemblyAI à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 70,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "comparing": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "button",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "running",
          "rows": [
            {
              "mark": {
                "id": "done",
                "text": "Terminé · 2,1 s"
              },
              "name": "Whisper Small",
              "why": null
            },
            {
              "mark": {
                "id": "run-failed",
                "text": "Échec"
              },
              "name": "Whisper Large v3 Turbo",
              "why": null
            },
            {
              "mark": {
                "id": "writing",
                "text": "Écriture…"
              },
              "name": "Canary 1b v2",
              "why": null
            },
            {
              "mark": {
                "id": "waiting",
                "text": "En attente"
              },
              "name": "Qwen3-ASR 0.6B",
              "why": null
            },
            {
              "mark": {
                "id": "waiting",
                "text": "En attente"
              },
              "name": "Parakeet TDT 0.6b v3",
              "why": null
            }
          ]
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": {
          "id": "step",
          "name": "Canary 1b v2",
          "text": "Modèle 3 sur 6"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 9,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "comparingCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "button",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "running",
          "rows": [
            {
              "mark": {
                "id": "run-failed",
                "text": "Échec"
              },
              "name": "Whisper Large v3 Turbo",
              "why": "Le service a refusé la clé. Vérifiez-la sur la page de la clé."
            },
            {
              "mark": {
                "id": "writing",
                "text": "Écriture…"
              },
              "name": "Whisper small",
              "why": null
            }
          ]
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
        "top": {
          "id": "step",
          "name": "Whisper small",
          "text": "Modèle 2 sur 2"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 27,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "device": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "check",
            "kind": "button",
            "label": "Tester",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "device",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "device",
          "rows": [
            {
              "check": null,
              "kind": "auto",
              "line": "Le processeur tant qu’aucune carte n’a réussi son test",
              "mark": {
                "id": "used",
                "text": "Utilisé"
              },
              "name": "Automatique",
              "tick": false,
              "used": true
            },
            {
              "check": null,
              "kind": "cpu",
              "line": "8 cœurs · 16 Go de mémoire, 9,4 Go libres",
              "mark": null,
              "name": "Processeur",
              "tick": false,
              "used": false
            },
            {
              "check": null,
              "kind": "card",
              "line": "Carte graphique · 8 Go, 7,1 Go libres",
              "mark": {
                "id": "passed",
                "text": "Réussi · 9× plus rapide"
              },
              "name": "NVIDIA GeForce RTX 4060 Laptop GPU",
              "tick": false,
              "used": false
            },
            {
              "check": null,
              "kind": "built-in",
              "line": "Carte graphique intégrée · 4 Go partagés, 2,6 Go libres",
              "mark": {
                "id": "failed",
                "text": "Test échoué"
              },
              "name": "AMD Radeon 780M Graphics",
              "tick": false,
              "used": false
            },
            {
              "check": {
                "line": "Test : préchauffage de la carte",
                "share": 0.4
              },
              "kind": "card",
              "line": "Carte graphique · 4 Go, 3,8 Go libres",
              "mark": null,
              "name": "Intel Arc A370M",
              "tick": true,
              "used": false
            }
          ]
        },
        "sub": "Les cartes sont testées avant usage",
        "ticked": 1,
        "title": "Unité de calcul",
        "top": {
          "id": "failed",
          "name": null,
          "text": "AMD Radeon 780M Graphics : test échoué. Son texte différait de 3 mots ; 1 mot permis."
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 7,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "folders": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "here",
            "kind": "button",
            "label": "Utiliser ce dossier",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "folders",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "folders",
          "rows": [
            "C:\\",
            "D:\\"
          ]
        },
        "sub": "Lecteurs",
        "ticked": 0,
        "title": "Dossier des modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 18,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "freeResults": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "rerun",
            "kind": "button",
            "label": "Comparer à nouveau",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "head": [
            "Modèle",
            "Mots erronés",
            "Attente",
            "Mémoire"
          ],
          "is": "results",
          "rows": [
            {
              "memory": "0,5 Go",
              "name": "Whisper Small",
              "text": "The evening train leaves at seven.",
              "tick": false,
              "wait": "2,1 s",
              "wrong": null
            },
            {
              "memory": "0,5 Go",
              "name": "Whisper Large v3 Turbo",
              "text": "The evening train leaves at seven.",
              "tick": true,
              "wait": "4,0 s",
              "wrong": null
            },
            {
              "memory": "1,2 Go",
              "name": "Canary 1b v2",
              "text": "The evening train leaves at seven.",
              "tick": false,
              "wait": "1,8 s",
              "wrong": null
            }
          ]
        },
        "sub": "12/11/2026 · Automatique (processeur)",
        "ticked": 1,
        "title": "Comparer les modèles",
        "top": {
          "id": "kept",
          "name": null,
          "text": "Seuls ces chiffres sont gardés, ni l’enregistrement ni les textes."
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 11,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "history": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "Tout supprimer",
          "on": true,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "button",
            "label": "Copier",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "pin",
            "kind": "button",
            "label": "Épingler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "insert",
            "kind": "button",
            "label": "Insérer (2)",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "empty": null,
          "is": "history",
          "rows": [
            {
              "arabic": false,
              "id": 12,
              "meta": "Aujourd’hui 14:32 · Anglais · Parakeet TDT 0.6b v3",
              "name": "Thanks for the update. I will send the signed form on Thursday., Aujourd’hui 14:32 · Anglais · Parakeet TDT 0.6b v3, Coché 2",
              "pinned": false,
              "rtl": false,
              "text": "Thanks for the update. I will send the signed form on Thursday.",
              "tick": 2
            },
            {
              "arabic": false,
              "id": 11,
              "meta": "Aujourd’hui 11:32 · Français · Parakeet TDT 0.6b v3",
              "name": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures., Aujourd’hui 11:32 · Français · Parakeet TDT 0.6b v3, Coché 1, Épinglé",
              "pinned": true,
              "rtl": false,
              "text": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
              "tick": 1
            },
            {
              "arabic": true,
              "id": 10,
              "meta": "Aujourd’hui 09:32 · Arabe · Cohere Transcribe Arabic",
              "name": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع., Aujourd’hui 09:32 · Arabe · Cohere Transcribe Arabic",
              "pinned": false,
              "rtl": true,
              "text": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
              "tick": null
            }
          ],
          "unpin": false
        },
        "sub": "12 textes · les plus récents d’abord",
        "ticked": 2,
        "title": "Historique",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 12,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "historyAsk": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "Tout supprimer",
          "on": false,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Oui",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Non",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Supprimer 11 textes ? Les textes épinglés restent.",
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 3
        },
        "sheet": {
          "empty": null,
          "is": "history",
          "rows": [
            {
              "arabic": false,
              "id": 12,
              "meta": "Aujourd’hui 14:32 · Anglais · Parakeet TDT 0.6b v3",
              "name": "Thanks for the update. I will send the signed form on Thursday., Aujourd’hui 14:32 · Anglais · Parakeet TDT 0.6b v3, Coché 2",
              "pinned": false,
              "rtl": false,
              "text": "Thanks for the update. I will send the signed form on Thursday.",
              "tick": 2
            },
            {
              "arabic": false,
              "id": 11,
              "meta": "Aujourd’hui 11:32 · Français · Parakeet TDT 0.6b v3",
              "name": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures., Aujourd’hui 11:32 · Français · Parakeet TDT 0.6b v3, Coché 1, Épinglé",
              "pinned": true,
              "rtl": false,
              "text": "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
              "tick": 1
            },
            {
              "arabic": true,
              "id": 10,
              "meta": "Aujourd’hui 09:32 · Arabe · Cohere Transcribe Arabic",
              "name": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع., Aujourd’hui 09:32 · Arabe · Cohere Transcribe Arabic",
              "pinned": false,
              "rtl": true,
              "text": "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
              "tick": null
            }
          ],
          "unpin": false
        },
        "sub": "12 textes · les plus récents d’abord",
        "ticked": 0,
        "title": "Historique",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 13,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "historyEmpty": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": {
          "arg": 0,
          "id": "delete-all",
          "kind": "button",
          "label": "Tout supprimer",
          "on": false,
          "tone": "plain"
        },
        "foot": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "button",
            "label": "Copier",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "pin",
            "kind": "button",
            "label": "Épingler",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "insert",
            "kind": "button",
            "label": "Insérer",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "history",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "empty": "Aucun texte pour l’instant. Dictez quelque chose et il s’affichera ici.",
          "is": "history",
          "rows": [],
          "unpin": false
        },
        "sub": "0 texte · les plus récents d’abord",
        "ticked": 0,
        "title": "Historique",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 14,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "home": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": null,
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "again",
            "kind": "button",
            "label": "Réessayer",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Retranscrit le dernier enregistrement avec les choix faits ici.",
        "note": null,
        "page": "home",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "is": "home",
          "rows": [
            {
              "label": "Modèles",
              "tab": "models",
              "value": "Parakeet TDT 0.6b v3"
            },
            {
              "label": "Services cloud",
              "tab": "cloud",
              "value": "Désactivé"
            },
            {
              "label": "Unité de calcul",
              "tab": "device",
              "value": "Automatique (processeur)"
            },
            {
              "label": "Comparer les modèles",
              "tab": "compare",
              "value": "Dernière comparaison : 12/11/2026"
            },
            {
              "label": "Historique",
              "tab": "history",
              "value": "12 textes"
            }
          ]
        },
        "sub": "Langue : français",
        "ticked": 0,
        "title": "Dictée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 6,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "models": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer le fichier",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Télécharger",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Changer",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Ouvrir",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Dossier des modèles : D:\\KeyXtend\\models · 222 Go libres"
          },
          "rows": [
            {
              "fetch": null,
              "line": "776 Mo",
              "marks": [
                {
                  "id": "used",
                  "text": "Utilisé"
                },
                {
                  "id": "default",
                  "text": "Par défaut"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 Mo",
              "marks": [
                {
                  "id": "ready",
                  "text": "Prêt"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "2,53 Go",
              "marks": [
                {
                  "id": "download",
                  "text": "Non téléchargé"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "Pause",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Annuler",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 Mo sur 1,22 Go",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Demande 3,2 Go de mémoire, 2,8 Go libres",
              "marks": [
                {
                  "id": "not-here",
                  "text": "Ne convient pas à ce PC"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "En français",
        "ticked": 1,
        "title": "Modèles",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Conseillé en français :"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 1,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "modelsCloud": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer le fichier",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Télécharger",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "Services cloud",
              "on": true,
              "tone": "plain"
            },
            "text": "Modèles des services que vous avez activés ; rien ne part avant l’arrêt."
          },
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "Dans le cloud, avec votre clé",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "ready",
                  "text": "Prêt"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "Aucun hébergeur ne s’engage à ne rien garder.",
              "marks": [
                {
                  "id": "cloud",
                  "text": "OpenRouter"
                },
                {
                  "id": "unavailable",
                  "text": "Indisponible"
                }
              ],
              "name": "voxtral-small",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Dans le cloud, avec votre clé",
              "marks": [
                {
                  "id": "cloud",
                  "text": "DeepInfra"
                },
                {
                  "id": "needs-key",
                  "text": "Clé requise"
                }
              ],
              "name": "Nemotron 3.5 Transcribe",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "cloud"
          },
          "slow": null,
          "untested": null
        },
        "sub": "En français",
        "ticked": 1,
        "title": "Modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 24,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "modelsLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer le fichier",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Télécharger",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "Services cloud",
              "on": true,
              "tone": "plain"
            },
            "text": "Modèles des services que vous avez activés ; rien ne part avant l’arrêt."
          },
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "Dans le cloud, avec votre clé",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "used",
                  "text": "Utilisé"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "Dans le cloud, avec votre clé",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Groq"
                },
                {
                  "id": "ready",
                  "text": "Prêt"
                }
              ],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Dans le cloud, avec votre clé",
              "marks": [
                {
                  "id": "cloud",
                  "text": "Adresse personnalisée"
                },
                {
                  "id": "needs-address",
                  "text": "Adresse requise"
                }
              ],
              "name": "whisper-large-v3-turbo",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Aucun hébergeur ne s’engage à ne rien garder.",
              "marks": [
                {
                  "id": "cloud",
                  "text": "OpenRouter"
                },
                {
                  "id": "unavailable",
                  "text": "Indisponible"
                }
              ],
              "name": "voxtral-small",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "cloud"
          },
          "slow": null,
          "untested": null
        },
        "sub": "En français",
        "ticked": 1,
        "title": "Modèles",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 40,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "modelsMore": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer le fichier",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Télécharger",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": {
          "id": "net",
          "text": "Le téléchargement n’a pas abouti. Réessayez plus tard."
        },
        "page": "models",
        "pages": {
          "at": 1,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Changer",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Ouvrir",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Dossier des modèles : D:\\KeyXtend\\models · 222 Go libres"
          },
          "rows": [
            {
              "fetch": null,
              "line": "1,6 Go",
              "marks": [
                {
                  "id": "other",
                  "text": "Autre fichier, non vérifié"
                }
              ],
              "name": "ggml-medium.bin",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 1,
                    "id": "resume",
                    "kind": "fetch",
                    "label": "Reprendre",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 1,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Annuler",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": true,
                "share": 0.2
              },
              "line": "En pause · 239 Mo sur 1,2 Go",
              "marks": [],
              "name": "Canary 1b v2",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Modèles",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Conseillé en français :"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 2,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "modelsOffer": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 1,
            "id": "",
            "kind": "confirm",
            "label": "Oui",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "",
            "kind": "confirm",
            "label": "Non",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Garder les modèles dans D:\\KeyXtend\\models ? 222 Go libres.",
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": null,
          "rows": [
            {
              "fetch": null,
              "line": "776 Mo",
              "marks": [
                {
                  "id": "used",
                  "text": "Utilisé"
                },
                {
                  "id": "default",
                  "text": "Par défaut"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 Mo",
              "marks": [
                {
                  "id": "ready",
                  "text": "Prêt"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": true
            },
            {
              "fetch": null,
              "line": "2,53 Go",
              "marks": [
                {
                  "id": "download",
                  "text": "Non téléchargé"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "Pause",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Annuler",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 Mo sur 1,22 Go",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Demande 3,2 Go de mémoire, 2,8 Go libres",
              "marks": [
                {
                  "id": "not-here",
                  "text": "Ne convient pas à ce PC"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Modèles",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Conseillé en français :"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 5,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "modelsSlow": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer le fichier",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Télécharger",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Changer",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Ouvrir",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Dossier des modèles : D:\\KeyXtend\\models · 222 Go libres"
          },
          "rows": [
            {
              "fetch": null,
              "line": "Demande 5 Go de mémoire, 2,8 Go libres",
              "marks": [
                {
                  "id": "default",
                  "text": "Par défaut"
                },
                {
                  "id": "not-here",
                  "text": "Ne convient pas à ce PC"
                }
              ],
              "name": "Cohere Transcribe Arabic",
              "tick": false
            },
            {
              "fetch": null,
              "line": "650 Mo",
              "marks": [
                {
                  "id": "ready",
                  "text": "Prêt"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "local"
          },
          "slow": {
            "button": {
              "arg": 0,
              "id": "cloud",
              "kind": "open",
              "label": "Services cloud",
              "on": true,
              "tone": "plain"
            },
            "text": "Les modèles locaux en français seront très lents sur ce PC."
          },
          "untested": "Cet ordre n’a pas été testé en français."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Modèles",
        "top": {
          "id": "best",
          "name": "Cohere Transcribe Arabic",
          "text": "Conseillé en français :"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 4,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "modelsUsed": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "delete",
            "kind": "button",
            "label": "Supprimer le fichier",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "download",
            "kind": "button",
            "label": "Télécharger",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": false,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "models",
        "pages": {
          "at": 0,
          "count": 2
        },
        "sheet": {
          "band": null,
          "is": "models",
          "room": {
            "buttons": [
              {
                "arg": 0,
                "id": "change",
                "kind": "button",
                "label": "Changer",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "open",
                "kind": "button",
                "label": "Ouvrir",
                "on": true,
                "tone": "plain"
              }
            ],
            "text": "Dossier des modèles : D:\\KeyXtend\\models · 222 Go libres"
          },
          "rows": [
            {
              "fetch": null,
              "line": "776 Mo",
              "marks": [
                {
                  "id": "used",
                  "text": "Utilisé"
                },
                {
                  "id": "default",
                  "text": "Par défaut"
                }
              ],
              "name": "Parakeet TDT 0.6b v3",
              "tick": true
            },
            {
              "fetch": null,
              "line": "650 Mo",
              "marks": [
                {
                  "id": "ready",
                  "text": "Prêt"
                }
              ],
              "name": "Whisper Large v3 Turbo",
              "tick": false
            },
            {
              "fetch": null,
              "line": "2,53 Go",
              "marks": [
                {
                  "id": "download",
                  "text": "Non téléchargé"
                }
              ],
              "name": "Cohere Transcribe",
              "tick": false
            },
            {
              "fetch": {
                "buttons": [
                  {
                    "arg": 3,
                    "id": "pause",
                    "kind": "fetch",
                    "label": "Pause",
                    "on": true,
                    "tone": "plain"
                  },
                  {
                    "arg": 3,
                    "id": "cancel",
                    "kind": "fetch",
                    "label": "Annuler",
                    "on": true,
                    "tone": "plain"
                  }
                ],
                "paused": false,
                "share": 0.4603448275862069
              },
              "line": "560 Mo sur 1,22 Go",
              "marks": [],
              "name": "Whisper Large v3",
              "tick": false
            },
            {
              "fetch": null,
              "line": "Demande 3,2 Go de mémoire, 2,8 Go libres",
              "marks": [
                {
                  "id": "not-here",
                  "text": "Ne convient pas à ce PC"
                }
              ],
              "name": "Qwen3-ASR 1.7B",
              "tick": false
            }
          ],
          "sides": {
            "chips": [
              {
                "arg": 0,
                "id": "local",
                "kind": "side",
                "label": "Sur ce PC",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "cloud",
                "kind": "side",
                "label": "Dans le cloud",
                "on": true,
                "tone": "plain"
              }
            ],
            "name": "Où le modèle s’exécute",
            "shown": "local"
          },
          "slow": null,
          "untested": null
        },
        "sub": "En français",
        "ticked": 1,
        "title": "Modèles",
        "top": {
          "id": "best",
          "name": "Parakeet TDT 0.6b v3",
          "text": "Conseillé en français :"
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 3,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "options": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Les changements s’appliquent tout de suite.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 0,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "sounds",
              "kind": "switch",
              "label": "Sons de début et de fin",
              "live": true,
              "on": true,
              "steps": null,
              "value": "Activé"
            },
            {
              "chips": null,
              "key": "silence_stop",
              "kind": "switch",
              "label": "Arrêt après un silence",
              "live": true,
              "on": false,
              "steps": null,
              "value": "Désactivé"
            },
            {
              "chips": null,
              "key": "silence_stop_s",
              "kind": "number",
              "label": "Silence avant l’arrêt",
              "live": false,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Moins : Silence avant l’arrêt",
                "plus": "Plus : Silence avant l’arrêt"
              },
              "value": "3 s"
            },
            {
              "chips": null,
              "key": "done_ms",
              "kind": "number",
              "label": "Durée d’affichage du texte",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Moins : Durée d’affichage du texte",
                "plus": "Plus : Durée d’affichage du texte"
              },
              "value": "9 s"
            },
            {
              "chips": null,
              "key": "place",
              "kind": "choice",
              "label": "Position de la barre",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": true,
                "minus": "Moins : Position de la barre",
                "plus": "Plus : Position de la barre"
              },
              "value": "Au-dessus du clavier"
            }
          ]
        },
        "sub": "Pour toutes les langues",
        "ticked": 0,
        "title": "Options",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 15,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "optionsInsert": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Les changements s’appliquent tout de suite.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 2,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "debounce_ms",
              "kind": "number",
              "label": "Ignorer un second clic pendant",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Moins : Ignorer un second clic pendant",
                "plus": "Plus : Ignorer un second clic pendant"
              },
              "value": "400 ms"
            },
            {
              "chips": [
                {
                  "label": "Le saisir",
                  "on": false
                },
                {
                  "label": "Le coller en une fois",
                  "on": true
                }
              ],
              "key": "insert",
              "kind": "pair",
              "label": "Insérer le texte",
              "live": true,
              "on": false,
              "steps": null,
              "value": "Le coller en une fois"
            },
            {
              "chips": null,
              "key": "insert",
              "kind": "hint",
              "label": "Passe par le presse-papiers, puis y remet votre contenu ; un champ de mot de passe est toujours saisi.",
              "live": true,
              "on": false,
              "steps": null,
              "value": ""
            },
            {
              "chips": null,
              "key": "type_suffix",
              "kind": "switch",
              "label": "Espace après le texte",
              "live": true,
              "on": true,
              "steps": null,
              "value": "Activé"
            },
            {
              "chips": null,
              "key": "type_gap_ms",
              "kind": "number",
              "label": "Pause entre les caractères",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Moins : Pause entre les caractères",
                "plus": "Plus : Pause entre les caractères"
              },
              "value": "5 ms"
            }
          ]
        },
        "sub": "Pour toutes les langues",
        "ticked": 0,
        "title": "Options",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 17,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "optionsMore": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "Les changements s’appliquent tout de suite.",
        "note": null,
        "page": "options",
        "pages": {
          "at": 1,
          "count": 5
        },
        "sheet": {
          "is": "options",
          "rows": [
            {
              "chips": null,
              "key": "threads",
              "kind": "number",
              "label": "Fils d’exécution du processeur",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": true,
                "minus": "Moins : Fils d’exécution du processeur",
                "plus": "Plus : Fils d’exécution du processeur"
              },
              "value": "Automatique"
            },
            {
              "chips": null,
              "key": "type_suffix",
              "kind": "switch",
              "label": "Espace après le texte",
              "live": true,
              "on": true,
              "steps": null,
              "value": "Activé"
            },
            {
              "chips": null,
              "key": "card_gain",
              "kind": "number",
              "label": "Gain minimal d’une carte",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Moins : Gain minimal d’une carte",
                "plus": "Plus : Gain minimal d’une carte"
              },
              "value": "1,5×"
            },
            {
              "chips": null,
              "key": "history_max",
              "kind": "number",
              "label": "Textes conservés",
              "live": true,
              "on": false,
              "steps": {
                "high": true,
                "low": false,
                "minus": "Moins : Textes conservés",
                "plus": "Plus : Textes conservés"
              },
              "value": "500 textes"
            },
            {
              "chips": null,
              "key": "card_diff",
              "kind": "number",
              "label": "Écart permis pour une carte",
              "live": true,
              "on": false,
              "steps": {
                "high": false,
                "low": false,
                "minus": "Moins : Écart permis pour une carte",
                "plus": "Plus : Écart permis pour une carte"
              },
              "value": "1 mot"
            }
          ]
        },
        "sub": "Pour toutes les langues",
        "ticked": 0,
        "title": "Options",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 16,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "results": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "rerun",
            "kind": "button",
            "label": "Comparer à nouveau",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "use",
            "kind": "button",
            "label": "Utiliser pour cette langue",
            "on": true,
            "tone": "primary"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": null,
        "note": null,
        "page": "compare",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "head": [
            "Modèle",
            "Mots erronés",
            "Attente",
            "Mémoire"
          ],
          "is": "results",
          "rows": [
            {
              "memory": "0,5 Go",
              "name": "Whisper Small",
              "text": null,
              "tick": false,
              "wait": "2,1 s",
              "wrong": "0 sur 28"
            },
            {
              "memory": "0,5 Go",
              "name": "Whisper Large v3 Turbo",
              "text": null,
              "tick": true,
              "wait": "4,0 s",
              "wrong": "0 sur 28"
            },
            {
              "memory": "1,2 Go",
              "name": "Canary 1b v2",
              "text": null,
              "tick": false,
              "wait": "1,8 s",
              "wrong": "1 sur 28"
            }
          ]
        },
        "sub": "12/11/2026 · Automatique (processeur)",
        "ticked": 1,
        "title": "Comparer les modèles",
        "top": {
          "id": "kept",
          "name": null,
          "text": "Seuls ces chiffres sont gardés, ni l’enregistrement ni les textes."
        },
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 10,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "service": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "ok",
            "text": "La clé fonctionne",
            "warns": false,
            "works": true
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 28,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceAllowance": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "allowance",
            "text": "Le quota gratuit est épuisé. Il se renouvelle le 14/11/2026 à 23:59.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 34,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceAssembly": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "no-test",
            "text": "Pas de test possible ici : la première dictée testera la clé.",
            "warns": false,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur www.assemblyai.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "AssemblyAI",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 30,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceCustom": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller l’adresse",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer l’adresse",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Adresse",
            "dots": 0,
            "fault": "L’adresse doit utiliser HTTPS, par exemple https://example.com",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 0,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 31,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceLong": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller l’adresse",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer l’adresse",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Adresse",
            "dots": 0,
            "fault": "L’adresse contient un caractère non autorisé, comme un espace.",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "Le service demande une étape sur son site, comme une vérification, un moyen de paiement ou un plafond de dépenses.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 33,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "servicePasted-address": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller l’adresse",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer l’adresse",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Adresse",
            "dots": 0,
            "fault": "Le presse-papiers ne contient aucun texte.",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "Le service demande une étape sur son site, comme une vérification, un moyen de paiement ou un plafond de dépenses.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 38,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "servicePasted-key": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller l’adresse",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer l’adresse",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Adresse",
            "dots": 0,
            "fault": "L’adresse contient un caractère non autorisé, comme un espace.",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": "Le presse-papiers ne contient aucun texte.",
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 37,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "servicePasted-model": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": false,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": {
            "at": 7,
            "buttons": [
              {
                "arg": 0,
                "id": "address",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller l’adresse",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "address",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer l’adresse",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Adresse",
            "dots": 0,
            "fault": "L’adresse contient un caractère non autorisé, comme un espace.",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example",
            "typed": 0,
            "typing": true
          },
          "check": {
            "id": "account",
            "text": "Le service demande une étape sur son site, comme une vérification, un moyen de paiement ou un plafond de dépenses.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": false
          },
          "link": null,
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Le presse-papiers ne contient aucun texte.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model",
            "typed": 0,
            "typing": false
          },
          "order": [
            "address",
            "model",
            "key"
          ]
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 39,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceRate": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "rate",
            "text": "Trop de requêtes. Patientez un instant et réessayez.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 35,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceRouter": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "no-test",
            "text": "Pas de test possible ici : la première dictée testera la clé.",
            "warns": false,
            "works": false
          },
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "Seulement des hébergeurs qui ne gardent rien",
            "on": true,
            "value": "Activé"
          },
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Aucun hébergeur ne s’engage à ne rien garder.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "mistralai/voxtral-small",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "OpenRouter",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 29,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceStrange": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "strange",
            "text": "La réponse ne venait pas du service, peut-être une page de connexion. Vérifiez le réseau.",
            "warns": true,
            "works": false
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 56,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": false,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 0,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 36,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    },
    "serviceTyped": {
      "bar": {
        "state": "hidden"
      },
      "cloud": null,
      "panel": {
        "back": {
          "arg": 0,
          "id": "back",
          "kind": "button",
          "label": "Retour",
          "on": true,
          "tone": "plain"
        },
        "close": {
          "arg": 0,
          "id": "close",
          "kind": "button",
          "label": "Fermer",
          "on": true,
          "tone": "plain"
        },
        "extra": null,
        "foot": [
          {
            "arg": 0,
            "id": "facts",
            "kind": "service",
            "label": "Informations",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "remove",
            "kind": "service",
            "label": "Retirer la clé",
            "on": true,
            "tone": "danger"
          },
          {
            "arg": 0,
            "id": "off",
            "kind": "service",
            "label": "Désactiver",
            "on": true,
            "tone": "plain"
          }
        ],
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "line": "La clé est chiffrée sur ce PC et jamais affichée.",
        "note": null,
        "page": "service",
        "pages": {
          "at": 0,
          "count": 1
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "ok",
            "text": "La clé fonctionne",
            "warns": false,
            "works": true
          },
          "is": "service",
          "keep": null,
          "key": {
            "at": 12,
            "buttons": [
              {
                "arg": 0,
                "id": "key",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "key",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer la clé",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
                "on": true,
                "tone": "plain"
              }
            ],
            "caption": "Clé",
            "dots": 56,
            "fault": null,
            "hint": "Tapez ou collez votre clé",
            "id": "key",
            "mask": "••••••••",
            "text": "",
            "typed": 20,
            "typing": true
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": true,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": {
            "at": 0,
            "buttons": [
              {
                "arg": 0,
                "id": "model",
                "kind": "paste",
                "label": "Coller",
                "name": "Coller le modèle",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "model",
                "kind": "save",
                "label": "Enregistrer",
                "name": "Enregistrer le modèle",
                "on": false,
                "tone": "plain"
              }
            ],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "",
            "typed": 0,
            "typing": false
          },
          "order": [
            "key",
            "model"
          ]
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "Groq",
        "top": null,
        "window": "Réglages de la dictée de KeyXtend"
      },
      "place": "above",
      "round": 32,
      "sounds": true,
      "words": {
        "copied": "Copié",
        "gear": "Réglages de la dictée",
        "timer": "%1:%2",
        "title": "Dictée",
        "window": "Barre de dictée de KeyXtend"
      }
    }
  }
};
var fewest = 1;

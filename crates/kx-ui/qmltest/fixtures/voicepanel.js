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
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
        "ticked": 0,
        "title": "عنوان مخصّص",
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
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
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
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
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
            "text": "قُرئت هذه المعلومات في 17/11/2026."
          },
          "untested": "غير مُختبَرة: لم تُفحص بمفتاح حقيقي بعد.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: Settings → Data Controls"
          }
        },
        "sub": "اقرأ هذا قبل التفعيل",
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
            "text": "قُرئت هذه المعلومات في 02/11/2026."
          },
          "untested": null,
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. قبل التأكيد، عطِّل هذا الخيار في حسابك: Settings → Data Controls"
          }
        },
        "sub": "فُعِّلت في 10/11/2026",
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
              "line": "عنوان HTTPS ونموذج ومفتاح تكتبها بنفسك",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "يلزم عنوان"
                }
              ],
              "name": "عنوان مخصّص",
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
            }
          ],
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
              "service": "Groq",
              "tick": true
            },
            {
              "name": "Whisper Large v3",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "whisper-large-v3-turbo",
              "service": "عنوان مخصّص",
              "tick": true
            }
          ],
          "recording": null,
          "send": "سترسل المقارنة صوتك إلى Groq، عنوان مخصّص عند الإيقاف.",
          "text": "يغادر قطار المساء في السابعة والربع. يُرجى إحضار معطف دافئ وشطيرتين والمظلة الزرقاء، لأن الطقس على الساحل يتغير بسرعة."
        },
        "sub": "للغة العربية",
        "ticked": 0,
        "title": "مقارنة النماذج",
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
            "text": "المفتاح يعمل"
          },
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "لصق",
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
            "text": ""
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
            "buttons": [],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": null,
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": ""
          }
        },
        "sub": "جاهزة",
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
            "buttons": [],
            "caption": "العنوان",
            "dots": 0,
            "fault": "يجب أن يستخدم العنوان HTTPS، مثل https://example.com",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example"
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "لصق",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
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
            "text": ""
          },
          "link": null,
          "model": {
            "buttons": [],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model"
          }
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
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
          "count": 2
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "failed",
            "text": "لم يصل رد بشأن المفتاح. تحقّق من الشبكة واختبره مرة أخرى."
          },
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "لصق",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "اختبار",
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
            "text": ""
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "فتح",
              "on": false,
              "tone": "plain"
            },
            "text": "إنشاء مفتاح في console.groq.com"
          },
          "model": null
        },
        "sub": "يلزم مفتاح",
        "ticked": 0,
        "title": "عنوان مخصّص",
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
    "serviceLongModel": {
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
          "at": 1,
          "count": 2
        },
        "sheet": {
          "address": {
            "buttons": [],
            "caption": "العنوان",
            "dots": 0,
            "fault": "يحتوي العنوان على حرف غير مسموح به، مثل المسافة.",
            "hint": "عنوان خدمتك، مثل https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example"
          },
          "check": null,
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "الاقتصار على مضيفين لا يحتفظون بشيء",
            "on": true,
            "value": "مفعّل"
          },
          "key": null,
          "link": null,
          "model": {
            "buttons": [],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "no such/model"
          }
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
            "text": "لا يوجد اختبار مسبق لهذه الخدمة، فسيُختبر المفتاح عند أول إملاء."
          },
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "الاقتصار على مضيفين لا يحتفظون بشيء",
            "on": true,
            "value": "مفعّل"
          },
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "لصق",
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
            "text": ""
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
            "buttons": [],
            "caption": "نموذج آخر",
            "dots": 0,
            "fault": "لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.",
            "hint": "معرّف النموذج من صفحة الخدمة",
            "id": "model",
            "mask": null,
            "text": "mistralai/voxtral-small"
          }
        },
        "sub": "جاهزة",
        "ticked": 0,
        "title": "OpenRouter",
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
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
        "ticked": 0,
        "title": "Custom address",
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
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
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
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
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
            "text": "Facts read on 17/11/2026."
          },
          "untested": "Untested: not yet checked with a real key.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: Settings → Data Controls"
          }
        },
        "sub": "Read this before you switch it on",
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
            "text": "Facts read on 02/11/2026."
          },
          "untested": null,
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Trains its models on your voice by default. Before you confirm, turn off this switch in your account: Settings → Data Controls"
          }
        },
        "sub": "Switched on 10/11/2026",
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
              "line": "An HTTPS address, a model and a key you type",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Needs an address"
                }
              ],
              "name": "Custom address",
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
            }
          ],
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
              "service": "Groq",
              "tick": true
            },
            {
              "name": "Whisper Large v3",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "whisper-large-v3-turbo",
              "service": "Custom address",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Compare sends your voice to Groq, Custom address at Stop.",
          "text": "The evening train leaves at a quarter past seven. Please bring a warm coat, two sandwiches and the blue umbrella, because the weather on the coast changes quickly."
        },
        "sub": "For English",
        "ticked": 0,
        "title": "Compare models",
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
            "text": "Key works"
          },
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Paste",
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
            "text": ""
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
            "buttons": [],
            "caption": "Other model",
            "dots": 0,
            "fault": null,
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": ""
          }
        },
        "sub": "Ready",
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
            "buttons": [],
            "caption": "Address",
            "dots": 0,
            "fault": "The address must use HTTPS, such as https://example.com",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example"
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Paste",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
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
            "text": ""
          },
          "link": null,
          "model": {
            "buttons": [],
            "caption": "Other model",
            "dots": 0,
            "fault": "This model id is too long or holds a character that is not allowed.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model"
          }
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
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
          "count": 2
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "failed",
            "text": "No answer about the key. Check the network and test again."
          },
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Paste",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Test",
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
            "text": ""
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Open",
              "on": false,
              "tone": "plain"
            },
            "text": "Make a key at console.groq.com"
          },
          "model": null
        },
        "sub": "Needs a key",
        "ticked": 0,
        "title": "Custom address",
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
    "serviceLongModel": {
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
          "at": 1,
          "count": 2
        },
        "sheet": {
          "address": {
            "buttons": [],
            "caption": "Address",
            "dots": 0,
            "fault": "The address holds a character that is not allowed, such as a space.",
            "hint": "Your service’s address, such as https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example"
          },
          "check": null,
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "Only hosts that keep nothing",
            "on": true,
            "value": "On"
          },
          "key": null,
          "link": null,
          "model": {
            "buttons": [],
            "caption": "Other model",
            "dots": 0,
            "fault": "This model id is too long or holds a character that is not allowed.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "no such/model"
          }
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
            "text": "No test call here: the first dictation tests the key."
          },
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "Only hosts that keep nothing",
            "on": true,
            "value": "On"
          },
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Paste",
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
            "text": ""
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
            "buttons": [],
            "caption": "Other model",
            "dots": 0,
            "fault": "No host of this model promises to keep nothing.",
            "hint": "A model id from the service’s page",
            "id": "model",
            "mask": null,
            "text": "mistralai/voxtral-small"
          }
        },
        "sub": "Ready",
        "ticked": 0,
        "title": "OpenRouter",
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
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
        "ticked": 0,
        "title": "Adresse personnalisée",
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
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
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
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
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
            "text": "Informations relevées le 17/11/2026."
          },
          "untested": "Non testé : pas encore vérifié avec une vraie clé.",
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : Settings → Data Controls"
          }
        },
        "sub": "À lire avant d’activer",
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
            "text": "Informations relevées le 02/11/2026."
          },
          "untested": null,
          "warn": {
            "parts": [
              {
                "own": false,
                "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : "
              },
              {
                "own": true,
                "text": "Settings → Data Controls"
              }
            ],
            "text": "Entraîne ses modèles sur votre voix par défaut. Avant de confirmer, désactivez ce réglage dans votre compte : Settings → Data Controls"
          }
        },
        "sub": "Activé le 10/11/2026",
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
              "line": "Une adresse HTTPS, un modèle et une clé à saisir",
              "marks": [
                {
                  "id": "needs-address",
                  "text": "Adresse requise"
                }
              ],
              "name": "Adresse personnalisée",
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
            }
          ],
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
              "service": "Groq",
              "tick": true
            },
            {
              "name": "Whisper Large v3",
              "service": "Groq",
              "tick": true
            },
            {
              "name": "whisper-large-v3-turbo",
              "service": "Adresse personnalisée",
              "tick": true
            }
          ],
          "recording": null,
          "send": "Comparer enverra votre voix à Groq, Adresse personnalisée à l’arrêt.",
          "text": "Le train du soir part à sept heures et quart. Prenez un manteau chaud, deux sandwichs et le parapluie bleu, car le temps change vite sur la côte."
        },
        "sub": "En français",
        "ticked": 0,
        "title": "Comparer les modèles",
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
            "text": "La clé fonctionne"
          },
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Coller",
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
            "text": ""
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
            "buttons": [],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": null,
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": ""
          }
        },
        "sub": "Prêt",
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
            "buttons": [],
            "caption": "Adresse",
            "dots": 0,
            "fault": "L’adresse doit utiliser HTTPS, par exemple https://example.com",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example"
          },
          "check": null,
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Coller",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
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
            "text": ""
          },
          "link": null,
          "model": {
            "buttons": [],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model"
          }
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
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
          "count": 2
        },
        "sheet": {
          "address": null,
          "check": {
            "id": "failed",
            "text": "Aucune réponse sur la clé. Vérifiez le réseau et testez de nouveau."
          },
          "is": "service",
          "keep": null,
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Coller",
                "on": true,
                "tone": "plain"
              },
              {
                "arg": 0,
                "id": "test",
                "kind": "service",
                "label": "Tester",
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
            "text": ""
          },
          "link": {
            "button": {
              "arg": 0,
              "id": "link",
              "kind": "service",
              "label": "Ouvrir",
              "on": false,
              "tone": "plain"
            },
            "text": "Créer une clé sur console.groq.com"
          },
          "model": null
        },
        "sub": "Clé requise",
        "ticked": 0,
        "title": "Adresse personnalisée",
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
    "serviceLongModel": {
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
          "at": 1,
          "count": 2
        },
        "sheet": {
          "address": {
            "buttons": [],
            "caption": "Adresse",
            "dots": 0,
            "fault": "L’adresse contient un caractère non autorisé, comme un espace.",
            "hint": "L’adresse de votre service, par exemple https://example.com/v1/audio/transcriptions",
            "id": "address",
            "mask": null,
            "text": "http://lan.example"
          },
          "check": null,
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "Seulement des hébergeurs qui ne gardent rien",
            "on": true,
            "value": "Activé"
          },
          "key": null,
          "link": null,
          "model": {
            "buttons": [],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "no such/model"
          }
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
            "text": "Pas de test possible ici : la première dictée testera la clé."
          },
          "is": "service",
          "keep": {
            "kind": "keep",
            "label": "Seulement des hébergeurs qui ne gardent rien",
            "on": true,
            "value": "Activé"
          },
          "key": {
            "buttons": [
              {
                "arg": 0,
                "id": "paste",
                "kind": "service",
                "label": "Coller",
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
            "text": ""
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
            "buttons": [],
            "caption": "Autre modèle",
            "dots": 0,
            "fault": "Aucun hébergeur ne s’engage à ne rien garder.",
            "hint": "L’identifiant du modèle, sur la page du service",
            "id": "model",
            "mask": null,
            "text": "mistralai/voxtral-small"
          }
        },
        "sub": "Prêt",
        "ticked": 0,
        "title": "OpenRouter",
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
    }
  }
};

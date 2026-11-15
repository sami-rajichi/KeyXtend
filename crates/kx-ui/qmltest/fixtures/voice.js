.pragma library
// Written by kx-ui's texts test; do not edit.
var acts = {
  "bar": "bar",
  "button": "button",
  "compare": "compare",
  "confirm": "confirm",
  "fetch": "fetch",
  "flip": "flip",
  "gear": "gear",
  "minus": "minus",
  "open": "open",
  "over": "over",
  "page": "page",
  "plus": "plus",
  "row": "row",
  "step": "step",
  "wheel": "wheel"
};
var views = {
  "ar": {
    "cancelled": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "anyway",
            "kind": "bar",
            "label": "تحويل التسجيل إلى نص",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "cancelled",
          "text": "أُلغي الإملاء، ولم يُكتب شيء."
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "checking": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "checking",
          "text": "ما زال الملف المختار قيد الفحص. حاول مرة أخرى بعد قليل."
        },
        "mic": true,
        "state": "note"
      },
      "panel": null,
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
    "copyFailed": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "نسخ",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "تراجع عن الإدراج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "copy-failed",
          "text": "تعذّر نسخ النص. حاول مرة أخرى."
        },
        "state": "text",
        "text": {
          "arabic": true,
          "masked": false,
          "rtl": true,
          "text": "شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل."
        }
      },
      "panel": null,
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
    "fault": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "fault",
          "text": "توقف الإملاء بسبب خطأ. حاول مرة أخرى."
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "heardNothing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "listen",
            "kind": "bar",
            "label": "إعادة الاستماع",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "nothing",
          "text": "لم يُسمع شيء. هل يعمل الميكروفون؟"
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "hidden": {
      "bar": {
        "state": "hidden"
      },
      "panel": null,
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
    "listening": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "إيقاف",
            "on": true,
            "tone": "primary"
          }
        ],
        "elapsedMs": 1200,
        "hint": null,
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "state": "listening",
        "waveMs": 300,
        "word": "جارٍ الاستماع…"
      },
      "panel": null,
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
    "long": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "نسخ",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "تراجع عن الإدراج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": true,
          "masked": false,
          "rtl": true,
          "text": "شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل. شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل. شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل."
        }
      },
      "panel": null,
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
    "noModel": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "download",
            "kind": "bar",
            "label": "تنزيل · 776 ميغابايت",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "no-model",
          "text": "لا يوجد نموذج للغة العربية بعد"
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "nothing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "إيقاف",
            "on": true,
            "tone": "primary"
          }
        ],
        "elapsedMs": 1200,
        "hint": "لم يُسمع شيء بعد",
        "lang": {
          "arabic": true,
          "name": "العربية",
          "rtl": true
        },
        "state": "listening",
        "waveMs": 300,
        "word": "جارٍ الاستماع…"
      },
      "panel": null,
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
    "password": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "تراجع عن الإدراج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "password",
          "text": "كُتب في حقل كلمة مرور، فلا يُحفَظ ولا يُنسَخ."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": true,
          "rtl": false,
          "text": "••••••••"
        }
      },
      "panel": null,
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
    "stopped": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "نسخ",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "تراجع عن الإدراج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "stopped",
          "text": "توقفت كتابة النص قبل اكتمالها. انسخه كاملًا من هنا أو من السجل."
        },
        "state": "text",
        "text": {
          "arabic": true,
          "masked": false,
          "rtl": true,
          "text": "شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل."
        }
      },
      "panel": null,
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
    "text": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "نسخ",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "تراجع عن الإدراج",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": true,
          "masked": false,
          "rtl": true,
          "text": "شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل."
        }
      },
      "panel": null,
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
    "transcribing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "إلغاء",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "إيقاف",
            "on": false,
            "tone": "primary"
          }
        ],
        "state": "transcribing",
        "word": "جارٍ تحويل الكلام إلى نص…"
      },
      "panel": null,
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
    "undone": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "نسخ",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "تراجع عن الإدراج",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "إغلاق",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": true,
          "masked": false,
          "rtl": true,
          "text": "شكرًا على التحديث. سأرسل الاستمارة الموقّعة صباح يوم الخميس، وستصل النسختان اللتان تحدثنا عنهما بالبريد في الأسبوع المقبل."
        }
      },
      "panel": null,
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
    }
  },
  "en": {
    "cancelled": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "anyway",
            "kind": "bar",
            "label": "Transcribe anyway",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "cancelled",
          "text": "Cancelled. Nothing was typed."
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "checking": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "checking",
          "text": "The chosen file is still being checked. Try again in a moment."
        },
        "mic": true,
        "state": "note"
      },
      "panel": null,
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
    "copyFailed": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copy",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Undo insert",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "copy-failed",
          "text": "The text could not be copied. Try again."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week."
        }
      },
      "panel": null,
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
    "fault": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "fault",
          "text": "Dictation stopped after an error. Try again."
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "heardNothing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "listen",
            "kind": "bar",
            "label": "Listen again",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "nothing",
          "text": "Nothing heard. Is the microphone on?"
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "hidden": {
      "bar": {
        "state": "hidden"
      },
      "panel": null,
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
    "listening": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "Stop",
            "on": true,
            "tone": "primary"
          }
        ],
        "elapsedMs": 1200,
        "hint": null,
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "state": "listening",
        "waveMs": 300,
        "word": "Listening…"
      },
      "panel": null,
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
    "long": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copy",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Undo insert",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week. Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week. Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week."
        }
      },
      "panel": null,
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
    "noModel": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "download",
            "kind": "bar",
            "label": "Download · 776 MB",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "no-model",
          "text": "No model for English yet"
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "nothing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "Stop",
            "on": true,
            "tone": "primary"
          }
        ],
        "elapsedMs": 1200,
        "hint": "Nothing heard yet",
        "lang": {
          "arabic": false,
          "name": "English",
          "rtl": false
        },
        "state": "listening",
        "waveMs": 300,
        "word": "Listening…"
      },
      "panel": null,
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
    "password": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Undo insert",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "password",
          "text": "Typed into a password box. Not kept, no Copy."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": true,
          "rtl": false,
          "text": "••••••••"
        }
      },
      "panel": null,
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
    "stopped": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copy",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Undo insert",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "stopped",
          "text": "Typing stopped part way. Copy the whole text from here or from History."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week."
        }
      },
      "panel": null,
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
    "text": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copy",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Undo insert",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week."
        }
      },
      "panel": null,
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
    "transcribing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "Cancel",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "Stop",
            "on": false,
            "tone": "primary"
          }
        ],
        "state": "transcribing",
        "word": "Transcribing…"
      },
      "panel": null,
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
    "undone": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copy",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Undo insert",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Close",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Thanks for the update. I will send the signed form on Thursday morning, and the two copies we talked about will follow by post next week."
        }
      },
      "panel": null,
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
    }
  },
  "fr": {
    "cancelled": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "anyway",
            "kind": "bar",
            "label": "Transcrire quand même",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "cancelled",
          "text": "Annulé. Rien n’a été tapé."
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "checking": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "checking",
          "text": "Le fichier choisi est encore en cours de vérification. Réessayez dans un instant."
        },
        "mic": true,
        "state": "note"
      },
      "panel": null,
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
    "copyFailed": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copier",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Annuler l’insertion",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "copy-failed",
          "text": "Le texte n’a pas pu être copié. Réessayez."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine."
        }
      },
      "panel": null,
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
    "fault": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "fault",
          "text": "La dictée s’est arrêtée après une erreur. Réessayez."
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "heardNothing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "listen",
            "kind": "bar",
            "label": "Écouter de nouveau",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "nothing",
          "text": "Rien entendu. Le micro est-il allumé ?"
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "hidden": {
      "bar": {
        "state": "hidden"
      },
      "panel": null,
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
    "listening": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "Arrêter",
            "on": true,
            "tone": "primary"
          }
        ],
        "elapsedMs": 1200,
        "hint": null,
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "state": "listening",
        "waveMs": 300,
        "word": "Écoute…"
      },
      "panel": null,
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
    "long": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copier",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Annuler l’insertion",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine. Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine. Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine."
        }
      },
      "panel": null,
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
    "noModel": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "download",
            "kind": "bar",
            "label": "Télécharger · 776 Mo",
            "on": true,
            "tone": "primary"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "no-model",
          "text": "Aucun modèle en Français pour l’instant"
        },
        "mic": false,
        "state": "note"
      },
      "panel": null,
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
    "nothing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "Arrêter",
            "on": true,
            "tone": "primary"
          }
        ],
        "elapsedMs": 1200,
        "hint": "Rien entendu pour l’instant",
        "lang": {
          "arabic": false,
          "name": "Français",
          "rtl": false
        },
        "state": "listening",
        "waveMs": 300,
        "word": "Écoute…"
      },
      "panel": null,
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
    "password": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Annuler l’insertion",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "password",
          "text": "Tapé dans un champ de mot de passe. Non conservé, sans copie."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": true,
          "rtl": false,
          "text": "••••••••"
        }
      },
      "panel": null,
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
    "stopped": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copier",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Annuler l’insertion",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": {
          "id": "stopped",
          "text": "La saisie s’est arrêtée en cours de route. Copiez le texte entier ici ou depuis l’historique."
        },
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine."
        }
      },
      "panel": null,
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
    "text": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copier",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Annuler l’insertion",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine."
        }
      },
      "panel": null,
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
    "transcribing": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "cancel",
            "kind": "bar",
            "label": "Annuler",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "stop",
            "kind": "bar",
            "label": "Arrêter",
            "on": false,
            "tone": "primary"
          }
        ],
        "state": "transcribing",
        "word": "Transcription…"
      },
      "panel": null,
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
    "undone": {
      "bar": {
        "buttons": [
          {
            "arg": 0,
            "id": "copy",
            "kind": "bar",
            "label": "Copier",
            "on": true,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "undo",
            "kind": "bar",
            "label": "Annuler l’insertion",
            "on": false,
            "tone": "plain"
          },
          {
            "arg": 0,
            "id": "close",
            "kind": "bar",
            "label": "Fermer",
            "on": true,
            "tone": "plain"
          }
        ],
        "hint": null,
        "state": "text",
        "text": {
          "arabic": false,
          "masked": false,
          "rtl": false,
          "text": "Merci pour la mise à jour. J’enverrai le formulaire signé jeudi matin, et les deux copies dont nous avons parlé suivront par la poste la semaine prochaine."
        }
      },
      "panel": null,
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
    }
  }
};

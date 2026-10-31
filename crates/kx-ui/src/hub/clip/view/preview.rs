//! The preview page as QML reads it: text with its lines kept and no hidden marks, the image's URL of the kept round,
//! or the files' paths, each marked when it holds Arabic.

use super::url;
use crate::I18n;
use crate::hub::clip::PREVIEW;
use crate::i18n::clean;
use crate::i18n::keys::CLIP_CHARS;
use kx_module_api::ports::Preview;
use kx_module_api::{is_arabic, is_rtl};
use serde::Serialize;

/// What ends a line of the preview's text.
const BREAK: char = '\n';
/// A tab, which the preview shows as a space.
const TAB: char = '\t';
/// What the preview shows in place of a tab.
const SPACE: &str = " ";

/// The preview page's content, by kind.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(super) enum Shown {
    Text {
        text: String,
        chars: String,
        arabic: bool,
        rtl: bool,
    },
    Image {
        url: Option<String>,
    },
    Files {
        paths: Vec<FilePath>,
    },
}

/// A file's path without hidden marks, and whether it holds Arabic, so it takes the Arabic font as a row title does.
#[derive(Serialize)]
pub(super) struct FilePath {
    path: String,
    arabic: bool,
}

/// Preview `p` of view `round`: text with its lines kept, the image's URL, or the files' paths.
pub(super) fn preview(p: &Preview, round: u32, t: &I18n) -> Shown {
    match p {
        Preview::Text { text, chars } => {
            let text = lines(text.expose());
            let chars = usize::try_from(*chars).unwrap_or(usize::MAX);
            Shown::Text {
                chars: t.counted(CLIP_CHARS, chars),
                arabic: is_arabic(&text),
                rtl: is_rtl(&text),
                text,
            }
        }
        Preview::Image(image) => Shown::Image {
            url: image.as_ref().map(|_| url(round, PREVIEW)),
        },
        Preview::Files(paths) => Shown::Files {
            paths: paths
                .expose()
                .iter()
                .map(|p| path(&p.to_string_lossy()))
                .collect(),
        },
    }
}

/// `p` without hidden marks, marked when it holds Arabic.
fn path(p: &str) -> FilePath {
    let path = clean(p);
    FilePath {
        arabic: is_arabic(&path),
        path,
    }
}

/// `text` with its line breaks kept, each line without hidden marks and its tabs as spaces.
fn lines(text: &str) -> String {
    let each: Vec<String> = text
        .split(BREAK)
        .map(|l| clean(&l.replace(TAB, SPACE)))
        .collect();
    each.join(&BREAK.to_string())
}

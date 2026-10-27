//! The UI Automation client's pure parts, moved from the spike with it: the selection budget, the password flag, the
//! arrays Windows hands back and the line boxes made from them.

use super::read::spend;
use super::*;
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayCreateVector, SafeArrayUnaccessData,
};
use windows::Win32::System::Variant::{VARENUM, VT_I4, VT_R8};
use windows::Win32::UI::WindowsAndMessaging::ES_PASSWORD;

/// The password style bit as a style word.
#[allow(
    clippy::cast_sign_loss,
    reason = "ES_PASSWORD is a small positive bit."
)]
const PW: u32 = ES_PASSWORD as u32;

/// A box from its four edges.
fn r(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

/// A vector of `kind` holding `values`, made here for `vector` to read and free.
#[allow(unsafe_code, reason = "Makes and fills a test array.")]
fn array<T: Copy>(kind: VARENUM, values: &[T]) -> *mut windows::Win32::System::Com::SAFEARRAY {
    let len = u32::try_from(values.len()).unwrap();
    // SAFETY: a new array of `len` elements; the fill stays in its bounds and the lock is undone.
    unsafe {
        let sa = SafeArrayCreateVector(kind, 0, len);
        let mut data = std::ptr::null_mut();
        SafeArrayAccessData(sa, &raw mut data).unwrap();
        std::slice::from_raw_parts_mut(data.cast::<T>(), values.len()).copy_from_slice(values);
        SafeArrayUnaccessData(sa).unwrap();
        sa
    }
}

#[test]
fn one_budget_covers_every_selected_range() {
    let (mut out, mut left) = (String::new(), 4);
    for part in ["abc", "déf", "ghi"] {
        spend(&mut out, part, &mut left);
    }
    assert_eq!((out.as_str(), left), ("abcd", 0));
}

#[test]
fn an_unreadable_uia_answer_counts_as_a_password() {
    assert!(password_flag(None, "", 0));
}

#[test]
fn office_rich_edit_boxes_are_edit_boxes() {
    assert!(password_flag(Some(false), "RICHEDIT60W", PW));
}

#[test]
#[allow(
    unsafe_code,
    reason = "Hands test arrays to `vector`, which frees them."
)]
fn vector_reads_numbers_of_its_size_and_skips_other_shapes() {
    // SAFETY: arrays made here, each handed over once to `vector`, which frees it.
    unsafe {
        let doubles = array(VT_R8, &[1.5, 2.0, -3.0]);
        assert_eq!(vector::<f64>(doubles), Ok(vec![1.5, 2.0, -3.0]));
        let ids = array(VT_I4, &[42_i32, 7]);
        assert_eq!(vector::<i32>(ids), Ok(vec![42, 7]));
        assert_eq!(vector::<f64>(array(VT_I4, &[1_i32, 2, 3])), Ok(vec![]));
        assert_eq!(vector::<f64>(std::ptr::null_mut()), Ok(vec![]));
    }
}

#[test]
fn uia_saying_password_is_enough() {
    assert!(password_flag(Some(true), "", 0));
}

#[test]
fn an_edit_box_with_the_password_style_counts_even_when_uia_says_no() {
    assert!(password_flag(Some(false), "Edit", PW));
    assert!(password_flag(Some(false), "edit", PW));
    assert!(password_flag(Some(false), "RichEdit20W", PW));
}

#[test]
fn the_style_bit_means_nothing_outside_an_edit_box() {
    assert!(!password_flag(
        Some(false),
        "Chrome_RenderWidgetHostHWND",
        PW
    ));
}

#[test]
fn neither_source_saying_so_is_not_a_password() {
    assert!(!password_flag(Some(false), "", 0));
    assert!(!password_flag(Some(false), "Edit", 0));
}

#[test]
fn line_boxes_come_from_groups_of_four_and_skip_empty_ones() {
    let v = [10.0, 20.0, 100.4, 18.6, 5.0, 5.0, 0.0, 10.0, 1.0, 2.0];
    assert_eq!(rects_of(&v), [r(10, 20, 110, 39)]);
    assert!(rects_of(&[]).is_empty());
}

#[test]
fn boxes_that_are_not_finite_are_dropped() {
    assert!(rects_of(&[f64::INFINITY, 0.0, 10.0, 10.0]).is_empty());
    assert!(rects_of(&[0.0, f64::NAN, 10.0, 10.0]).is_empty());
}

#[test]
fn huge_boxes_from_another_app_saturate() {
    assert_eq!(rects_of(&[1e12, 0.0, 1e12, 10.0])[0].right, i32::MAX);
}

#[test]
fn a_window_with_no_handle_has_no_class_and_no_style() {
    assert_eq!(class_of(HWND::default()), "");
    assert_eq!(style_of(HWND::default()), 0);
}

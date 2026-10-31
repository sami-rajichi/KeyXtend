//! The rights verdict from made-up readings, the token lists read only within their buffer, and the Medium label
//! built with its SID inside it.

use std::mem::offset_of;

use crate::worker::access::{Rights, beyond, change_notify, listed, usable};
use crate::worker::label::{Label, MEDIUM};
use windows::Win32::Foundation::LUID;
use windows::Win32::Security::{
    LUID_AND_ATTRIBUTES, SID_AND_ATTRIBUTES, TOKEN_MANDATORY_LABEL, TOKEN_PRIVILEGES,
    TOKEN_PRIVILEGES_ATTRIBUTES,
};
use windows::Win32::System::SystemServices::{
    SE_GROUP_ENABLED, SE_GROUP_ENABLED_BY_DEFAULT, SE_GROUP_MANDATORY, SE_GROUP_USE_FOR_DENY_ONLY,
    SECURITY_MANDATORY_HIGH_RID,
};

/// Words in a made-up list buffer.
const WORDS: usize = 4;

#[test]
fn a_token_that_keeps_any_right_is_refused() {
    let clean = Rights {
        ui_access: false,
        level: MEDIUM,
        admin: false,
        privileged: false,
    };
    assert!(clean.reduced());
    let kept = [
        Rights {
            ui_access: true,
            ..clean
        },
        Rights {
            level: SECURITY_MANDATORY_HIGH_RID.cast_unsigned(),
            ..clean
        },
        Rights {
            admin: true,
            ..clean
        },
        Rights {
            privileged: true,
            ..clean
        },
    ];
    for rights in kept {
        assert!(!rights.reduced(), "{rights:?}");
    }
}

#[test]
fn administrators_can_be_used_unless_deny_only() {
    let on = SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY;
    assert!(usable(on.cast_unsigned()), "enabled");
    assert!(usable(0), "disabled, which the process could undo");
    assert!(!usable(SE_GROUP_USE_FOR_DENY_ONLY.cast_unsigned()));
}

#[test]
fn any_privilege_but_change_notify_counts() {
    let kept = change_notify().unwrap();
    let other = LUID {
        LowPart: kept.LowPart.wrapping_add(1),
        ..kept
    };
    let held = |ids: &[LUID]| -> Vec<LUID_AND_ATTRIBUTES> {
        let off = TOKEN_PRIVILEGES_ATTRIBUTES::default();
        let entry = |&id: &LUID| LUID_AND_ATTRIBUTES {
            Luid: id,
            Attributes: off,
        };
        ids.iter().map(entry).collect()
    };
    assert!(!beyond(&held(&[]), kept), "none");
    assert!(!beyond(&held(&[kept]), kept), "change-notify alone");
    assert!(
        beyond(&held(&[kept, other]), kept),
        "one more, even disabled"
    );
}

#[test]
fn a_list_is_read_only_within_its_buffer() {
    let at = offset_of!(TOKEN_PRIVILEGES, Privileges);
    let fits = (WORDS * size_of::<usize>() - at) / size_of::<LUID_AND_ATTRIBUTES>();
    let buf = |count: usize| {
        let mut b = vec![0_usize; WORDS];
        b[0] = count;
        b
    };
    let len = |count| listed::<LUID_AND_ATTRIBUTES>(&buf(count), at).map(<[_]>::len);
    assert_eq!(len(fits), Some(fits));
    assert_eq!(len(fits + 1), None, "past the end");
    assert!(listed::<LUID_AND_ATTRIBUTES>(&[], at).is_none(), "no count");
    assert!(
        listed::<SID_AND_ATTRIBUTES>(&buf(1), 1).is_none(),
        "out of line"
    );
}

#[test]
fn the_medium_label_holds_its_sid_within_the_size_it_gives() {
    let mut label = Label::default();
    let size = usize::try_from(label.medium().unwrap()).unwrap();
    let sid = label.label.Label.Sid.0.cast_const().cast::<u32>();
    assert_eq!(sid, label.sid.as_ptr(), "right behind the label, inside it");
    assert_eq!(offset_of!(Label, sid), size_of::<TOKEN_MANDATORY_LABEL>());
    assert!(
        size > offset_of!(Label, sid) && size <= size_of::<Label>(),
        "{size}"
    );
    assert_eq!(label.level(), Some(MEDIUM));
    let moved = Box::new(label);
    assert_eq!(moved.level(), None, "a moved label's SID is elsewhere");
}

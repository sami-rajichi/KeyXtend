//! Well-known SIDs in buffers of ours, and the integrity label with its SID right behind it, as Windows reads and
//! writes `TokenIntegrityLevel`.

use std::mem::offset_of;

use kx_platform::pictures::SpawnError;
use windows::Win32::Security::{
    CreateWellKnownSid, GetSidLengthRequired, GetSidSubAuthority, GetSidSubAuthorityCount,
    IsValidSid, PSID, SECURITY_MAX_SID_SIZE, SID_AND_ATTRIBUTES, TOKEN_MANDATORY_LABEL,
    WELL_KNOWN_SID_TYPE, WinMediumLabelSid,
};
use windows::Win32::System::SystemServices::{SE_GROUP_INTEGRITY, SECURITY_MANDATORY_MEDIUM_RID};

use super::refused;

/// The integrity level's last SID part the worker runs at: Medium, S-1-16-8192.
pub(super) const MEDIUM: u32 = SECURITY_MANDATORY_MEDIUM_RID.cast_unsigned();

/// A SID buffer of the largest size, aligned as a SID's parts are.
pub(super) type SidRoom = [u32; SECURITY_MAX_SID_SIZE as usize / size_of::<u32>()];

/// Writes the well-known SID `kind` into `room`; hands back a pointer into `room` and the SID's length.
#[allow(unsafe_code, reason = "Builds a well-known SID in our buffer.")]
pub(super) fn well_known(
    kind: WELL_KNOWN_SID_TYPE,
    room: &mut SidRoom,
) -> Result<(PSID, u32), SpawnError> {
    let sid = PSID(room.as_mut_ptr().cast());
    let mut len = SECURITY_MAX_SID_SIZE;
    // SAFETY: `room` holds `len` writable bytes, aligned for a SID, for the call.
    unsafe { CreateWellKnownSid(kind, None, Some(sid), &raw mut len) }
        .map_err(refused("make a well-known SID"))?;
    Ok((sid, len))
}

/// An integrity label with room for its SID right behind it. Its SID pointer points into itself, so it is filled in
/// place and never moved while Windows reads it.
#[repr(C)]
#[derive(Default)]
pub(super) struct Label {
    pub(super) label: TOKEN_MANDATORY_LABEL,
    pub(super) sid: SidRoom,
}

impl Label {
    /// Fills this label with the Medium SID in its own `sid`; hands back the bytes that hold the label and its SID.
    pub(super) fn medium(&mut self) -> Result<u32, SpawnError> {
        let (sid, len) = well_known(WinMediumLabelSid, &mut self.sid)?;
        self.label = TOKEN_MANDATORY_LABEL {
            Label: SID_AND_ATTRIBUTES {
                Sid: sid,
                Attributes: SE_GROUP_INTEGRITY.cast_unsigned(),
            },
        };
        let head = u32::try_from(offset_of!(Self, sid)).map_err(|_| SpawnError::Rights)?;
        Ok(head.saturating_add(len))
    }

    /// The level's last SID part; `None` for a SID that is not valid or not wholly inside this label's own room.
    #[allow(unsafe_code, reason = "Reads the SID in this label's own room.")]
    pub(super) fn level(&self) -> Option<u32> {
        let sid = self.label.Label.Sid;
        let left = u32::try_from(self.left_from(sid)?).unwrap_or(u32::MAX);
        // SAFETY: `sid` points into `self.sid`, which outlives the reads. Its header is checked to fit in the `left`
        // bytes before Windows reads it, and its whole length before any part is read; the last part is in its count.
        unsafe {
            if left < GetSidLengthRequired(0) || !IsValidSid(sid).as_bool() {
                return None;
            }
            let count = *GetSidSubAuthorityCount(sid);
            if left < GetSidLengthRequired(count) {
                return None;
            }
            Some(*GetSidSubAuthority(sid, u32::from(count).checked_sub(1)?))
        }
    }

    /// The bytes of this label's room from `sid` on; `None` when `sid` points elsewhere, as after a move.
    fn left_from(&self, sid: PSID) -> Option<usize> {
        let room = self.sid.as_ptr_range();
        let (at, end) = (sid.0.addr(), room.end.addr());
        (room.start.addr()..end).contains(&at).then(|| end - at)
    }
}

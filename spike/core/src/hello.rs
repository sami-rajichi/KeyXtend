//! Windows Hello consent for one of our windows; quick-fill types only after a Verified answer.

use windows::Security::Credentials::UI::{
    UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::IUserConsentVerifierInterop;
use windows::core::{HSTRING, factory};
use windows_future::IAsyncOperation;

/// What Windows Hello answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// The user proved who they are.
    Verified,
    /// The user closed the prompt.
    Canceled,
    /// Hello is off, not set up, busy or refused for another reason.
    Unavailable,
}

/// True only for Verified: every other answer types nothing.
pub fn may_type(a: Answer) -> bool {
    a == Answer::Verified
}

/// Maps Hello's result; anything but Verified or Canceled counts as unavailable.
fn answer_of(r: UserConsentVerificationResult) -> Answer {
    match r {
        UserConsentVerificationResult::Verified => Answer::Verified,
        UserConsentVerificationResult::Canceled => Answer::Canceled,
        _ => Answer::Unavailable,
    }
}

/// Hello's error, as a short note.
fn err(e: windows::core::Error) -> String {
    format!("Windows Hello: {e}")
}

/// True when Windows Hello is set up and ready on this PC.
pub fn available() -> Result<bool, String> {
    let can = UserConsentVerifier::CheckAvailabilityAsync()
        .and_then(|op| op.join())
        .map_err(err)?;
    Ok(can == UserConsentVerifierAvailability::Available)
}

/// Asks Windows Hello, with its prompt owned by our window `owner`; blocks until the user answers.
pub fn ask(owner: HWND, message: &str) -> Result<Answer, String> {
    if !available()? {
        return Ok(Answer::Unavailable);
    }
    let interop = factory::<UserConsentVerifier, IUserConsentVerifierInterop>().map_err(err)?;
    // SAFETY: any handle is accepted, as a dead or foreign one makes the call fail; the message outlives the call.
    let op: IAsyncOperation<UserConsentVerificationResult> =
        unsafe { interop.RequestVerificationForWindowAsync(owner, &HSTRING::from(message)) }
            .map_err(err)?;
    Ok(answer_of(op.join().map_err(err)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_verified_types() {
        assert!(may_type(Answer::Verified));
        assert!(!may_type(Answer::Canceled));
        assert!(!may_type(Answer::Unavailable));
    }

    #[test]
    fn hello_results_map_to_three_answers() {
        use UserConsentVerificationResult as R;
        assert_eq!(answer_of(R::Verified), Answer::Verified);
        assert_eq!(answer_of(R::Canceled), Answer::Canceled);
        for other in [
            R::DeviceNotPresent,
            R::NotConfiguredForUser,
            R::DeviceBusy,
            R::RetriesExhausted,
            R::DisabledByPolicy,
        ] {
            assert_eq!(answer_of(other), Answer::Unavailable);
        }
    }
}

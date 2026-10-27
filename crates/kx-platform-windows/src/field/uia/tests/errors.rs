//! UI Automation's failures as the port's errors: its own timeout is `TimedOut`, any other a refusal.

use super::super::failed;
use kx_module_api::ports::FieldError;
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::UI::Accessibility::UIA_E_TIMEOUT;
use windows::core::{Error, HRESULT};

#[test]
fn a_uia_timeout_is_timed_out_and_any_other_failure_a_refusal() {
    let timeout = Error::from(HRESULT(UIA_E_TIMEOUT.cast_signed()));
    assert_eq!(failed(timeout), FieldError::TimedOut);
    assert_eq!(failed(Error::from(E_FAIL)), FieldError::Refused);
}

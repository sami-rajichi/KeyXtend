//! The memory port on Windows: after a look change, the heaps' free pages go back to the system (ADR-0023).

use std::ffi::c_void;

use kx_platform::ui::{Memory, UiError};
use windows::Win32::System::Memory::{HeapOptimizeResources, HeapSetInformation};
use windows::Win32::System::SystemServices::{
    HEAP_OPTIMIZE_RESOURCES_CURRENT_VERSION, HEAP_OPTIMIZE_RESOURCES_INFORMATION,
};

use crate::refused;

/// Decommits the free pages of the process heaps through `HeapOptimizeResources` (Windows 8.1 or later); with no heap
/// named, Windows takes every heap with a low-fragmentation front end, each of which serialises its own calls.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinMemory;

impl Memory for WinMemory {
    #[allow(unsafe_code, reason = "A heap call with a local struct and its size.")]
    fn decommit(&self) -> Result<(), UiError> {
        let info = HEAP_OPTIMIZE_RESOURCES_INFORMATION {
            Version: HEAP_OPTIMIZE_RESOURCES_CURRENT_VERSION,
            ..Default::default()
        };
        let data = std::ptr::from_ref(&info).cast::<c_void>();
        // SAFETY: `info` outlives the call and its exact size goes with it; no heap handle asks for every heap.
        unsafe { HeapSetInformation(None, HeapOptimizeResources, Some(data), size_of_val(&info)) }
            .map_err(|e| refused(&e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Only Windows' answer is checked: what the heaps give back depends on the whole process.
    #[test]
    fn windows_accepts_the_give_back_after_a_free() {
        let freed: Vec<Vec<u8>> = (0..1024).map(|n| vec![0; n]).collect();
        drop(freed);
        assert_eq!(WinMemory.decommit(), Ok(()));
    }
}

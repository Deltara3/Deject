use crate::catch_unwrap;
use crate::error::{InjectorError, InjectorResult};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};

/// Shared library injection implementation, should be dropped after injection.
pub struct Injector(HANDLE);

impl Injector {
    /// Constructs a new `Injector` from a Process ID.
    pub fn from_pid(pid: u32) -> InjectorResult<Self> {
        let handle = catch_unwrap!(unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) }, |_error| {
            return Err(InjectorError::OpenProcessFailed);
        });

        Ok(Self(handle))
    }
}

impl Drop for Injector {
    fn drop(&mut self) {
        // If this returns an error then something is really wrong, assume success.
        let _close_result = unsafe { CloseHandle(self.0) };
    }
}
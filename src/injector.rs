use crate::catch_unwrap;
use crate::error::{InjectorError, InjectorResult};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WIN32_ERROR};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;

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

    /// Returns if the process is a UWP one. 
    pub fn is_uwp(&self) -> bool {
        let mut size = 0;

        // If GetPackageFamilyName returns ERROR_INSUFFICIENT_BUFFER (122) then the app is likely UWP.
        if unsafe { GetPackageFamilyName(self.0, &mut size, None) } == WIN32_ERROR(122) {
            return true;
        }

        false
    }
}

impl Drop for Injector {
    fn drop(&mut self) {
        // If this returns an error then something is really wrong, assume success.
        let _close_result = unsafe { CloseHandle(self.0) };
    }
}
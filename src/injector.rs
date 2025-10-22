use std::ptr;
use crate::catch_unwrap;
use crate::error::{InjectorError, InjectorResult};
use windows::core::{HSTRING, PCWSTR, PWSTR, w};
use windows::Win32::Foundation::{CloseHandle, LocalFree, GENERIC_EXECUTE, GENERIC_READ, HANDLE, HLOCAL, WIN32_ERROR};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::Win32::Storage::Packaging::Appx::GetPackageFamilyName;
use windows::Win32::Security::{ACL, PSECURITY_DESCRIPTOR, PSID, DACL_SECURITY_INFORMATION, SUB_CONTAINERS_AND_OBJECTS_INHERIT};
use windows::Win32::Security::Authorization::{
    ConvertStringSidToSidW,
    GetNamedSecurityInfoW,
    SetEntriesInAclW,
    SetNamedSecurityInfoW,
    EXPLICIT_ACCESS_W,
    SET_ACCESS,
    SE_FILE_OBJECT,
    TRUSTEE_IS_SID,
    TRUSTEE_IS_WELL_KNOWN_GROUP
};

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

    //  TODO: Handle all errors?
    /// Fixes the access control on libraries when a process is detected as UWP.
    pub fn fix_access_control(module: &String) {
        let pc_module = PCWSTR(HSTRING::from(module).as_ptr());

        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();
        let mut explicit_access     = EXPLICIT_ACCESS_W::default();

        let mut access_control_current = ptr::null_mut::<ACL>();
        let mut access_control_new     = ptr::null_mut::<ACL>();
        let mut security_identifier    = PSID::default();

        let security_result = unsafe { GetNamedSecurityInfoW(
            pc_module,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut access_control_current),
            None,
            &mut security_descriptor
        ) };

        // If GetNamedSecurityInfoW returns ERROR_SUCCESS (0) then we continue.
        if security_result == WIN32_ERROR(0) {
            // Create SID for ALL_APPLICATION_PACKAGES.
            let _sid_result = unsafe { ConvertStringSidToSidW(w!("S-1-15-2-1"), &mut security_identifier) };

            if !security_identifier.is_invalid() {
                explicit_access.grfAccessPermissions = GENERIC_READ.0 | GENERIC_EXECUTE.0;
                explicit_access.grfAccessMode        = SET_ACCESS;
                explicit_access.grfInheritance       = SUB_CONTAINERS_AND_OBJECTS_INHERIT;
                explicit_access.Trustee.TrusteeForm  = TRUSTEE_IS_SID;
                explicit_access.Trustee.TrusteeType  = TRUSTEE_IS_WELL_KNOWN_GROUP;
                explicit_access.Trustee.ptstrName    = PWSTR(security_identifier.0.cast());

                let set_entries_result = unsafe { SetEntriesInAclW(
                    Some(&[explicit_access]),
                    Some(access_control_current),
                    &mut access_control_new) 
                };

                // If SetEntriesInAclW returns ERROR_SUCCESS (0) then we continue.
                if set_entries_result == WIN32_ERROR(0) {
                    let _set_info_result = unsafe { SetNamedSecurityInfoW(
                        pc_module,
                        SE_FILE_OBJECT,
                        DACL_SECURITY_INFORMATION,
                        None,
                        None,
                        Some(access_control_new),
                        None
                    ) };
                }
            }
        }

        if !security_descriptor.is_invalid() {
            unsafe { LocalFree(Some(HLOCAL(security_descriptor.0.cast()))) };
        }

        if !access_control_new.is_null() {
            unsafe { LocalFree(Some(HLOCAL(access_control_new.cast()))) };
        }
    }
}

impl Drop for Injector {
    fn drop(&mut self) {
        // If this returns an error then something is really wrong, assume success.
        let _close_result = unsafe { CloseHandle(self.0) };
    }
}
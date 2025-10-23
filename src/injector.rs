use std::ptr;
use std::mem;
use core::ffi::c_void;
use crate::{catch_unwrap, pcwstr};
use crate::error::{InjectorError, InjectorResult};
use windows::core::{HSTRING, PCWSTR, PWSTR, PCSTR, w, s};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::Foundation::{CloseHandle, LocalFree, GENERIC_EXECUTE, GENERIC_READ, HANDLE, HLOCAL, WIN32_ERROR};
use windows::Win32::System::Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RESERVE, MEM_RELEASE, PAGE_READWRITE};
use windows::Win32::System::Threading::{OpenProcess, CreateRemoteThread, WaitForSingleObject, INFINITE, PROCESS_ALL_ACCESS};
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

/// String used to locate Kernel32.
const KERNEL32_W: PCWSTR    = w!("kernel32.dll");
/// String used to locate LoadLibraryW.
const LOAD_LIBRARY_W: PCSTR = s!("LoadLibraryW");

/// Callback type for `CreateRemoteThread`.
type RoutineCallback = unsafe extern "system" fn(*mut c_void) -> u32;

/// Shared library injection implementation, should be dropped after injection.
pub struct Injector {
    /// Remote process associated with this `Injector` instance.
    process: HANDLE,
    /// Determines if `Injector` instance should perform cleanup operations.
    cleanup: bool
}

impl Injector {
    /// Constructs a new `Injector` from a Process ID.
    pub fn from_pid(pid: u32, do_cleanup: bool) -> InjectorResult<Self> {
        let handle = catch_unwrap!(unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) }, |_error| {
            return Err(InjectorError::OpenProcessFailed);
        });

        Ok(Self {
            process: handle,
            cleanup: do_cleanup
        })
    }

    /// Returns if the process is a UWP one. 
    pub fn is_uwp(&self) -> bool {
        let mut size = 0;

        // If GetPackageFamilyName returns ERROR_INSUFFICIENT_BUFFER (122) then the app is likely UWP.
        if unsafe { GetPackageFamilyName(self.process, &mut size, None) } == WIN32_ERROR(122) {
            return true;
        }

        false
    }

    //  TODO: Handle all errors?
    /// Fixes the access control on libraries when a process is detected as UWP.
    pub fn fix_access_control(module: &String) {
        let pc_module  = pcwstr!(module);

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

    /// Injects a given library into the associated process.
    pub fn inject_dll(&self, module: &String) -> InjectorResult<()> {
        let pc_module  = pcwstr!(module);
        let module_len = unsafe { pc_module.as_wide() }.len() * 2 + 2;

        // Allocate memory in the remote process, handling any error.
        let remote_addr = unsafe { VirtualAllocEx(self.process, None, module_len, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE) };
        if remote_addr.is_null() {
            return Err(InjectorError::RemoteAllocFailed);
        }

        // Write our path data to the remote address, handling any error.
        catch_unwrap!(unsafe { WriteProcessMemory(self.process, remote_addr, pc_module.as_ptr() as *const _, module_len, None) }, |_err| {
            // If this returns an error then something is really wrong, assume success.
            let _free_result = unsafe { VirtualFreeEx(self.process, remote_addr, 0, MEM_RELEASE) };
            return Err(InjectorError::RemoteWriteFailed);
        });

        // Find Kernel32, handling any error.
        let kernel32 = catch_unwrap!(unsafe { GetModuleHandleW(KERNEL32_W) }, |_err| {
            // If this returns an error then something is really wrong, assume success.
            let _free_result = unsafe { VirtualFreeEx(self.process, remote_addr, 0, MEM_RELEASE) };
            return Err(InjectorError::RemoteWriteFailed);
        });

        let load_library = catch_unwrap!(unsafe { GetProcAddress(kernel32, LOAD_LIBRARY_W) }, || {
            // If this returns an error then something is really wrong, assume success.
            let _free_result = unsafe { VirtualFreeEx(self.process, remote_addr, 0, MEM_RELEASE) };
            return Err(InjectorError::GetProcedureFailed);
        });

        // Transmute LoadLibraryW so we can pass it to CreateRemoteThread.
        let routine: RoutineCallback = unsafe { mem::transmute(load_library) };

        // Create remote thread to inject the given library.
        let thread_result = unsafe { CreateRemoteThread(self.process, None, 0, Some(routine), Some(remote_addr), 0, None) };
        let thread_handle = catch_unwrap!(thread_result, |_err| {
            // If this returns an error then something is really wrong, assume success.
            let _free_result = unsafe { VirtualFreeEx(self.process, remote_addr, 0, MEM_RELEASE) };
            return Err(InjectorError::CreateThreadFailed);
        });

        // Wait for thread exit and cleanup.
        if self.cleanup {
            let _wait_result  = unsafe { WaitForSingleObject(thread_handle, INFINITE) };
            let _close_result = unsafe { CloseHandle(thread_handle) };
        }

        Ok(())
    }
}

impl Drop for Injector {
    fn drop(&mut self) {
        // If this returns an error then something is really wrong, assume success.
        let _close_result = unsafe { CloseHandle(self.process) };
    }
}
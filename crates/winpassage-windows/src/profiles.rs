use anyhow::Result;

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, bail};
    use std::ffi::c_void;
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};

    fn to_wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }

    unsafe fn from_wide_ptr(ptr: *const u16) -> Option<String> {
        if ptr.is_null() {
            return None;
        }

        let mut len = 0usize;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        Some(String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len)))
    }

    #[link(name = "Advapi32")]
    extern "system" {
        fn LookupAccountNameW(
            system_name: *const u16,
            account_name: *const u16,
            sid: *mut c_void,
            cb_sid: *mut u32,
            referenced_domain_name: *mut u16,
            cch_referenced_domain_name: *mut u32,
            pe_use: *mut u32,
        ) -> i32;

        fn ConvertSidToStringSidW(sid: *const c_void, string_sid: *mut *mut u16) -> i32;
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn LocalFree(hmem: *mut c_void) -> *mut c_void;
    }

    #[link(name = "Userenv")]
    extern "system" {
        fn DeleteProfileW(sid_string: *const u16, profile_path: *const u16, computer_name: *const u16) -> i32;
    }

    struct LocalHandle<T>(*mut T);

    impl<T> Drop for LocalHandle<T> {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let _ = LocalFree(self.0.cast::<c_void>());
                }
            }
        }
    }

    fn account_sid_string(username: &str) -> Result<String> {
        let username_wide = to_wide(username);
        let mut sid_len = 0u32;
        let mut domain_len = 0u32;
        let mut sid_use = 0u32;

        unsafe {
            let _ = LookupAccountNameW(
                null(),
                username_wide.as_ptr(),
                null_mut(),
                &mut sid_len,
                null_mut(),
                &mut domain_len,
                &mut sid_use,
            );
        }

        if sid_len == 0 {
            bail!("could not resolve SID for local account {username}");
        }

        let mut sid = vec![0u8; sid_len as usize];
        let mut domain = vec![0u16; domain_len.max(1) as usize];
        let ok = unsafe {
            LookupAccountNameW(
                null(),
                username_wide.as_ptr(),
                sid.as_mut_ptr().cast::<c_void>(),
                &mut sid_len,
                domain.as_mut_ptr(),
                &mut domain_len,
                &mut sid_use,
            )
        };

        if ok == 0 {
            bail!("LookupAccountNameW failed for local account {username}");
        }

        let mut sid_string_ptr: *mut u16 = null_mut();
        let ok = unsafe {
            ConvertSidToStringSidW(sid.as_ptr().cast::<c_void>(), &mut sid_string_ptr)
        };

        if ok == 0 || sid_string_ptr.is_null() {
            bail!("ConvertSidToStringSidW failed for local account {username}");
        }

        let _owned = LocalHandle(sid_string_ptr);
        unsafe { from_wide_ptr(sid_string_ptr) }
            .ok_or_else(|| anyhow!("resolved SID string was empty for local account {username}"))
    }

    pub fn delete_local_user_profile(username: &str) -> Result<()> {
        let sid = account_sid_string(username)?;
        let sid_wide = to_wide(&sid);
        let ok = unsafe { DeleteProfileW(sid_wide.as_ptr(), null(), null()) };

        if ok == 0 {
            bail!("DeleteProfileW failed for local account {username}");
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    use anyhow::bail;

    pub fn delete_local_user_profile(_username: &str) -> Result<()> {
        bail!("local Windows profile deletion is available only on Windows")
    }
}

pub fn delete_local_user_profile(username: &str) -> Result<()> {
    imp::delete_local_user_profile(username)
}

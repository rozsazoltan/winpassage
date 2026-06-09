use anyhow::Result;
use winpassage_protocol::LocalUserSummary;

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, bail, Context};
    use std::ffi::OsStr;
    use std::ffi::c_void;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};
    use zeroize::Zeroize;

    const NERR_SUCCESS: u32 = 0;
    const MAX_PREFERRED_LENGTH: u32 = u32::MAX;
    const FILTER_NORMAL_ACCOUNT: u32 = 0x0002;
    const UF_ACCOUNTDISABLE: u32 = 0x0002;
    const UF_PASSWD_NOTREQD: u32 = 0x0020;

    #[repr(C)]
    struct UserInfo1 {
        usri1_name: *mut u16,
        usri1_password: *mut u16,
        usri1_password_age: u32,
        usri1_priv: u32,
        usri1_home_dir: *mut u16,
        usri1_comment: *mut u16,
        usri1_flags: u32,
        usri1_script_path: *mut u16,
    }

    #[repr(C)]
    struct UserInfo1003 {
        usri1003_password: *mut u16,
    }

    #[link(name = "Netapi32")]
    extern "system" {
        fn NetUserEnum(
            servername: *const u16,
            level: u32,
            filter: u32,
            bufptr: *mut *mut u8,
            prefmaxlen: u32,
            entriesread: *mut u32,
            totalentries: *mut u32,
            resume_handle: *mut u32,
        ) -> u32;

        fn NetUserSetInfo(
            servername: *const u16,
            username: *const u16,
            level: u32,
            buf: *mut u8,
            parm_err: *mut u32,
        ) -> u32;

        fn NetUserChangePassword(
            domainname: *const u16,
            username: *const u16,
            oldpassword: *const u16,
            newpassword: *const u16,
        ) -> u32;

        fn NetApiBufferFree(buffer: *mut c_void) -> u32;
    }

    struct NetBuffer(*mut u8);

    impl Drop for NetBuffer {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let _ = NetApiBufferFree(self.0.cast::<c_void>());
                }
            }
        }
    }

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

    fn net_error(context: &str, code: u32) -> anyhow::Error {
        anyhow!("{context} failed with Win32/NetAPI status {code}")
    }

    pub fn list_local_users() -> Result<Vec<LocalUserSummary>> {
        let mut buffer: *mut u8 = null_mut();
        let mut entries_read = 0u32;
        let mut total_entries = 0u32;
        let mut resume_handle = 0u32;
        let mut users = Vec::new();

        loop {
            let status = unsafe {
                NetUserEnum(
                    null(),
                    1,
                    FILTER_NORMAL_ACCOUNT,
                    &mut buffer,
                    MAX_PREFERRED_LENGTH,
                    &mut entries_read,
                    &mut total_entries,
                    &mut resume_handle,
                )
            };

            if status != NERR_SUCCESS && status != 234 {
                bail!(net_error("NetUserEnum", status));
            }

            let _owned = NetBuffer(buffer);

            if !buffer.is_null() && entries_read > 0 {
                let slice = unsafe {
                    std::slice::from_raw_parts(buffer.cast::<UserInfo1>(), entries_read as usize)
                };

                for item in slice {
                    let username = unsafe { from_wide_ptr(item.usri1_name) }
                        .unwrap_or_else(|| "<unknown>".to_string());
                    let full_name = unsafe { from_wide_ptr(item.usri1_comment) };
                    let disabled = item.usri1_flags & UF_ACCOUNTDISABLE != 0;
                    let password_required = item.usri1_flags & UF_PASSWD_NOTREQD == 0;

                    users.push(LocalUserSummary {
                        username,
                        full_name,
                        disabled,
                        password_required,
                    });
                }
            }

            buffer = null_mut();

            if status == NERR_SUCCESS {
                break;
            }
        }

        users.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
        Ok(users)
    }

    pub fn reset_local_user_password(username: &str, new_password: &str) -> Result<()> {
        let username_wide = to_wide(username);
        let mut password_wide = to_wide(new_password);
        let mut user_info = UserInfo1003 {
            usri1003_password: password_wide.as_mut_ptr(),
        };
        let mut parameter_error = 0u32;

        let status = unsafe {
            NetUserSetInfo(
                null(),
                username_wide.as_ptr(),
                1003,
                (&mut user_info as *mut UserInfo1003).cast::<u8>(),
                &mut parameter_error,
            )
        };

        password_wide.zeroize();

        if status != NERR_SUCCESS {
            return Err(net_error("NetUserSetInfo", status)
                .context(format!("parameter error: {parameter_error}")));
        }

        Ok(())
    }

    pub fn change_own_password(username: &str, current_password: &str, new_password: &str) -> Result<()> {
        let username_wide = to_wide(username);
        let mut current_password_wide = to_wide(current_password);
        let mut new_password_wide = to_wide(new_password);

        let status = unsafe {
            NetUserChangePassword(
                null(),
                username_wide.as_ptr(),
                current_password_wide.as_ptr(),
                new_password_wide.as_ptr(),
            )
        };

        current_password_wide.zeroize();
        new_password_wide.zeroize();

        if status != NERR_SUCCESS {
            return Err(net_error("NetUserChangePassword", status));
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    use anyhow::bail;

    pub fn list_local_users() -> Result<Vec<LocalUserSummary>> {
        bail!("local Windows user enumeration is available only on Windows")
    }

    pub fn reset_local_user_password(_username: &str, _new_password: &str) -> Result<()> {
        bail!("local Windows password reset is available only on Windows")
    }

    pub fn change_own_password(_username: &str, _current_password: &str, _new_password: &str) -> Result<()> {
        bail!("local Windows password change is available only on Windows")
    }
}

pub fn list_local_users() -> Result<Vec<LocalUserSummary>> {
    imp::list_local_users()
}

pub fn reset_local_user_password(username: &str, new_password: &str) -> Result<()> {
    imp::reset_local_user_password(username, new_password)
}

pub fn change_own_password(username: &str, current_password: &str, new_password: &str) -> Result<()> {
    imp::change_own_password(username, current_password, new_password)
}

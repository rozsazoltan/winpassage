use anyhow::Result;
use winpassage_protocol::LocalUserSummary;

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, bail, Context};
    use std::ffi::c_void;
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};
    use zeroize::Zeroize;

    const NERR_SUCCESS: u32 = 0;
    const ERROR_MORE_DATA: u32 = 234;
    const MAX_PREFERRED_LENGTH: u32 = u32::MAX;
    const FILTER_NORMAL_ACCOUNT: u32 = 0x0002;
    const USER_PRIV_USER: u32 = 1;
    const UF_SCRIPT: u32 = 0x0001;
    const UF_ACCOUNTDISABLE: u32 = 0x0002;
    const UF_PASSWD_NOTREQD: u32 = 0x0020;
    const UF_NORMAL_ACCOUNT: u32 = 0x0200;
    const UF_PASSWORD_EXPIRED: u32 = 0x800000;

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

    #[repr(C)]
    struct UserInfo1008 {
        usri1008_flags: u32,
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

        fn NetUserGetInfo(
            servername: *const u16,
            username: *const u16,
            level: u32,
            bufptr: *mut *mut u8,
        ) -> u32;

        fn NetUserAdd(servername: *const u16, level: u32, buf: *mut u8, parm_err: *mut u32) -> u32;

        fn NetUserDel(servername: *const u16, username: *const u16) -> u32;

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

        Some(String::from_utf16_lossy(std::slice::from_raw_parts(
            ptr, len,
        )))
    }

    fn net_error(context: &str, code: u32) -> anyhow::Error {
        anyhow!("{context} failed with Win32/NetAPI status {code}")
    }

    fn get_user_flags(username: &str) -> Result<u32> {
        let username_wide = to_wide(username);
        let mut buffer: *mut u8 = null_mut();
        let status = unsafe { NetUserGetInfo(null(), username_wide.as_ptr(), 1, &mut buffer) };

        if status != NERR_SUCCESS {
            return Err(net_error("NetUserGetInfo", status));
        }

        let _owned = NetBuffer(buffer);
        let info = unsafe { &*(buffer.cast::<UserInfo1>()) };
        Ok(info.usri1_flags)
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

            if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
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
                        is_administrator: false,
                        active_session_count: 0,
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

    pub fn create_local_user(
        username: &str,
        full_name: Option<&str>,
        password: &str,
        must_change_password: bool,
        enabled: bool,
    ) -> Result<()> {
        let mut username_wide = to_wide(username);
        let mut password_wide = to_wide(password);
        let mut full_name_wide = full_name.map(to_wide).unwrap_or_else(|| vec![0]);
        let mut flags = UF_SCRIPT | UF_NORMAL_ACCOUNT;

        if !enabled {
            flags |= UF_ACCOUNTDISABLE;
        }

        if must_change_password {
            flags |= UF_PASSWORD_EXPIRED;
        }

        let mut user_info = UserInfo1 {
            usri1_name: username_wide.as_mut_ptr(),
            usri1_password: password_wide.as_mut_ptr(),
            usri1_password_age: 0,
            usri1_priv: USER_PRIV_USER,
            usri1_home_dir: null_mut(),
            usri1_comment: full_name_wide.as_mut_ptr(),
            usri1_flags: flags,
            usri1_script_path: null_mut(),
        };
        let mut parameter_error = 0u32;

        let status = unsafe {
            NetUserAdd(
                null(),
                1,
                (&mut user_info as *mut UserInfo1).cast::<u8>(),
                &mut parameter_error,
            )
        };

        password_wide.zeroize();
        username_wide.zeroize();
        full_name_wide.zeroize();

        if status != NERR_SUCCESS {
            return Err(net_error("NetUserAdd", status)
                .context(format!("parameter error: {parameter_error}")));
        }

        Ok(())
    }

    pub fn delete_local_user(username: &str) -> Result<()> {
        let username_wide = to_wide(username);
        let status = unsafe { NetUserDel(null(), username_wide.as_ptr()) };

        if status != NERR_SUCCESS {
            return Err(net_error("NetUserDel", status));
        }

        Ok(())
    }

    pub fn set_local_user_enabled(username: &str, enabled: bool) -> Result<()> {
        let current_flags = get_user_flags(username)?;
        let mut next_flags = current_flags;

        if enabled {
            next_flags &= !UF_ACCOUNTDISABLE;
        } else {
            next_flags |= UF_ACCOUNTDISABLE;
        }

        let username_wide = to_wide(username);
        let mut user_info = UserInfo1008 {
            usri1008_flags: next_flags,
        };
        let mut parameter_error = 0u32;

        let status = unsafe {
            NetUserSetInfo(
                null(),
                username_wide.as_ptr(),
                1008,
                (&mut user_info as *mut UserInfo1008).cast::<u8>(),
                &mut parameter_error,
            )
        };

        if status != NERR_SUCCESS {
            return Err(net_error("NetUserSetInfo", status)
                .context(format!("parameter error: {parameter_error}")));
        }

        Ok(())
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

    pub fn change_own_password(
        username: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<()> {
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

    pub fn create_local_user(
        _username: &str,
        _full_name: Option<&str>,
        _password: &str,
        _must_change_password: bool,
        _enabled: bool,
    ) -> Result<()> {
        bail!("local Windows user creation is available only on Windows")
    }

    pub fn delete_local_user(_username: &str) -> Result<()> {
        bail!("local Windows user deletion is available only on Windows")
    }

    pub fn set_local_user_enabled(_username: &str, _enabled: bool) -> Result<()> {
        bail!("local Windows account status changes are available only on Windows")
    }

    pub fn reset_local_user_password(_username: &str, _new_password: &str) -> Result<()> {
        bail!("local Windows password reset is available only on Windows")
    }

    pub fn change_own_password(
        _username: &str,
        _current_password: &str,
        _new_password: &str,
    ) -> Result<()> {
        bail!("local Windows password change is available only on Windows")
    }
}

pub fn list_local_users_base() -> Result<Vec<LocalUserSummary>> {
    imp::list_local_users()
}

pub fn create_local_user(
    username: &str,
    full_name: Option<&str>,
    password: &str,
    must_change_password: bool,
    enabled: bool,
) -> Result<()> {
    imp::create_local_user(username, full_name, password, must_change_password, enabled)
}

pub fn delete_local_user(username: &str) -> Result<()> {
    imp::delete_local_user(username)
}

pub fn set_local_user_enabled(username: &str, enabled: bool) -> Result<()> {
    imp::set_local_user_enabled(username, enabled)
}

pub fn reset_local_user_password(username: &str, new_password: &str) -> Result<()> {
    imp::reset_local_user_password(username, new_password)
}

pub fn change_own_password(
    username: &str,
    current_password: &str,
    new_password: &str,
) -> Result<()> {
    imp::change_own_password(username, current_password, new_password)
}

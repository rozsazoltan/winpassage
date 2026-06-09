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

    const NERR_SUCCESS: u32 = 0;
    const ERROR_MORE_DATA: u32 = 234;
    const MAX_PREFERRED_LENGTH: u32 = u32::MAX;
    const ADMINISTRATORS_SID: &str = "S-1-5-32-544";

    #[repr(C)]
    struct LocalGroupMembersInfo3 {
        lgrmi3_domainandname: *mut u16,
    }

    #[link(name = "Netapi32")]
    extern "system" {
        fn NetLocalGroupGetMembers(
            servername: *const u16,
            localgroupname: *const u16,
            level: u32,
            bufptr: *mut *mut u8,
            prefmaxlen: u32,
            entriesread: *mut u32,
            totalentries: *mut u32,
            resumehandle: *mut u32,
        ) -> u32;

        fn NetLocalGroupAddMembers(
            servername: *const u16,
            groupname: *const u16,
            level: u32,
            buf: *mut u8,
            totalentries: u32,
        ) -> u32;

        fn NetLocalGroupDelMembers(
            servername: *const u16,
            groupname: *const u16,
            level: u32,
            buf: *mut u8,
            totalentries: u32,
        ) -> u32;

        fn NetApiBufferFree(buffer: *mut c_void) -> u32;
    }

    #[link(name = "Advapi32")]
    extern "system" {
        fn ConvertStringSidToSidW(string_sid: *const u16, sid: *mut *mut c_void) -> i32;
        fn LookupAccountSidW(
            system_name: *const u16,
            sid: *const c_void,
            name: *mut u16,
            cch_name: *mut u32,
            referenced_domain_name: *mut u16,
            cch_referenced_domain_name: *mut u32,
            pe_use: *mut u32,
        ) -> i32;
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn LocalFree(hmem: *mut c_void) -> *mut c_void;
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

    struct LocalHandle(*mut c_void);

    impl Drop for LocalHandle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let _ = LocalFree(self.0);
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

    fn administrators_group_name() -> Result<String> {
        let sid_wide = to_wide(ADMINISTRATORS_SID);
        let mut sid: *mut c_void = null_mut();

        let ok = unsafe { ConvertStringSidToSidW(sid_wide.as_ptr(), &mut sid) };
        if ok == 0 || sid.is_null() {
            return Ok("Administrators".to_string());
        }
        let _sid = LocalHandle(sid);

        let mut name_len = 0u32;
        let mut domain_len = 0u32;
        let mut sid_use = 0u32;

        unsafe {
            let _ = LookupAccountSidW(
                null(),
                sid,
                null_mut(),
                &mut name_len,
                null_mut(),
                &mut domain_len,
                &mut sid_use,
            );
        }

        if name_len == 0 {
            return Ok("Administrators".to_string());
        }

        let mut name = vec![0u16; name_len as usize];
        let mut domain = vec![0u16; domain_len.max(1) as usize];
        let ok = unsafe {
            LookupAccountSidW(
                null(),
                sid,
                name.as_mut_ptr(),
                &mut name_len,
                domain.as_mut_ptr(),
                &mut domain_len,
                &mut sid_use,
            )
        };

        if ok == 0 {
            return Ok("Administrators".to_string());
        }

        Ok(String::from_utf16_lossy(&name[..name_len as usize]))
    }

    fn administrator_members() -> Result<Vec<String>> {
        let group_name = administrators_group_name()?;
        let group_wide = to_wide(&group_name);
        let mut buffer: *mut u8 = null_mut();
        let mut entries_read = 0u32;
        let mut total_entries = 0u32;
        let mut resume_handle = 0u32;
        let mut members = Vec::new();

        loop {
            let status = unsafe {
                NetLocalGroupGetMembers(
                    null(),
                    group_wide.as_ptr(),
                    3,
                    &mut buffer,
                    MAX_PREFERRED_LENGTH,
                    &mut entries_read,
                    &mut total_entries,
                    &mut resume_handle,
                )
            };

            if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
                bail!(net_error("NetLocalGroupGetMembers", status));
            }

            let _owned = NetBuffer(buffer);

            if !buffer.is_null() && entries_read > 0 {
                let slice = unsafe {
                    std::slice::from_raw_parts(
                        buffer.cast::<LocalGroupMembersInfo3>(),
                        entries_read as usize,
                    )
                };

                for item in slice {
                    if let Some(name) = unsafe { from_wide_ptr(item.lgrmi3_domainandname) } {
                        members.push(name);
                    }
                }
            }

            buffer = null_mut();

            if status == NERR_SUCCESS {
                break;
            }
        }

        Ok(members)
    }

    fn member_matches_username(member: &str, username: &str) -> bool {
        member.eq_ignore_ascii_case(username)
            || member
                .rsplit_once('\\')
                .map(|(_, right)| right.eq_ignore_ascii_case(username))
                .unwrap_or(false)
    }

    pub fn is_local_administrator(username: &str) -> Result<bool> {
        Ok(administrator_members()?
            .iter()
            .any(|member| member_matches_username(member, username)))
    }

    pub fn local_administrator_count() -> Result<usize> {
        Ok(administrator_members()?.len())
    }

    pub fn grant_local_administrator(username: &str) -> Result<()> {
        if is_local_administrator(username)? {
            return Ok(());
        }

        let group_name = administrators_group_name()?;
        let group_wide = to_wide(&group_name);
        let mut username_wide = to_wide(username);
        let mut member = LocalGroupMembersInfo3 {
            lgrmi3_domainandname: username_wide.as_mut_ptr(),
        };

        let status = unsafe {
            NetLocalGroupAddMembers(
                null(),
                group_wide.as_ptr(),
                3,
                (&mut member as *mut LocalGroupMembersInfo3).cast::<u8>(),
                1,
            )
        };

        if status != NERR_SUCCESS {
            return Err(net_error("NetLocalGroupAddMembers", status));
        }

        Ok(())
    }

    pub fn revoke_local_administrator(username: &str) -> Result<()> {
        if !is_local_administrator(username)? {
            return Ok(());
        }

        if local_administrator_count()? <= 1 {
            bail!("refusing to remove the last local administrator account");
        }

        let group_name = administrators_group_name()?;
        let group_wide = to_wide(&group_name);
        let mut username_wide = to_wide(username);
        let mut member = LocalGroupMembersInfo3 {
            lgrmi3_domainandname: username_wide.as_mut_ptr(),
        };

        let status = unsafe {
            NetLocalGroupDelMembers(
                null(),
                group_wide.as_ptr(),
                3,
                (&mut member as *mut LocalGroupMembersInfo3).cast::<u8>(),
                1,
            )
        };

        if status != NERR_SUCCESS {
            return Err(net_error("NetLocalGroupDelMembers", status));
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    use anyhow::bail;

    pub fn is_local_administrator(_username: &str) -> Result<bool> {
        bail!("local Windows administrator checks are available only on Windows")
    }

    pub fn local_administrator_count() -> Result<usize> {
        bail!("local Windows administrator checks are available only on Windows")
    }

    pub fn grant_local_administrator(_username: &str) -> Result<()> {
        bail!("local Windows administrator changes are available only on Windows")
    }

    pub fn revoke_local_administrator(_username: &str) -> Result<()> {
        bail!("local Windows administrator changes are available only on Windows")
    }
}

pub fn is_local_administrator(username: &str) -> Result<bool> {
    imp::is_local_administrator(username)
}

pub fn local_administrator_count() -> Result<usize> {
    imp::local_administrator_count()
}

pub fn grant_local_administrator(username: &str) -> Result<()> {
    imp::grant_local_administrator(username)
}

pub fn revoke_local_administrator(username: &str) -> Result<()> {
    imp::revoke_local_administrator(username)
}

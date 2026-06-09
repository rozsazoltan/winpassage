use anyhow::Result;
use winpassage_protocol::{DriveMapping, DriveReconnectResult};

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, bail};
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::{null, null_mut};
    use zeroize::Zeroize;

    const NO_ERROR: u32 = 0;
    const DRIVE_REMOTE: u32 = 4;
    const RESOURCETYPE_DISK: u32 = 0x00000001;
    const CONNECT_UPDATE_PROFILE: u32 = 0x00000001;

    #[repr(C)]
    struct NetResourceW {
        dw_scope: u32,
        dw_type: u32,
        dw_display_type: u32,
        dw_usage: u32,
        lp_local_name: *mut u16,
        lp_remote_name: *mut u16,
        lp_comment: *mut u16,
        lp_provider: *mut u16,
    }

    #[link(name = "Mpr")]
    extern "system" {
        fn WNetAddConnection2W(
            lp_net_resource: *mut NetResourceW,
            lp_password: *const u16,
            lp_user_name: *const u16,
            dw_flags: u32,
        ) -> u32;

        fn WNetCancelConnection2W(lp_name: *const u16, dw_flags: u32, force: i32) -> u32;

        fn WNetGetConnectionW(
            lp_local_name: *const u16,
            lp_remote_name: *mut u16,
            lpn_length: *mut u32,
        ) -> u32;
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn GetDriveTypeW(lp_root_path_name: *const u16) -> u32;
    }

    fn to_wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(once(0)).collect()
    }

    fn normalize_drive_letter(letter: &str) -> String {
        let trimmed = letter.trim().trim_end_matches('\\').trim_end_matches('/');
        if trimmed.ends_with(':') {
            trimmed.to_uppercase()
        } else {
            format!("{}:", trimmed.to_uppercase())
        }
    }

    fn message_for(code: u32) -> String {
        if code == NO_ERROR {
            "ok".to_string()
        } else {
            format!("Win32 network error {code}")
        }
    }

    pub fn list_mapped_drives() -> Result<Vec<DriveMapping>> {
        let mut drives = Vec::new();

        for letter in b'A'..=b'Z' {
            let drive = format!("{}:", letter as char);
            let root = format!("{}\\", drive);
            let root_wide = to_wide(&root);
            let drive_type = unsafe { GetDriveTypeW(root_wide.as_ptr()) };

            if drive_type != DRIVE_REMOTE {
                continue;
            }

            let drive_wide = to_wide(&drive);
            let mut buffer = vec![0u16; 2048];
            let mut len = buffer.len() as u32;
            let status = unsafe {
                WNetGetConnectionW(drive_wide.as_ptr(), buffer.as_mut_ptr(), &mut len)
            };

            if status == NO_ERROR {
                let remote = String::from_utf16_lossy(&buffer[..len as usize])
                    .trim_end_matches('\0')
                    .to_string();
                drives.push(DriveMapping {
                    letter: drive,
                    remote_path: remote,
                });
            }
        }

        Ok(drives)
    }

    pub fn reconnect_mapped_drives(
        username: &str,
        password: &str,
        drives: &[DriveMapping],
    ) -> Result<Vec<DriveReconnectResult>> {
        if drives.is_empty() {
            bail!("no mapped drives were provided");
        }

        let username_wide = to_wide(username);
        let mut password_wide = to_wide(password);
        let mut results = Vec::new();

        for drive in drives {
            let letter = normalize_drive_letter(&drive.letter);
            let mut letter_wide = to_wide(&letter);
            let mut remote_wide = to_wide(&drive.remote_path);

            let _ = unsafe { WNetCancelConnection2W(letter_wide.as_ptr(), CONNECT_UPDATE_PROFILE, 1) };

            let mut resource = NetResourceW {
                dw_scope: 0,
                dw_type: RESOURCETYPE_DISK,
                dw_display_type: 0,
                dw_usage: 0,
                lp_local_name: letter_wide.as_mut_ptr(),
                lp_remote_name: remote_wide.as_mut_ptr(),
                lp_comment: null_mut(),
                lp_provider: null_mut(),
            };

            let status = unsafe {
                WNetAddConnection2W(
                    &mut resource,
                    password_wide.as_ptr(),
                    username_wide.as_ptr(),
                    CONNECT_UPDATE_PROFILE,
                )
            };

            results.push(DriveReconnectResult {
                letter,
                remote_path: drive.remote_path.clone(),
                success: status == NO_ERROR,
                message: message_for(status),
            });
        }

        password_wide.zeroize();

        if results.iter().all(|item| !item.success) {
            return Err(anyhow!("all mapped drive reconnect attempts failed"));
        }

        Ok(results)
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    use anyhow::bail;

    pub fn list_mapped_drives() -> Result<Vec<DriveMapping>> {
        bail!("mapped drive enumeration is available only on Windows")
    }

    pub fn reconnect_mapped_drives(
        _username: &str,
        _password: &str,
        _drives: &[DriveMapping],
    ) -> Result<Vec<DriveReconnectResult>> {
        bail!("mapped drive reconnect is available only on Windows")
    }
}

pub fn list_mapped_drives() -> Result<Vec<DriveMapping>> {
    imp::list_mapped_drives()
}

pub fn reconnect_mapped_drives(
    username: &str,
    password: &str,
    drives: &[DriveMapping],
) -> Result<Vec<DriveReconnectResult>> {
    imp::reconnect_mapped_drives(username, password, drives)
}

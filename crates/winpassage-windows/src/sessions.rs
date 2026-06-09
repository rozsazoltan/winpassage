use anyhow::Result;
use winpassage_protocol::LocalSessionSummary;

#[cfg(windows)]
mod imp {
    use super::*;
    use anyhow::{anyhow, bail};
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    const WTS_CURRENT_SERVER_HANDLE: *mut c_void = null_mut();
    const WTS_USER_NAME: u32 = 5;
    const WTS_DOMAIN_NAME: u32 = 7;
    const WTS_CLIENT_NAME: u32 = 10;
    const WTS_CURRENT_SESSION: u32 = 0xFFFF_FFFF;

    #[repr(C)]
    struct WtsSessionInfoW {
        session_id: u32,
        win_station_name: *mut u16,
        state: u32,
    }

    #[link(name = "Wtsapi32")]
    extern "system" {
        fn WTSEnumerateSessionsW(
            h_server: *mut c_void,
            reserved: u32,
            version: u32,
            pp_session_info: *mut *mut WtsSessionInfoW,
            p_count: *mut u32,
        ) -> i32;

        fn WTSQuerySessionInformationW(
            h_server: *mut c_void,
            session_id: u32,
            wts_info_class: u32,
            pp_buffer: *mut *mut u16,
            p_bytes_returned: *mut u32,
        ) -> i32;

        fn WTSLogoffSession(h_server: *mut c_void, session_id: u32, wait: i32) -> i32;
        fn WTSFreeMemory(p_memory: *mut c_void);
    }

    struct WtsBuffer<T>(*mut T);

    impl<T> Drop for WtsBuffer<T> {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { WTSFreeMemory(self.0.cast::<c_void>()) }
            }
        }
    }

    unsafe fn from_wide_ptr(ptr: *const u16) -> Option<String> {
        if ptr.is_null() {
            return None;
        }

        let mut len = 0usize;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        if len == 0 {
            return None;
        }

        Some(String::from_utf16_lossy(std::slice::from_raw_parts(
            ptr, len,
        )))
    }

    fn query_string(session_id: u32, info_class: u32) -> Result<Option<String>> {
        let mut buffer: *mut u16 = null_mut();
        let mut bytes_returned = 0u32;
        let ok = unsafe {
            WTSQuerySessionInformationW(
                WTS_CURRENT_SERVER_HANDLE,
                session_id,
                info_class,
                &mut buffer,
                &mut bytes_returned,
            )
        };

        if ok == 0 {
            return Ok(None);
        }

        let _owned = WtsBuffer(buffer);
        Ok(unsafe { from_wide_ptr(buffer) })
    }

    fn state_label(state: u32) -> String {
        match state {
            0 => "active",
            1 => "connected",
            2 => "connect_query",
            3 => "shadow",
            4 => "disconnected",
            5 => "idle",
            6 => "listen",
            7 => "reset",
            8 => "down",
            9 => "init",
            _ => "unknown",
        }
        .to_string()
    }

    pub fn list_sessions() -> Result<Vec<LocalSessionSummary>> {
        let mut session_info: *mut WtsSessionInfoW = null_mut();
        let mut count = 0u32;
        let ok = unsafe {
            WTSEnumerateSessionsW(
                WTS_CURRENT_SERVER_HANDLE,
                0,
                1,
                &mut session_info,
                &mut count,
            )
        };

        if ok == 0 {
            bail!("WTSEnumerateSessionsW failed");
        }

        let _owned = WtsBuffer(session_info);
        let sessions = unsafe { std::slice::from_raw_parts(session_info, count as usize) };

        let mut output = Vec::new();
        for session in sessions {
            let username = query_string(session.session_id, WTS_USER_NAME)?;
            let domain = query_string(session.session_id, WTS_DOMAIN_NAME)?;
            let client_name = query_string(session.session_id, WTS_CLIENT_NAME)?;

            output.push(LocalSessionSummary {
                session_id: session.session_id,
                username,
                domain,
                state: state_label(session.state),
                client_name,
                is_console: session.session_id == 1,
            });
        }

        Ok(output)
    }

    pub fn active_session_count_for_user(username: &str) -> Result<usize> {
        Ok(list_sessions()?
            .into_iter()
            .filter(|session| {
                session
                    .username
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(username))
            })
            .filter(|session| {
                matches!(
                    session.state.as_str(),
                    "active" | "connected" | "disconnected"
                )
            })
            .count())
    }

    pub fn logoff_session(session_id: u32) -> Result<()> {
        if session_id == WTS_CURRENT_SESSION {
            bail!("refusing to log off the current service session")
        }

        let ok = unsafe { WTSLogoffSession(WTS_CURRENT_SERVER_HANDLE, session_id, 0) };
        if ok == 0 {
            return Err(anyhow!("WTSLogoffSession failed for session {session_id}"));
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use super::*;
    use anyhow::bail;

    pub fn list_sessions() -> Result<Vec<LocalSessionSummary>> {
        bail!("local Windows session enumeration is available only on Windows")
    }

    pub fn active_session_count_for_user(_username: &str) -> Result<usize> {
        bail!("local Windows session enumeration is available only on Windows")
    }

    pub fn logoff_session(_session_id: u32) -> Result<()> {
        bail!("local Windows session logoff is available only on Windows")
    }
}

pub fn list_sessions() -> Result<Vec<LocalSessionSummary>> {
    imp::list_sessions()
}

pub fn active_session_count_for_user(username: &str) -> Result<usize> {
    imp::active_session_count_for_user(username)
}

pub fn logoff_session(session_id: u32) -> Result<()> {
    imp::logoff_session(session_id)
}

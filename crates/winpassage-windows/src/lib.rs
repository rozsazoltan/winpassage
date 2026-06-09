pub mod local_groups;
pub mod local_users;
pub mod mapped_drives;
pub mod sessions;

use anyhow::Result;
use winpassage_protocol::LocalUserSummary;

pub use local_groups::{
    grant_local_administrator, is_local_administrator, local_administrator_count,
    revoke_local_administrator,
};
pub use local_users::{
    change_own_password, create_local_user, delete_local_user, reset_local_user_password,
    set_local_user_enabled,
};
pub use mapped_drives::{list_mapped_drives, reconnect_mapped_drives};
pub use sessions::{active_session_count_for_user, list_sessions, logoff_session};

pub fn list_local_users() -> Result<Vec<LocalUserSummary>> {
    let mut users = local_users::list_local_users_base()?;

    for user in &mut users {
        user.is_administrator =
            local_groups::is_local_administrator(&user.username).unwrap_or(false);
        user.active_session_count =
            sessions::active_session_count_for_user(&user.username).unwrap_or(0);
    }

    Ok(users)
}

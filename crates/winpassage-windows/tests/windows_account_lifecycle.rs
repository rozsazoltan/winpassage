#![cfg(windows)]

use std::time::{SystemTime, UNIX_EPOCH};

fn destructive_tests_enabled() -> bool {
    std::env::var("WINPASSAGE_ENABLE_DESTRUCTIVE_WINDOWS_TESTS").as_deref() == Ok("1")
}

fn test_username() -> String {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_secs();

    format!("wp-ci-{suffix}")
}

fn cleanup_user(username: &str) {
    let _ = winpassage_windows::revoke_local_administrator(username);
    let _ = winpassage_windows::delete_local_user(username);
}

#[test]
#[ignore = "creates and deletes a real local Windows user"]
fn local_account_lifecycle_works_on_windows() {
    assert!(
        destructive_tests_enabled(),
        "set WINPASSAGE_ENABLE_DESTRUCTIVE_WINDOWS_TESTS=1 to run this destructive test"
    );

    let username = test_username();
    let initial_password = "WpInitialPassword123!";
    let reset_password = "WpResetPassword123!";
    let changed_password = "WpChangedPassword123!";

    cleanup_user(&username);

    winpassage_windows::create_local_user(
        &username,
        Some("WinPassage CI Test User"),
        initial_password,
        true,
        true,
    )
    .expect("test user should be created");

    let users = winpassage_windows::list_local_users().expect("users should be listed");
    assert!(users.iter().any(|user| user.username.eq_ignore_ascii_case(&username)));

    winpassage_windows::set_local_user_enabled(&username, false)
        .expect("test user should be disabled");
    winpassage_windows::set_local_user_enabled(&username, true)
        .expect("test user should be enabled again");

    winpassage_windows::reset_local_user_password(&username, reset_password)
        .expect("admin password reset should work");
    winpassage_windows::change_own_password(&username, reset_password, changed_password)
        .expect("self-service password change should accept the current password");

    winpassage_windows::grant_local_administrator(&username)
        .expect("test user should be grantable as administrator");
    assert!(
        winpassage_windows::is_local_administrator(&username)
            .expect("administrator membership should be queryable")
    );
    winpassage_windows::revoke_local_administrator(&username)
        .expect("test user administrator membership should be revokable");
    assert!(
        !winpassage_windows::is_local_administrator(&username)
            .expect("administrator membership should be queryable after revoke")
    );

    winpassage_windows::delete_local_user(&username).expect("test user should be deleted");

    let users = winpassage_windows::list_local_users()
        .expect("users should be listed after delete");
    assert!(!users.iter().any(|user| user.username.eq_ignore_ascii_case(&username)));
}

#[test]
fn non_destructive_windows_observability_works() {
    let users = winpassage_windows::list_local_users().expect("local users should be listable");
    assert!(!users.is_empty(), "a Windows machine should have at least one local user account");

    let _sessions = winpassage_windows::list_sessions().expect("sessions should be listable");
    let admin_count = winpassage_windows::local_administrator_count()
        .expect("local administrator membership should be countable");
    assert!(admin_count >= 1, "there should be at least one local administrator");
}

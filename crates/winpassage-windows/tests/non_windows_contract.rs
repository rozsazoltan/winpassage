#![cfg(not(windows))]

#[test]
fn windows_only_operations_fail_explicitly_on_non_windows() {
    let error =
        winpassage_windows::list_local_users().expect_err("listing users should be Windows-only");
    assert!(error.to_string().contains("available only on Windows"));

    let error =
        winpassage_windows::list_sessions().expect_err("listing sessions should be Windows-only");
    assert!(error.to_string().contains("available only on Windows"));

    let error = winpassage_windows::list_mapped_drives()
        .expect_err("listing drives should be Windows-only");
    assert!(error.to_string().contains("available only on Windows"));
}

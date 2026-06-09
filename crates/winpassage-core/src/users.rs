use crate::errors::{CoreError, CoreResult};

pub fn validate_local_username(username: &str) -> CoreResult<()> {
    let trimmed = username.trim();

    if trimmed.is_empty() {
        return Err(CoreError::UsernameRequired);
    }

    let unsupported = [
        '/', '\\', '[', ']', ':', ';', '|', '=', ',', '+', '*', '?', '<', '>', '"',
    ];

    if trimmed
        .chars()
        .any(|ch| unsupported.contains(&ch) || ch.is_control())
    {
        return Err(CoreError::UsernameContainsUnsupportedCharacters);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_local_username;

    #[test]
    fn accepts_common_local_username() {
        assert!(validate_local_username("julia").is_ok());
    }

    #[test]
    fn rejects_empty_username() {
        assert!(validate_local_username("   ").is_err());
    }

    #[test]
    fn rejects_path_like_username() {
        assert!(validate_local_username("domain\\julia").is_err());
    }
}

#[cfg(test)]
mod strict_tests {
    use super::validate_local_username;
    use crate::CoreError;

    #[test]
    fn accepts_usernames_with_common_safe_separators() {
        assert!(validate_local_username("julia.nagy").is_ok());
        assert!(validate_local_username("julia-nagy").is_ok());
        assert!(validate_local_username("julia_nagy").is_ok());
    }

    #[test]
    fn rejects_each_unsupported_character() {
        for ch in ['/', '\\', '[', ']', ':', ';', '|', '=', ',', '+', '*', '?', '<', '>', '"'] {
            let username = format!("julia{ch}nagy");
            assert!(matches!(
                validate_local_username(&username),
                Err(CoreError::UsernameContainsUnsupportedCharacters)
            ));
        }
    }

    #[test]
    fn rejects_control_characters() {
        assert!(matches!(
            validate_local_username("julia\nnagy"),
            Err(CoreError::UsernameContainsUnsupportedCharacters)
        ));
    }
}

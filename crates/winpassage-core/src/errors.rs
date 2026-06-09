use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("username is required")]
    UsernameRequired,

    #[error("username contains unsupported characters")]
    UsernameContainsUnsupportedCharacters,

    #[error("password is too short; expected at least {min} characters")]
    PasswordTooShort { min: usize },

    #[error("password must contain at least one uppercase letter")]
    PasswordMissingUppercase,

    #[error("password must contain at least one lowercase letter")]
    PasswordMissingLowercase,

    #[error("password must contain at least one number")]
    PasswordMissingNumber,

    #[error("password must contain at least one symbol")]
    PasswordMissingSymbol,
}

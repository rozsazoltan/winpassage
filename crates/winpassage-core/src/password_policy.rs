use crate::errors::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_symbol: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_symbol: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordValidation {
    pub accepted: bool,
}

impl PasswordPolicy {
    pub fn validate(&self, password: &str) -> CoreResult<PasswordValidation> {
        if password.chars().count() < self.min_length {
            return Err(CoreError::PasswordTooShort {
                min: self.min_length,
            });
        }

        if self.require_uppercase && !password.chars().any(|ch| ch.is_uppercase()) {
            return Err(CoreError::PasswordMissingUppercase);
        }

        if self.require_lowercase && !password.chars().any(|ch| ch.is_lowercase()) {
            return Err(CoreError::PasswordMissingLowercase);
        }

        if self.require_number && !password.chars().any(|ch| ch.is_numeric()) {
            return Err(CoreError::PasswordMissingNumber);
        }

        if self.require_symbol
            && !password
                .chars()
                .any(|ch| !ch.is_alphanumeric() && !ch.is_whitespace())
        {
            return Err(CoreError::PasswordMissingSymbol);
        }

        Ok(PasswordValidation { accepted: true })
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordPolicy;

    #[test]
    fn accepts_strong_password() {
        assert!(PasswordPolicy::default()
            .validate("LongPassword123!")
            .is_ok());
    }

    #[test]
    fn rejects_short_password() {
        assert!(PasswordPolicy::default().validate("Aa1!").is_err());
    }
}

use crate::core::AppError;

/// Password Policy Configuration
#[derive(Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false, // Optional by default
        }
    }
}

impl PasswordPolicy {
    /// Strict policy for admin accounts
    #[allow(dead_code)]
    pub fn strict() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }

    /// Validate password against policy
    pub fn validate(&self, password: &str) -> Result<(), AppError> {
        let mut errors: Vec<String> = Vec::new();

        // Check minimum length
        if password.len() < self.min_length {
            errors.push(format!("Password must be at least {} characters", self.min_length));
        }

        // Check uppercase
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        // Check lowercase
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        // Check digit
        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            errors.push("Password must contain at least one number".to_string());
        }

        // Check special character
        if self.require_special && !password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?".contains(c)) {
            errors.push("Password must contain at least one special character".to_string());
        }

        // Check common passwords
        if is_common_password(password) {
            errors.push("Password is too common, please choose a stronger password".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError {
                code: 400,
                message: errors.join("; "),
            })
        }
    }
}

/// Check if password is in a list of common passwords
fn is_common_password(password: &str) -> bool {
    const COMMON_PASSWORDS: &[&str] = &[
        "password", "123456", "12345678", "qwerty", "abc123",
        "password123", "admin", "letmein", "welcome", "monkey",
        "dragon", "master", "login", "passw0rd", "hello",
        "shadow", "sunshine", "princess", "football", "baseball",
        "1234567890", "password1", "admin123", "root", "toor",
    ];
    
    let lower = password.to_lowercase();
    COMMON_PASSWORDS.iter().any(|&common| lower == common)
}

/// Validate password with default policy
pub fn validate_password(password: &str) -> Result<(), AppError> {
    PasswordPolicy::default().validate(password)
}

/// Validate password with strict policy (for admin setup)
#[allow(dead_code)]
pub fn validate_password_strict(password: &str) -> Result<(), AppError> {
    PasswordPolicy::strict().validate(password)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        assert!(validate_password("Admin@123456").is_ok());
    }

    #[test]
    fn test_too_short() {
        assert!(validate_password("Ab1").is_err());
    }

    #[test]
    fn test_no_uppercase() {
        assert!(validate_password("password123").is_err());
    }

    #[test]
    fn test_common_password() {
        assert!(validate_password("Password123").is_err()); // "password" is common
    }
}

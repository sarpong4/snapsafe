use std::io;

use argon2::{
    password_hash::{rand_core::OsRng, 
        PasswordHasher, PasswordVerifier, PasswordHash, SaltString}, 
    Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// struct `Password` will hold a hash: hashed password in phc string form generated from zeroized user input.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Password {
    pub hash: String,
}

impl Default for Password {
    fn default() -> Self {
        Self {
            hash: "<unset>".into()
        }
    }
}

impl Password {
    /// Given the password a user enters/provides, return a `Password` with a hashed string in phc string format
    ///
    /// If it fails, return `PasswordError::HashError`
    pub fn new(input: String, policy: &PasswordPolicy) -> Result<Self, PasswordError> {
        policy.validate(&input)?;
        let password = Zeroizing::new(input);
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::DEFAULT);
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

        Ok(Self{ hash: password_hash })
    }

    /// Compare the current hash from Self with the users input and verify if the password is correct.
    pub fn verify(&self, input: &str) -> Result<bool, PasswordError> {
        
        let parsed_hash = PasswordHash::new(&self.hash)?;
        Ok(Argon2::default().verify_password(&input.as_bytes(), &parsed_hash).is_ok())
    }
}

/// struct `PasswordPolicy` will deal with implementing secure password policies and validating against user provided passwords.
///
/// `Password Policy`: password needs to be between 8 to 16 characters.
/// It should contain both uppercase and lowercase alphabets.
/// It should have at least 1 number.
/// It should have at least 1 symbol.
pub struct PasswordPolicy {
    min_length: usize,
    max_length: usize,
    require_lowercase: bool,
    require_uppercase: bool,
    require_symbol: bool,
    require_digit: bool,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 16,
            require_lowercase: true,
            require_uppercase: true,
            require_digit: true,
            require_symbol: true
        }
    }
}

impl PasswordPolicy {
    /// Build a new `PasswordPolicy` instance
    pub fn new(min: usize, max: usize, lowercase: bool, uppercase: bool, digit: bool, symbol: bool) -> Self {
        Self {
            min_length: min,
            max_length: max,
            require_uppercase: uppercase,
            require_lowercase: lowercase,
            require_digit: digit,
            require_symbol: symbol,
        }
    }

    /// Validate user's entered password against password policy definitions
    pub fn validate(&self, pwd: &str) -> Result<(), PasswordError> {
        let len = pwd.len();
        let is_valid_length = len >= self.min_length && len <= self.max_length;

        let has_lowercase = if self.require_lowercase {
            pwd.chars().any(|c| c.is_ascii_lowercase())
        } else {
            true
        };

        let has_uppercase = if self.require_uppercase {
            pwd.chars().any(|c| c.is_ascii_uppercase())
        }else {
            true
        };

        let has_digit = if self.require_digit {
            pwd.chars().any(|c| c.is_ascii_digit())
        }else {
            true
        };

        let has_symbol = if self.require_symbol {
            pwd.chars().any(|c| !c.is_ascii_alphanumeric() && c.is_ascii_graphic())
        } else {
            true
        };

        if has_lowercase && has_digit && has_symbol && has_uppercase && is_valid_length {
            return Ok(())
        }

        let message = self.generate_policy();
        Err(PasswordError::InvalidFormat(format!("InvalidFormat: {message}")))
    }

    /// Generate a String format of the current `PasswordPolicy` instance
    pub fn generate_policy(&self) -> String {
        let case_message = if self.require_lowercase && self.require_uppercase {
            "It needs both uppercase and lowercase letters"
        }
        else if self.require_lowercase {
            "It needs only lowercase letters"
        }
        else {
            "It needs only uppercase letters"
        };

        let symbol = if self.require_symbol {
            "It needs at least 1 symbol"
        }else {
            ""
        };

        let digit = if self.require_digit {
            "It needs at least 1 digit"
        }else {
            ""
        };

        let message = format!(
            "Your password must be between {} to {} characters\n{}\n{}\n{}",
            self.min_length, self.max_length, case_message, symbol, digit
        );

        message
    }
}

/// enum `PasswordError` will represent the various errors generated related to passwords. 
#[derive(Debug)]
pub enum PasswordError {
    IncorrectPassword,
    InvalidFormat(String),
    HashError(argon2::password_hash::Error),
    InputError(io::Error),
}

impl From<argon2::password_hash::Error> for PasswordError {
    fn from(err: argon2::password_hash::Error) -> Self {
        PasswordError::HashError(err)
    }
}

impl From<io::Error> for PasswordError{
    fn from(err: io::Error) -> Self {
        PasswordError::InputError(err)
    }
}

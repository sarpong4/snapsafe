use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use argon2::Argon2;

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    println!("Deriving the key...");
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt, &mut key).unwrap();
    key
} // derive encrytption key from password

pub fn encrypt_file_bytes(data: &[u8], key: &[u8]) -> (Vec<u8>, [u8; 12]) {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, data).unwrap();
    (ciphertext, nonce_bytes)
}

pub fn decrypt_file_bytes(ciphertext: &[u8], key: &[u8], nonce_bytes: &[u8; 12]) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_consistency() {
        let password = "my-password";
        let salt = b"fixed-salt";
        let key1 = derive_key(password, salt);
        let key2 = derive_key(password, salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_key_derivation_changes() {
        let password1 = "password1";
        let password2 = "password2";
        let salt = b"fixed-salt";
        let key1 = derive_key(password1, salt);
        let key2 = derive_key(password2, salt);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let data = b"important data";
        let password = "backup123";
        let salt = b"some-salt";

        let key = derive_key(password, salt);
        let (encrypted, nonce) = encrypt_file_bytes(data, &key);
        let decrypted = decrypt_file_bytes(&encrypted, &key, &nonce);
        assert_eq!(decrypted, data);
    }

    #[test]
    #[should_panic]
    fn test_wrong_key_panics() {
        let data = b"secret";
        let key1 = derive_key("pass1", b"salt");
        let key2 = derive_key("pass2", b"salt");

        let (encrypted, nonce) = encrypt_file_bytes(data, &key1);
        let _ = decrypt_file_bytes(&encrypted, &key2, &nonce); // should panic
    }
}
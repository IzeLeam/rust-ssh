pub mod crypto {
    use argon2::{Argon2, PasswordHasher, PasswordVerifier};
    use argon2::password_hash::SaltString;

    pub fn hash_password(password: &str) -> String {
        let argon2 = Argon2::default();
        let salt = rand::random::<[u8; 16]>();
        let salt_string = SaltString::encode_b64(&salt).expect("Failed to encode salt");
        let key_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .expect("Failed to hash password");
        key_hash.to_string()
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        let argon2 = Argon2::default();
        let parsed_hash = argon2::password_hash::PasswordHash::new(hash).expect("Failed to parse hash");
        argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }

    pub fn encrypt(data: &[u8], _key: &[u8]) -> Vec<u8> {
        // Placeholder for encryption logic
        data.to_vec()
    }

    pub fn decrypt(data: &[u8], _key: &[u8]) -> Vec<u8> {
        // Placeholder for decryption logic
        data.to_vec()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_hash_password() {
            let password = "password123";
            let hash = hash_password(password);
            assert!(verify_password(password, &hash));
        }
    }
}

pub mod protocol {
    use serde::{Serialize, Deserialize};

    use crate::auth::AuthMethod;

    pub enum ClientState {
        Authentication(i32),
        Connected
    }

    #[derive(Serialize, Deserialize, PartialEq)]
    pub enum TypedMessage {
        Command { command: String },
        CommandResponse { response: String, success: bool },
        TabComplete { stdin: String },
        TabCompleteResponse { completions: Vec<String> },
        Auth { auth_method: AuthMethod, username: String, secret: String },
        AuthResponse { success: bool },
    }
}

pub mod network {
    pub const SERVER_PORT: u16 = 9999;
    pub const SERVER_ADDRESS: &str = "127.0.0.1";

    pub fn get_address() -> String {
        format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)
    }
}

pub mod auth {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub enum AuthMethod {
        Password,
        Certificate,
    }
}

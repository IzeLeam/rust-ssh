pub mod crypto {
    use argon2::{Argon2, PasswordHasher, PasswordVerifier};
    use argon2::password_hash::SaltString;

    /// Hash a password using Argon2
    ///
    /// # Arguments
    /// * `password` - The password to hash
    /// # Returns
    /// * The hashed password as a string
    pub fn hash_password(password: &str) -> String {
        let argon2 = Argon2::default();
        let salt = rand::random::<[u8; 16]>();
        let salt_string = SaltString::encode_b64(&salt).expect("Failed to encode salt");
        let key_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .expect("Failed to hash password");
        key_hash.to_string()
    }

    /// Verify a password if it matches the hash
    ///
    /// # Arguments
    /// * `password` - The password to verify
    /// * `hash` - The hash to verify against
    /// # Returns
    /// * `true` if the password matches the hash, `false` otherwise
    pub fn verify_password(password: &str, hash: &str) -> bool {
        let argon2 = Argon2::default();
        let parsed_hash = argon2::password_hash::PasswordHash::new(hash).expect("Failed to parse hash");
        argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
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

    pub enum ClientState {
        Authentication(i32),
        Connected
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub enum AuthMethod {
        Password,
        Certificate, // Not implemented yet
    }

    /// Both server and client have their own message types
    /// The response types are only used by the server
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
    // Constants for the server
    pub const SERVER_PORT: u16 = 9999;
    pub const SERVER_ADDRESS: &str = "127.0.0.1";

    pub fn get_address() -> String {
        format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)
    }
}
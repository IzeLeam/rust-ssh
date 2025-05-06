// Module pour le protocole
pub mod protocol {
    use serde::{Serialize, Deserialize};

    use crate::auth::AuthMethod;

    #[derive(Serialize, Deserialize, PartialEq)]
    pub enum TypedMessage {
        Command(Command),
        CommandResponse(CommandResponse),
        TabComplete(String),
        TabCompleteResponse(Vec<String>),
        Auth(Auth),
        AuthResponse(bool),
    }

    #[derive(Serialize, Deserialize, PartialEq)]
    pub struct Command {
        pub command: String,
    }

    #[derive(Serialize, Deserialize, PartialEq)]
    pub struct CommandResponse {
        pub response: String,
        pub return_code: bool,
    }

    #[derive(Serialize, Deserialize, PartialEq)]
    pub struct Auth {
        pub auth_method: AuthMethod,
        pub username: String,
        pub secret: String,
    }

    impl TypedMessage {
        pub fn new_command(command: String) -> Self {
            TypedMessage::Command(Command { command })
        }

        pub fn new_auth(auth_method: AuthMethod, username: String, secret: String) -> Self {
            TypedMessage::Auth(Auth { auth_method, username, secret })
        }
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
    
    pub trait Authenticator {
        fn authenticate(&self, secret: &str) -> bool;
    }

    pub struct PasswordAuth {
        pub password: String,
    }

    pub struct CertificateAuth {
        pub pubkey: String,
    }

    impl Authenticator for PasswordAuth {
        fn authenticate(&self, secret: &str) -> bool {
            &self.password == secret
        }
    }

    impl Authenticator for CertificateAuth {
        fn authenticate(&self, secret: &str) -> bool {
            &self.pubkey == secret
        }
    }
}

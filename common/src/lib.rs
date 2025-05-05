// Module pour le protocole
pub mod protocol {
    use serde::{Serialize, Deserialize};

    use crate::auth::AuthType;

    /// Affiche la version du protocole utilisé.
    pub fn print_protocol_version() {
        println!("Protocole d'administration à distance, version 0.1.0");
    }

    #[derive(Serialize, Deserialize)]
    pub enum TypedMessage {
        Command(Command),
        Auth(Auth), 
    }

    #[derive(Serialize, Deserialize)]
    pub struct Command {
        command: String,
    }

    #[derive(Serialize, Deserialize)]
    pub enum AuthMethod {
        Password,
        Certificate,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Auth {
        auth_method: AuthType,
        secret: Vec<u8>,
    }
}

// Module pour la gestion des connexions et sécurisation (exemple basique)
pub mod network {
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use std::io;

    pub const SERVER_PORT: u16 = 9999;
    pub const SERVER_ADDRESS: &str = "127.0.0.1";

    /// Envoi une donnée sur une connexion TCP.
    pub async fn send_data(mut stream: TcpStream, data: &str) -> io::Result<()> {
        let data_bytes = data.as_bytes();
        stream.write_all(data_bytes).await?;
        Ok(())
    }

    /// Réception des données depuis une connexion TCP.
    pub async fn receive_data(mut stream: TcpStream) -> io::Result<String> {
        let mut buffer = [0; 512];
        let size = stream.read(&mut buffer).await?;
        Ok(String::from_utf8_lossy(&buffer[..size]).to_string())
    }
}

pub mod auth {
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    pub enum AuthType {
        Password,
        Certificate,
    }
    
    pub trait Authenticator {
        fn authenticate(&self, identifiant: &str, secret: &str) -> bool;
    }

    pub struct PasswordAuth {
        pub password: String,
    }

    pub struct CertificateAuth {
        pub key: String,
    }

    impl Authenticator for PasswordAuth {
        fn authenticate(&self, _identifiant: &str, secret: &str) -> bool {
            &self.password == secret
        }
    }
}

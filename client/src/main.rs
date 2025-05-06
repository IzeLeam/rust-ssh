use std::path::PathBuf;
use common::network::get_address;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use clap::Parser;
use rpassword::prompt_password;
use std::io::Write;

use common::protocol::AuthMethod;
use common::protocol::TypedMessage;
use common::protocol::ClientState;

use rustls::ClientConfig;
use rustls::RootCertStore;
use tokio_rustls::TlsConnector;
use std::fs::File;
use rustls_pemfile;
use std::sync::Arc;

use rustls::ServerName;
use rustls::Certificate;
use std::convert::TryFrom;
use std::io::BufReader as StdBufReader;
use rustls_pemfile as pem;

/// Command parsing
#[derive(Parser)]
#[clap(name = "Rust SSH", version = "1.0",
    about = "A simple SSH client",
    long_about = "Usage: client username [options]")]
pub struct Cli {
    /// Username
    #[clap(help = "Username to connect to the server")]
    username: String,
    /// Public key (not implemented yet)
    #[clap(short = 'i', help = "Public key to connect to the server")]
    pubkey: Option<PathBuf>,
}

/// Load the root certificate from a file
/// 
/// # Arguments
/// * `p` - The path to the root certificate file
/// # Returns
/// * `Ok(RootCertStore)` if the root certificate was loaded successfully
/// * `Err(Box<dyn std::error::Error>)` if there was an error loading the root certificate
fn load_root(p: &str) -> Result<RootCertStore, Box<dyn std::error::Error>> {
    let mut s = RootCertStore::empty();
    let mut r = StdBufReader::new(File::open(p)?);
    for cert_result in pem::certs(&mut r) {
        match cert_result {
            Ok(cert) => { let _ = s.add(&Certificate(cert.as_ref().to_vec())); },
            Err(e) => return Err(Box::new(e)),
        }
    }
    Ok(s)
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Get the root certificates and build the client config
    let roots = load_root("../certs/root-ca.pem").unwrap();
    let cfg = ClientConfig::builder().with_safe_defaults().with_root_certificates(roots).with_no_client_auth();

    // Initialize the TCP stream and TLS connector
    let tcp = TcpStream::connect(get_address()).await.expect("Erreur de connexion au serveur");
    let mut stream=TlsConnector::from(Arc::new(cfg)).connect(ServerName::try_from("localhost").unwrap(), tcp).await.unwrap();

    let mut stdin_buffer = String::new();
    let mut buffer: [u8; 1024] = [0; 1024];

    let mut state: ClientState = ClientState::Authentication(0);

    // Main loop of :
    // Client -> Server (Wait for stdin input)
    // Server -> Client (Wait for server response)
    loop {
        // Client command
        stdin_buffer.clear();
        match state {
            ClientState::Authentication(_) => {
                // Authentication
                match cli.pubkey {
                    Some(_path) => {
                        println!("Not implemented yet");
                        return;
                    },
                    None => {
                        // Prompt the password and senf an authentication request to the server
                        let password = prompt_password("Enter your passsword: ").unwrap();
                        let message_type = TypedMessage::Auth 
                        { auth_method: AuthMethod::Password, username: cli.username.clone(), secret: password };
                        let serialized_message = serde_json::to_string(&message_type).unwrap();
                        stream.write_all(serialized_message.as_bytes()).await.expect("Erreur d'écriture dans le serveur");
                    }
                }
            }
            // If the client is connected, all is sent is commands (Tab completion is not implemented yet)
            ClientState::Connected => {
                print!("{} > ", cli.username);
                std::io::stdout().flush().unwrap();
                std::io::stdin().read_line(&mut stdin_buffer).expect("Erreur de lecture de la ligne");

                if stdin_buffer.is_empty() || stdin_buffer.trim().is_empty() {
                    continue;
                }
                // Send the command to the server
                let command_str = stdin_buffer.trim();
                let message = TypedMessage::Command { command: command_str.to_string() };
                let serialized_message = serde_json::to_string(&message).unwrap();
                stream.write_all(serialized_message.as_bytes()).await.expect("Erreur d'écriture dans le serveur");
            }
        }

        // Server response
        buffer.fill(0);
        // Handle the deconnection
        let size = match stream.read(&mut buffer).await {
            Ok(size) => {
                if size == 0 {
                    println!("Disconnected");
                    return;
                }
                size
            }
            Err(_) => {
                println!("Disconnected");
                return;
            }
        };

        // Deserialize the message
        let serialized_message: TypedMessage = match serde_json::from_slice(&buffer[..size]) {
            Ok(msg) => msg,
            Err(e) => {
                println!("Failed to parse message: {}", e);
                continue;
            }
        };

        match state {
            ClientState::Authentication(attemps) => {
                match serialized_message {
                    // In the authentication process, the user have 3 attempts to provide the right password
                    TypedMessage::AuthResponse { success } => {
                        if success {
                            println!("Authenticated");
                            state = ClientState::Connected;
                        } else {
                            println!("Authentication failed");
                            state = ClientState::Authentication(attemps + 1);
                            if attemps == 2 {
                                println!("Too many attempts");
                                return;
                            }
                        }
                    }
                    _ => {
                        println!("Unexpected message type");
                        return;
                    }
                }
            }
            ClientState::Connected => {
                match serialized_message {
                    // The response to a command with the return code
                    TypedMessage::CommandResponse { response, success } => {
                        if success {
                            println!("{}", response);
                        } else {
                            println!("Error: {}", response);
                        }
                    },
                    // Tab completion is not implemented yet
                    TypedMessage::TabCompleteResponse { completions} => {
                        println!("Completions: {:?}", completions);
                    },
                    _ => {
                        println!("Unexpected message type");
                        return;
                    }
                }
            }
        }
    }
}

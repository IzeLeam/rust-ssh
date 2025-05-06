use common::{crypto::{hash_password, verify_password}, network::{self, get_address}, protocol::TypedMessage};
use common::protocol::AuthMethod;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    spawn,
};
use std::sync::Arc;
use std::io::{Error, ErrorKind};

use tokio_rustls::{server::TlsStream, TlsAcceptor};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};
use std::{fs::File, io::BufReader};

mod filesys;
use crate::filesys::filesys::*;

mod database;
use crate::database::database::Database;

/// Authenticate the user with password
/// 
/// # Arguments
/// * `auth_method` - The authentication method to use
/// * `username` - The username of the user
/// * `secret` - The password to use
/// * `db` - The database of all users
/// # Returns
/// * `Ok(())` if the authentication is successful
/// * `Err(Error)` if the authentication fails
async fn authenticate_client(auth_method: AuthMethod, username: String, secret: String, db: &Arc<Mutex<Database>>) -> std::io::Result<()> {
    let mut db_guard = db.lock().await;
    // Get the user from the database
    let user = db_guard.get_user(&username);

    match auth_method {
        AuthMethod::Password => {
            // If the user does not exist, create it
            if user.is_none() {
                let hash = hash_password(secret.as_str());
                let _ = db_guard.add_user(username, hash.clone(), String::new());
                db_guard.save_users();
                return Ok(());
            }
            // The user exists, verify the hashed password
            let hash = user.unwrap().password.clone();
            if verify_password(secret.as_str(), hash.as_str()) {
                return Ok(());
            } else {
                return Err(Error::new(ErrorKind::Other, "Invalid password"));
            }
        }
        // Future implementation for certificate authentication
        AuthMethod::Certificate => {
            println!("Authenticating with certificate");
            Ok(())
        }
    }
}

/// Main function for the process of user commands
/// 
/// # Arguments
/// * `command` - The command to process
/// * `node` - The current node of the user
/// # Returns
/// * `Ok(String)` if the command is successful
/// * `Err(String)` if the command fails
async fn process_command(command: String, node: Arc<Mutex<Node>>) -> Result<String, String> {
    // Parse the command into args
    let mut parts = command.trim().split_whitespace();

    match parts.next() {
        Some("pwd") => {
            let node_guard = node.lock().await;
            Ok(Node::pwd(&node_guard).to_string())
        },
        Some("cd") => {
            if let Some(dir) = parts.next() {
                let current = node.lock().await;
                if let Some(new_node) = Node::cd(&current, dir) {
                    *node.lock().await = new_node;
                    Ok(format!("Changed directory to: {}", dir))
                } else {
                    Err("Directory not found".into())
                }
            } else {
                Err("No directory specified".into())
            }
        },
        Some("ls") => {
            let current = node.lock().await;
            Ok(current.ls().join(" "))
        },
        _ => Err("Unknown command".into()),
    }
}

/// Handle a client connection
/// 
/// # Arguments
/// * `stream` - The TLS stream of the client
/// * `root` - The root node of the file system
/// * `db` - The database of all users
/// # Returns
/// * `Ok(())` if the connection is closed by the client
/// * `Err(Error)` if any error
async fn handle_client(mut stream: TlsStream<TcpStream>, root: Arc<Mutex<Node>>, db: Arc<Mutex<Database>>) -> std::io::Result<()> {
    let node = root.clone();
    let peer_addr = stream.get_ref().0.peer_addr().unwrap();

    let mut buffer = [0; 1024];
    loop {
        buffer.fill(0);
        match stream.read(&mut buffer).await {
            Ok(size) => {
                if size == 0 {
                    println!("Client disconnected : {:?}", peer_addr);
                    return Ok(());
                }

                // Deserialize the message, every message has a type
                let serialized_message: TypedMessage = match serde_json::from_slice(&buffer[..size]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("Failed to parse message: {}", e);
                        continue;
                    }
                };

                // Parse the typed message to handle it
                match serialized_message {
                    TypedMessage::Command { command } => {
                        match command.as_str() {
                            "exit" => {
                                println!("Client disconnected : {:?}", peer_addr);
                                return Ok(());
                            }
                            // The result (Ok or Err) of the command is sent to the client as the return code
                            _ => match process_command(command, node.clone()).await{
                                // Send the response to the client with the command response and the return code
                                Ok(response) => {
                                    let command_response = TypedMessage::CommandResponse { response: response, success: true };
                                    let serialized_response = serde_json::to_string(&command_response).unwrap();
                                    stream.write_all(serialized_response.as_bytes()).await?;
                                },
                                Err(reponse) => {
                                    let command_response = TypedMessage::CommandResponse { response: reponse, success: false };
                                    let serialized_response = serde_json::to_string(&command_response).unwrap();
                                    stream.write_all(serialized_response.as_bytes()).await?;
                                }
                            }
                        }
                    },
                    // The tab completion is not implemented yet so it returns an empty list
                    TypedMessage::TabComplete { .. } => {
                        let completions: Vec<String> = Vec::new();
                        let tab_complete_response = TypedMessage::TabCompleteResponse { completions };
                        let serialized_response = serde_json::to_string(&tab_complete_response).unwrap();
                        stream.write_all(serialized_response.as_bytes()).await?;
                    },
                    // Handle the authentication of the client
                    TypedMessage::Auth { auth_method, username, secret } => {
                        match authenticate_client(auth_method, username, secret, &db).await {
                            Ok(_) => {
                                println!("Client authenticated : {:?}", peer_addr);
                                let response = TypedMessage::AuthResponse { success: true };
                                let serialized_response = serde_json::to_string(&response).unwrap();
                                stream.write_all(serialized_response.as_bytes()).await?;
                            }
                            Err(_) => {
                                println!("Client failed authentication : {:?}", peer_addr);
                                let response = TypedMessage::AuthResponse { success: false };
                                let serialized_response = serde_json::to_string(&response).unwrap();
                                stream.write_all(serialized_response.as_bytes()).await?;
                            }
                        }
                    },
                    _ => {
                        println!("Unknown message type");
                        return Ok(());
                    }
                }
            }
            Err(e) => println!("Erreur en lisant depuis le client : {}", e),
        }
    }
}

/// Load the certificates for the TLS connection
///
/// # Arguments
/// * `path` - The path to the certificate file
/// # Returns
/// * `Vec<Certificate>` - The vector of certificates
fn load_certs(path: &str) -> Vec<Certificate> {
    let certfile = File::open(path).unwrap();
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect()
}

/// Load the private key for the TLS connection
///
/// # Arguments
/// * `path` - The path to the private key file
/// # Returns
/// * `PrivateKey` - The private key
fn load_key(path: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    let mut r = BufReader::new(File::open(path)?);
    Ok(PrivateKey(rustls_pemfile::pkcs8_private_keys(&mut r)?.get(0).ok_or("No private keys found")?.clone()))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // Initialize the database and load users
    let db = Arc::new(Mutex::new(Database::new()));
    {
        let mut db_guard = db.lock().await;
        db_guard.load_users();
    }
    // Create the root node of the file system
    let node = Arc::new(Mutex::new(create_tree()));

    // Load the TLS certificates and private key
    let certs = load_certs("../certs/cert.pem");
    let key = load_key("../certs/cert.key.pem");

    // TLS configuration
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key.unwrap())
        .unwrap();

    // Initialize the TLS acceptor and the TCP listener
    let acceptor = Arc::new(TlsAcceptor::from(Arc::new(config)));
    let listener = TcpListener::bind(get_address()).await?;
    println!("Server started on port {}", network::SERVER_PORT);

    // For each new client connection, spawn a new task to handle it
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connected client : {:?}", addr);

        let node = std::sync::Arc::clone(&node);
        let db = std::sync::Arc::clone(&db);
        let acceptor = Arc::clone(&acceptor);
        spawn(async move {
            let tls_stream = acceptor.accept(stream).await.unwrap();
            if let Err(e) = handle_client(tls_stream, node, db).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });

    }
}

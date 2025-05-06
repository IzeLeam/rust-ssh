use common::{crypto::{hash_password, verify_password}, network::{self, get_address}, protocol::TypedMessage};
use common::auth::AuthMethod;

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
use crate::filesys::filesys::{Node, create_tree};

mod database;
use crate::database::database::Database;

async fn authenticate_client(auth_method: AuthMethod, username: String, secret: String, db: &Arc<Mutex<Database>>) -> std::io::Result<()> {
    let mut db_guard = db.lock().await;
    let user = db_guard.get_user(&username);

    match auth_method {
        AuthMethod::Password => {
            if user.is_none() {
                let hash = hash_password(secret.as_str());
                let _ = db_guard.add_user(username, hash.clone(), String::new());
                db_guard.save_users();
                return Ok(());
            }
            let hash = user.unwrap().password.clone();
            if verify_password(secret.as_str(), hash.as_str()) {
                return Ok(());
            } else {
                return Err(Error::new(ErrorKind::Other, "Invalid password"));
            }
        }
        AuthMethod::Certificate => {
            println!("Authenticating with certificate");
            Ok(())
        }
    }
}

async fn process_command(command: String, node: Arc<Mutex<Node>>) -> Result<String, String> {
    let mut parts = command.trim().split_whitespace();

    match parts.next() {
        Some("pwd") => {
            let node_guard = node.lock().await;
            Ok(node_guard.pwd())
        },
        Some("cd") => {
            if let Some(dir) = parts.next() {
                let node_guard = node.lock().await;
                if let Some(new_node) = node_guard.cd(dir) {
                    *node.lock().await = new_node;
                    let node_guard = node.lock().await;
                    Ok(format!("Changed directory to: {}", node_guard.name))
                } else {
                    Err("Directory not found".into())
                }
            } else {
                Err("No directory specified".into())
            }
        }
        Some("ls") => {
            let node_guard = node.lock().await;
            Ok(node_guard.ls().join(" "))
        },
        _ => Err("Unknown command".into()),
    }
}

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

                let serialized_message: TypedMessage = match serde_json::from_slice(&buffer[..size]) {
                    Ok(msg) => msg,
                    Err(e) => {
                        println!("Failed to parse message: {}", e);
                        continue;
                    }
                };

                match serialized_message {
                    TypedMessage::Command { command } => {
                        match command.as_str() {
                            "exit" => {
                                println!("Client disconnected : {:?}", peer_addr);
                                return Ok(());
                            }
                            _ => match process_command(command, node.clone()).await{
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
                    TypedMessage::TabComplete { stdin } => {
                        if stdin.split(' ').count() > 1 {
                            let mut parts = stdin.split_whitespace();
                            let last_part = parts.next_back().unwrap();
                            let node_guard = node.lock().await;
                            let completions = node_guard.tab_complete_arg(last_part);
                            let response = TypedMessage::TabCompleteResponse { completions: completions };
                            let serialized_response = serde_json::to_string(&response).unwrap();
                            stream.write_all(serialized_response.as_bytes()).await?;
                        } else {
                            let node_guard = node.lock().await;
                            let completions = node_guard.tab_complete(&stdin);

                            let response = TypedMessage::TabCompleteResponse { completions: completions };
                            let serialized_response = serde_json::to_string(&response).unwrap();
                            stream.write_all(serialized_response.as_bytes()).await?;
                        }
                    },
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


fn load_certs(path: &str) -> Vec<Certificate> {
    let certfile = File::open(path).unwrap();
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect()
}

fn load_key(path: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    let mut r = BufReader::new(File::open(path)?);
    Ok(PrivateKey(rustls_pemfile::pkcs8_private_keys(&mut r)?.get(0).ok_or("No private keys found")?.clone()))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let db = Arc::new(Mutex::new(Database::new()));
    {
        let mut db_guard = db.lock().await;
        db_guard.load_users();
    }
    let node = Arc::new(Mutex::new(create_tree()));

    let certs = load_certs("../certs/cert.pem");
    let key = load_key("../certs/cert.key.pem");

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key.unwrap())
        .unwrap();

    let acceptor = Arc::new(TlsAcceptor::from(Arc::new(config)));

    let listener = TcpListener::bind(get_address()).await?;
    println!("Server started on port {}", network::SERVER_PORT);

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

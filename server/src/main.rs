use common::{auth::{self, Authenticator}, network::{self, get_address}, protocol::TypedMessage};
use common::auth::AuthMethod;
use common::protocol::Auth;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    spawn,
};

use std::io::{Error, ErrorKind};
use std::sync::Arc;

mod filesys;
use crate::filesys::filesys::{Node, create_tree};

mod database;
use crate::database::database::Database;

async fn authenticate_client(auth_message: Auth, db: &Arc<Mutex<Database>>) -> std::io::Result<()> {
    let mut db_guard = db.lock().await;
    let user = db_guard.get_user(&auth_message.username);

    match auth_message.auth_method {
        AuthMethod::Password => {
            if user.is_none() {
                let _ = db_guard.add_user(auth_message.username, auth_message.secret.clone(), String::new());
                db_guard.save_users();
                return Ok(());
            }
            let saved_password = user.unwrap().password.clone();
            let password = auth_message.secret;
            let authenticator = auth::PasswordAuth {
                password: saved_password.to_string(),
            };
            authenticator.authenticate(password.clone().as_str()).then(|| {
                Ok(())
            }).ok_or_else(|| {
                Error::new(ErrorKind::PermissionDenied, "Authentication failed")
            })?
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
                if let Some(new_node) = {
                    let node_guard = node.lock().await;
                    node_guard.cd(dir)
                } {
                    *node.lock().await = new_node;
                    Ok(format!("Changed directory to: {}", dir))
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

async fn handle_client(mut stream: TcpStream, node: Arc<Mutex<Node>>, db: Arc<Mutex<Database>>) -> std::io::Result<()> {
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) => {
                if size == 0 {
                    println!("Client disconnected : {:?}", stream.peer_addr().unwrap());
                    return Ok(());
                }

                let serialized_message: TypedMessage = serde_json::from_slice(&buffer[..size]).unwrap();

                match serialized_message {
                    TypedMessage::Command(command) => {
                        match command.command.as_str() {
                            "exit" => {
                                println!("Client disconnected : {:?}", stream.peer_addr().unwrap());
                                return Ok(());
                            }
                            _ => match process_command(command.command, node.clone()).await{
                                Ok(response) => {
                                    stream.write_all(response.as_bytes()).await?;
                                },
                                Err(reponse) => {
                                    stream.write_all(reponse.as_bytes()).await?;
                                }
                            }
                        }
                    },
                    TypedMessage::TabComplete(start) => {
                        if start.split(' ').count() > 1 {
                            let mut parts = start.split_whitespace();
                            let last_part = parts.next_back().unwrap();
                            let node_guard = node.lock().await;
                            let completions = node_guard.tab_complete_arg(last_part);
                            let response = TypedMessage::TabCompleteResponse(completions);
                            let serialized_response = serde_json::to_string(&response).unwrap();
                            stream.write_all(serialized_response.as_bytes()).await?;
                        } else {
                            let node_guard = node.lock().await;
                            let completions = node_guard.tab_complete(&start);

                            let response = TypedMessage::TabCompleteResponse(completions);
                            let serialized_response = serde_json::to_string(&response).unwrap();
                            stream.write_all(serialized_response.as_bytes()).await?;
                        }
                    },
                    TypedMessage::Auth(auth_message) => {
                        match authenticate_client(auth_message, &db).await {
                            Ok(_) => {
                                println!("Client authenticated : {:?}", stream.peer_addr().unwrap());
                                let response = TypedMessage::AuthResponse(true);
                                let serialized_response = serde_json::to_string(&response).unwrap();
                                stream.write_all(serialized_response.as_bytes()).await?;
                            }
                            Err(_) => {
                                println!("Client failed authentication : {:?}", stream.peer_addr().unwrap());
                                let response = TypedMessage::AuthResponse(false);
                                let serialized_response = serde_json::to_string(&response).unwrap();
                                stream.write_all(serialized_response.as_bytes()).await?;
                                return Ok(());
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

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let db = Arc::new(Mutex::new(Database::new()));
    {
        let mut db_guard = db.lock().await;
        db_guard.load_users();
    }
    let node = Arc::new(Mutex::new(create_tree()));

    let listener = TcpListener::bind(get_address()).await?;
    println!("Server started on port {}", network::SERVER_PORT);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New connected client : {:?}", addr);

        let node = std::sync::Arc::clone(&node);
        let db = std::sync::Arc::clone(&db);
        spawn(async move {
            if let Err(e) = handle_client(stream, node, db).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });

    }
}

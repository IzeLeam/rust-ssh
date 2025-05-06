use std::path::PathBuf;
use common::network::get_address;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use clap::Parser;
use rpassword::prompt_password;
use std::io::Write;

use common::auth::AuthMethod;
use common::protocol::TypedMessage;

/// Command parsing
#[derive(Parser)]
#[clap(name = "Rust SSH", version = "1.0",
    about = "A simple SSH client",
    long_about = "Usage: client username [options]")]
pub struct Cli {
    /// Username
    #[clap(help = "Username to connect to the server")]
    username: String,
    /// Public key
    #[clap(short = 'i', help = "Public key to connect to the server")]
    pubkey: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let tcp_stream = TcpStream::connect(get_address()).await.expect("Erreur de connexion au serveur");
    let (mut read_half, mut write_half) = tcp_stream.into_split();

    // Authentication
    match cli.pubkey {
        Some(_path) => {
            println!("Not implemented yet");
            std::process::exit(1);
        },
        None => {
            let password = prompt_password("Enter your passsword: ").unwrap();
            let message_type = TypedMessage::new_auth(AuthMethod::Password, cli.username, password);
            let serialized_message = serde_json::to_string(&message_type).unwrap();
            write_half.write_all(serialized_message.as_bytes()).await.expect("Erreur d'écriture dans le serveur");
        }
    }

    let mut write_buffer = String::new();
    let mut read_buffer: [u8; 1024] = [0; 1024];

    // Read the server response
    let size = read_half.read(&mut read_buffer).await.expect("Erreur de lecture depuis le serveur");
    let serialized_message: TypedMessage = serde_json::from_slice(&read_buffer[..size]).unwrap();
    match serialized_message {
        TypedMessage::AuthResponse(b) => {
            if b {
                println!("Authenticated");
            } else {
                println!("Authentication failed");
                return;
            }
        }
        _ => {
            println!("Unexpected message type");
            return;
        }
    }
    
    println!("Enter a command (exit to quit) :");
    loop {
        // Read command from stdin
        write_buffer.clear();
        std::io::stdout().write(b"> ").unwrap();
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut write_buffer).expect("Erreur de lecture de la ligne");
        if write_buffer.is_empty() || write_buffer.trim().is_empty() {
            continue;
        }

        // Send the command
        let command_str = write_buffer.trim();
        let message = TypedMessage::new_command(command_str.to_string());
        let serialized_message = serde_json::to_string(&message).unwrap();
        write_half.write_all(serialized_message.as_bytes()).await.expect("
    TypedMessage,
    Command,
    Auth,
};Erreur d'écriture dans le serveur");

        // Get the server response
        read_buffer.fill(0);
        match read_half.read(&mut read_buffer).await {
            Ok(size) => {
                if size == 0 {
                    println!("Connection closed");
                    return;
                }

                let utf8_string = String::from_utf8_lossy(&read_buffer[..size]);
                println!("Server: {}", utf8_string);
            }
            Err(e) => {
                println!("Erreur en lisant depuis le serveur : {}", e);
                break;
            }
        }
    }
}

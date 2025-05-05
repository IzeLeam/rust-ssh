use common::network;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    spawn,
};

mod filesys;
use filesys::filesys::*;


async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!("Nouveau client connecté : {:?}", stream.peer_addr());
    let mut buffer = [0; 512];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) => {
                let utf8_string = String::from_utf8_lossy(&buffer[..size]);
                let mut parts = utf8_string.trim().split_whitespace();
                match parts.next() {
                    Some("pwd") => {
                        let _ = stream.write_all(b"pwd of current dir\n").await;            
                    }
                    Some("cd") => {
                        println!("Command cd received");
                        if let Some(arg) = parts.next() {
                            let _ = stream.write_all(format!("cd to arg : {}\n", arg).as_bytes()).await;
                        } else {
                            let _ = stream.write_all(b"cd root\n").await;
                        }
                    }
                    Some("ls") => {
                        println!("Command ls received");
                        if let Some(arg) = parts.next() {
                            let _ = stream.write_all(format!("ls to arg : {}\n", arg).as_bytes()).await;
                        } else {
                            let _ = stream.write_all(b"ls current dir\n").await;
                        }                
                    }
                    Some("exit") => {
                        println!("Command exit received");
                        let _ = stream.write_all(b"exiting the server\n").await;
                        return Ok(());
                    }
                    _ => {
                        println!("Unknown command received");
                        let _ = stream.write_all(b"Unknown command\n").await;
                    }
                }
            }
            Err(e) => println!("Erreur en lisant depuis le client : {}", e),
        }
    }
}

/// TODO : Serde pour bdd json


#[tokio::main]
async fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Le serveur est en écoute sur le port 7878");

    // Création du dossier racine
    let mut root = Node::new_directory("root".to_string());

    // Création de sous-dossiers
    let mut dir1 = Node::new_directory("dir1".to_string());
    let mut dir2 = Node::new_directory("dir2".to_string());
    let mut dir3 = Node::new_directory("dir3".to_string());

    // Création de fichiers
    let file1 = Node::new_file("file1.txt".to_string());
    let file2 = Node::new_file("file2.txt".to_string());
    let file3 = Node::new_file("file3.txt".to_string());
    let file4 = Node::new_file("file4.txt".to_string());

    // Ajout des fichiers et dossiers à l'arborescence
    dir1.add_child(file1);
    dir1.add_child(file2);
    dir2.add_child(file3);
    dir3.add_child(file4);

    root.add_child(dir1);
    root.add_child(dir2);
    root.add_child(dir3);

    println!("{:#?}", root);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Nouveau client connecté : {:?}", addr);
        spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });

    }
}

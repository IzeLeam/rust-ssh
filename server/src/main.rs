use common::network;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};

fn handle_client(mut stream: TcpStream) {
    println!("Nouveau client connecté : {:?}", stream.peer_addr());
    let mut buffer = [0; 512];
    match stream.read(&mut buffer) {
        Ok(size) => {
            println!("Données reçues : {}", String::from_utf8_lossy(&buffer[..size]));
            let _ = stream.write_all(b"Commande re\xE7ue !");
        }
        Err(e) => println!("Erreur en lisant depuis le client : {}", e),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Impossible de lier le serveur sur le port 7878");
    println!("Le serveur est en écoute sur le port 7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(e) => println!("Erreur de connexion : {}", e),
        }
    }
}

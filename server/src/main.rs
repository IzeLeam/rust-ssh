use common::network;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::Mutex,
    spawn,
};


async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    println!("Nouveau client connecté : {:?}", stream.peer_addr());
    let mut buffer = [0; 512];
    match stream.read(&mut buffer).await {
        Ok(size) => {
            println!("Données reçues : {}", String::from_utf8_lossy(&buffer[..size]));
            let _ = stream.write_all(b"Commande re\xE7ue !").await;
        }
        Err(e) => println!("Erreur en lisant depuis le client : {}", e),
    }
    Ok(())
}

/// TODO : Serde pour bdd json


#[tokio::main]
async fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Le serveur est en écoute sur le port 7878");

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

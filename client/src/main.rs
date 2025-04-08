use common::protocol;
use std::io;

fn main() {
    println!("Lancement du client d'administration à distance !");
    
    // Exemple d'utilisation d'un module du crate commun :
    protocol::afficher_version();

    // Ici, vous pourrez ajouter la logique de connexion, authentification, et REPL.
    println!("Entrez une commande (ou 'exit' pour quitter) :");

    let mut buffer = String::new();
    loop {
        buffer.clear();
        print!("> ");
        let _ = io::Write::flush(&mut io::stdout());
        io::stdin().read_line(&mut buffer).expect("Erreur de lecture");
        let commande = buffer.trim();
        if commande.eq_ignore_ascii_case("exit") {
            println!("Fermeture du client.");
            break;
        }
        println!("Commande reçue : {}", commande);
        // Vous traiterez la commande ici...
    }
}

use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::str;

// Définition des paramètres
const LOCAL: &str = "0.0.0.0:8888";
const MSG_SIZE: usize = 32;

// Lecture de la saisie utilisateur
fn read_user_entry() -> String {
    let mut user_entry = String::new();
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut user_entry).expect("Failed to read stdin");

    user_entry = user_entry.trim().parse().unwrap();

    return user_entry;

}

fn main() {
    println!("---- Bienvenue chez Massimora's Chat ! ----");
    print!("Saisie ton pseudo > ");
    let username: String = read_user_entry();

    // Connexion au serveur, en mode non bloquant
    let mut client = TcpStream::connect(LOCAL).expect("Failed to connect");
    client.set_nonblocking(true).expect("Non-blocking can't be initiate");

    // Sender / Received
    let (tx, rx) = mpsc::channel::<String>();

    // Création d'un thread permettant la reception des données venant du client
    thread::spawn(move || loop {
        let mut buff = vec![0; MSG_SIZE];
        // A la réception d'un message
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg_buffer = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg_ascii = String::from_utf8(msg_buffer).expect("Invalid UTF-8 sequence");

                println!("{}", msg_ascii);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Error ... Connection stopped");
                break;
            }
        }

        // Envoie des données au serveur
        match rx.try_recv() {
            Ok(msg) => {
                if msg != "" {
                    // Parsing du message afin d'ajouter le username dans le message
                    let mut full_message: String = String::new();
                    full_message.push_str(username.trim());
                    full_message.push_str(" : ");
                    full_message.push_str(&*msg);
                    let mut buff = full_message.clone().into_bytes();
                    // Réecriture du message pour qu'il corresponde à la taille du buffer
                    buff.resize(MSG_SIZE, 0);
                    // Envoie à TOUS les clients notre message
                    client.write_all(&buff).expect("Unable to write into socket...");
                }

            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        // Raffraîchissement du thread toutes les 100ms
        thread::sleep(Duration::from_millis(100));
    });


    // Ecriture d'un message dans le terminal
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Failed to read stdin");
        let msg = buff.trim().to_string();

        // Commande pour quitter le chat
        if msg == "!quit" || "msg" == ":quit" || tx.send(msg).is_err() {
            break
        }
    }

    println!("---- A bientôt sur Massimora's Chat ! ----");
}

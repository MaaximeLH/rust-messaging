use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;

// Définition des paramètres
const LOCAL: &str = "0.0.0.0:8888";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    println!("---- Massimora's Chat Server Listening to {} ! ----", LOCAL);
    // Création d'un Listener TCP, en mode non-bloquant
    let server = TcpListener::bind(LOCAL).expect("Unable to bind listener");
    server.set_nonblocking(true).expect("Non-blocking can't be initiate");

    // Tableau de nos clients
    let mut clients = vec![];

    // Sender / Received
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        // Nouvelle connexion Tcp / Nouveau client
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Unable to clone client"));

            // Création d'un thread, permettant la reception des données des clients
            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("Unable to send message to client");
                    }, 
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("{} has closed connection", addr);
                        break;
                    }
                }

                sleep();
            });
        }

        // Envoie du message à tous les clients
        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);

                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }

        sleep();
    }
}

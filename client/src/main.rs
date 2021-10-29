use std::io;
use std::net::{TcpStream};
use std::io::{Read, Write};

fn read(to_print: String) -> String {
    let mut user_entry = String::new();
    print!("{}", to_print);
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut user_entry).unwrap();

    return user_entry;
}

fn main() {
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            println!("[INFO] Connexion au serveur distant réussie via le port 3333");

            let mut username: String = read(String::from("[INFO] Saisie ton pseudo > "));
            username.pop();
            println!("[INFO] Bienvenue {}", username.trim());

            loop {
                println!();
                let mut message = read(String::from("[INFO] Ton message > "));
                message.pop();

                let mut final_message = username.clone();
                final_message.push_str(" : ");
                final_message.push_str(&*message);

                println!("{}", final_message.trim());

                stream.write(final_message.as_ref()).unwrap();

                /*let mut data = [0 as u8; 6]; // using 6 byte buffer

                match stream.read_exact(&mut data) {
                    Ok(_) => {
                        let message_from_server = from_utf8(&data).unwrap();
                        println!("{}", message_from_server);
                    },
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }*/
            }
        },
        Err(e) => {
            println!("Impossible de me connecter: {}", e);
        }
    }
}
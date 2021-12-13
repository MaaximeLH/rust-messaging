use std::{io::{ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, thread, sync::mpsc};
use argon2::{self, Config};
use json::{self, JsonValue, object};

// Définition des paramètres
const LOCAL: &str = "0.0.0.0:8888";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

struct User {
    /// The pseudo the user will use inside the chat.
    pseudo: String,
    /// The user's password to authenticate on the chat.
    pwd: String,
    /// The socket the user is connected on.
    socket: TcpStream
}

impl User {
    /// Function to get the user's pseudo.
    /// Returns a String
    fn get_pseudo(&self) -> &String {
        return &self.pseudo;
    }

    /// Function to get the user's password.
    /// Returns a String
    fn get_pwd(&self) -> &String {
        return &self.pwd;
    }

    /// Function to get the user's socket.
    /// Returns a TcpStream
    fn get_socket(&self) -> &TcpStream {
        return &self.socket;
    }

    /// Function to create a new User.
    /// Returns an instance of User Structure
    fn create_user(pseudo: String, pwd: String, socket: TcpStream) -> User {
        User {
            pseudo,
            socket,
            pwd
        }
    }

    /// Function to update the socket of a user.
    fn update_socket(&mut self, socket: TcpStream) {
        self.socket = socket;
    }

    /// Returns a printable string containing user data
    fn to_string(&self) -> String {
        let mut serie:String = String::from("User: \"");
        serie.push_str(self.pseudo.as_str());
        serie.push_str("\", pwd: \"");
        serie.push_str(self.pwd.as_str());
        serie.push_str("\"");
    
        return serie;
    }

    /// Returns a json string containing user data
    fn to_json(&self) -> String {
        let user_json:JsonValue = object!{
            username: self.pseudo.clone(),
            pwd: self.pwd.clone(),
        };
        
        return json::stringify(user_json);
    }
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
    let users:Vec<User> = Vec::new();
    let tmp = object! {"username":"toto","pwd":"$argon2i$v=19$m=4096,t=3,p=1$cnVzdF9tZXNzYWdpbmc$a+URuyk304JEitqJVXFafsjJC0bigXQvePC7IWUJ75k"};


    // println!("")

    let hash = encode_pwd(String::from("test"));
    verify_pwd(String::from("test"), &hash);
    verify_pwd(String::from("ok"), &hash);

    println!("Rust m3ss4g1ng by ESGI starting ...");

    parse_user_json(tmp.to_string());

}

/// Function to read a user entry.
/// Returns a String.
fn read_user_entry() -> String {
    use std::io;
    let mut user_entry = String::new();
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut user_entry);

    user_entry = user_entry.trim().parse().unwrap();

    return user_entry;

}

/// Return an encoded string corresponding to the hash of the given one.
fn encode_pwd(pwd:String) -> String{
    return argon2::hash_encoded(pwd.as_bytes(), String::from("rust_messaging").as_bytes(), &Config::default()).unwrap();
}

/// Verify the match between the pwd and the hash.
/// Returns true if match, else false.
fn verify_pwd(pwd:String, hash:&String) -> bool {
    return argon2::verify_encoded(&hash, pwd.as_bytes()).unwrap();
}

fn parse_user_json(mut user:String) /*-> User */{
    let user= json::parse(&user);
    println!("{:?}", user);
    // return User::create_user(user[""], pwd: String, socket: TcpStream);
}
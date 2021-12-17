use std::{io::{ErrorKind, Read, Write}, net::{TcpListener, TcpStream}, thread, sync::{mpsc, Arc, Mutex}};
use argon2::{self, Config};
use json::{self, JsonValue, object};
use rand::{Rng, thread_rng, distributions::Alphanumeric};

// Définition des paramètres
const CHAT: &str = "0.0.0.0:8888";
const CONNECT: &str = "0.0.0.0:8889";
const REGISTER: &str = "0.0.0.0:8890";

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

struct User {
    /// The pseudo the user will use inside the chat.
    pseudo: String,
    /// The user's password to authenticate on the chat.
    pwd: String,
    /// The socket the user is connected on.
    socket: TcpStream,
    /// Token send by the server to keep user connected
    token: String
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

    /// Function to get the user's token.
    /// Returns a String
    fn get_token(&self) -> &String {
        return &self.token;
    }

    /// Function to set the new token of the user
    fn set_token(&mut self, new_token: String) {
        self.token = new_token
    }

    /// Function to create a new User.
    /// Returns an instance of User Structure
    fn create_user(pseudo: String, pwd: String, socket: TcpStream) -> User {
        User {
            pseudo,
            socket,
            pwd,
            token: String::new()
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
        serie.push_str("\", token: \"");
        serie.push_str(self.token.as_str());
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

impl Clone for User {
    fn clone(&self) -> User {
        let mut user = User::create_user(self.get_pseudo().to_string(), self.get_pwd().to_string(), self.get_socket().try_clone().expect("Can't clone"));
        user.set_token(self.get_token().to_string());
        return user;
    }
}


fn main() {
    println!("---- Massimora's Chat Server Listening to {} ! ----", CHAT);
    // Création d'un Listener TCP, en mode non-bloquant
    let server = TcpListener::bind(CHAT).expect("Unable to bind listener");
    let connect = TcpListener::bind(CONNECT).expect("Unable to bind listener");
    let register = TcpListener::bind(REGISTER).expect("Unable to bind listener");
    server.set_nonblocking(true).expect("Non-blocking can't be initiate");
    connect.set_nonblocking(true).expect("Non-blocking can't be initiate");
    register.set_nonblocking(true).expect("Non-blocking can't be initiate");

    // Tableau de nos clients
    // let mut clients = vec![];

    let mut registered = Arc::new(Mutex::new(vec![]));

    // Sender / Received
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        // Nouvelle connexion Tcp / Nouveau client
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            let tx = tx.clone();
            // clients.push(socket.try_clone().expect("Unable to clone client"));

            // Création d'un thread, permettant la reception des données des clients
            let mut clone_registered = Arc::clone(&registered);
            thread::spawn(move || loop {
                let mut buff = vec![0; 256];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let mut msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        let mut data_registered = clone_registered.lock().unwrap();

                        // println!("{}: {:?}", addr, msg);
                        let content = json::parse(msg.as_str()).unwrap_or(object !{});
                        let user = content["from"].clone();
                        let user = json::parse(user.to_string().as_str()).unwrap_or(object !{});
                        let my_user = User::create_user(user["username"].to_string(), user["pwd"].to_string(), socket.try_clone().expect("Can't clone socket"));
                        // println!("user {} on {:?}", user["username"].to_string(), socket);
                        update_user_socket(my_user.get_pseudo().to_string(), user["token"].to_string(), &mut data_registered, socket.try_clone().expect("Can't clone socket"));

                        // println!("{}", msg);
                        
                        if is_connected(user["username"].to_string(), user["token"].to_string(), data_registered.to_vec()) {
                            tx.send(msg.clone()).expect("Unable to send message to client");
                        }
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

        // User login
        if let Ok((mut socket, addr)) = connect.accept() {
            println!("Client {} try to connect", addr);

            let tx = tx.clone();
            // clients.push(socket.try_clone().expect("Unable to clone client"));

            let mut clone_registered = Arc::clone(&registered);
            thread::spawn(move || loop {
                let mut buff = vec![0; 256];
                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        let mut data_registered = clone_registered.lock().unwrap();

                        // println!("{}: {:?}", addr, msg);
                        let users = json::parse(msg.as_str()).unwrap_or(object!{});

                        let username:String = users["username"].to_string();
                        let pwd:String = users["pwd"].to_string();

                        let user:User = User::create_user(username, pwd, socket.try_clone().expect("Can't open stream"));

                        if search_registered(user.clone(), data_registered.to_vec()) {
                            println!("{} connected", user.get_pseudo());
                            let token = create_token();
                            define_token(token.clone(), &mut data_registered, user.clone());
                            let mut buffer = token.into_bytes();
                            buffer.resize(30, 0);
                            user.get_socket().write(&buffer).map(|_| user.get_socket()).ok();
                        } else {
                            let mut buffer = String::from("").into_bytes();
                            buffer.resize(30, 0);
                            user.get_socket().write_all(&buffer).map(|_| user.get_socket()).ok();
                        }
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

        // User register
        if let Ok((mut socket, addr)) = register.accept() {
            println!("Client {} try to register", addr);

            let tx = tx.clone();
            // clients.push(socket.try_clone().expect("Unable to clone client"));

            let mut clone_registered = Arc::clone(&registered);

            thread::spawn(move || loop {
                let mut buff = vec![0; 256];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        let mut data_registered = clone_registered.lock().unwrap();

                        let data = json::parse(msg.as_str()).unwrap_or(object!{});
                        let mut user:User = User::create_user(data["username"].to_string(), data["pwd"].to_string(), socket.try_clone().expect("Can't clone"));

                        if verify_pseudo(data["username"].to_string(), data_registered.to_vec()) {
                            println!("{} registered", user.get_pseudo());
                            user.set_token(create_token());
                            data_registered.push(user.clone());
                            let mut buffer = user.get_token().to_string().into_bytes();
                            buffer.resize(30, 0);
                            user.get_socket().write(&buffer).map(|_| user.get_socket()).ok();
                        } else {
                            let mut buffer = String::from("").into_bytes();
                            buffer.resize(30, 0);
                            user.get_socket().write(&buffer).map(|_| user.get_socket()).ok();
                        }
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
            
            let content = json::parse(msg.as_str()).unwrap_or(object !{});
            let user = content["from"].clone();
            let user = json::parse(user.to_string().as_str()).unwrap_or(object !{});
            if content["content"].to_string() != "" {
            let mut msg = String::new();
            msg.push_str(&user["username"].to_string());
            msg.push_str(" : ");
            msg.push_str(&content["content"].to_string());
            println!("{}", msg);
                for send_to in registered.lock().unwrap().clone() {
                    // println!("{}", send_to.get_pseudo());
                    // println!("{}", send_to.get_token());
                    // println!("{}", user["username"].to_string());
                    // println!("{}", user["token"].to_string());
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(256, 0);
                    if send_to.get_pseudo().to_string() != user["username"].to_string() && send_to.get_token().to_string() != user["token"].to_string() {
                        // println!("Différent pour {} socket {:?}", send_to.get_pseudo(), send_to.get_socket());
                        send_to.get_socket().write(&buff).map(|_| send_to.get_socket()).ok();
                    }
                }
            }
        }
        sleep();
    }
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

fn search_registered(user: User, users:Vec<User> ) -> bool {
    for all_users in users {
        if all_users.get_pseudo() == user.get_pseudo() && all_users.get_pwd() == user.get_pwd() {
            return true;
        }
    }
    return false;
}

fn create_token() -> String {
    let token:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    return token;
}

fn verify_pseudo(pseudo:String, users: Vec<User>) -> bool {
    for all_users in users {
        if all_users.get_pseudo().to_string() == pseudo {
            return false;
        }
    }
    return true;
}

fn define_token(token:String, users:&mut Vec<User>, user:User) {
for all_users in users {
        if all_users.get_pseudo() == user.get_pseudo() && all_users.get_pwd() == user.get_pwd() {
            all_users.set_token(token.clone());
        }
    }
}

fn is_connected(pseudo:String, token:String, users:Vec<User>) -> bool {
    for mut x in users {
        if x.get_pseudo().to_string() == pseudo && x.get_token().to_string() == token {
            return true;
        }
    }
    return false;
}

fn update_user_socket(pseudo:String, token:String, users:&mut Vec<User>, socket:TcpStream) {
    for x in users {
        // println!("{}", x.get_token());
        // println!("{}", x.get_pseudo());
        // println!("{} !{}!", pseudo, token);
        if x.get_pseudo().to_string() == pseudo && x.get_token().to_string() == token {
            x.update_socket(socket.try_clone().expect("Can't clone socket"));
            // println!("socket update")
        }
    }
}

// fn print_user(users:Vec<User>) -> bool {
//     for x in users {
//         println!("{} <{:?}>", x.get_pseudo(), x.get_socket());
//     }
//     return false;
// }
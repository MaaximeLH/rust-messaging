use std::{io::{Write, Read, ErrorKind, self}, 
{str, time::Duration, thread, net::{TcpStream}, }, 
sync::{mpsc::{self, TryRecvError}, Arc, Mutex}};
use argon2::{self, Config};
use json::{self, JsonValue, object};
use regex;

/// Definition of server addresses
const CHAT: &str = "0.0.0.0:8888";
const CONNECT: &str = "0.0.0.0:8889";
const REGISTER: &str = "0.0.0.0:8890";

struct User {
    /// The pseudo the user will use inside the chat.
    pseudo: String,
    /// The user's password to authenticate on the chat.
    pwd: String,
    /// Token send by the server to keep user connected
    token: String
}

struct Message {
    /// The user who send the message.
    from: User,
    /// The destination of the message
    to: String,
    /// Content of the message sent.
    content: String
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
    /// Returns an instance of User Structure.
    fn create_user(pseudo: String, pwd: String) -> User {
        User {
            pseudo,
            pwd: encode_pwd(pwd),
            token: String::new()
        }
    }

    /// Returns a printable string containing user data.
    fn to_string(&self) -> String {
        let mut serie:String = String::from("User: \"");
        serie.push_str(self.pseudo.as_str());
        serie.push_str("\", pwd: \"");
        serie.push_str(self.pwd.as_str());
        serie.push_str("\", token: \"");
        serie.push_str(self.token.as_str());
        serie.push_str("\"");
    
        return serie;
    }

    /// Returns a json string containing user data.
    fn to_json(&self) -> String {
        let user_json:JsonValue = object!{
            username: self.pseudo.clone(),
            pwd: self.pwd.clone(),
            token: self.token.clone(),
        };

        return json::stringify(user_json);
    }

    /// Return a clone of user.
    fn to_user(&self) -> User {
        User {
            pseudo: self.get_pseudo().clone(),
            pwd: self.get_pwd().clone(),
            token: self.get_token().clone()
        }
    }

    /// Create a new user.
    fn new(pseudo: String, pwd: String) -> User {
        User {
            pseudo,
            pwd,
            token: String::new()
        }
    }
}

impl Clone for User {
    fn clone(&self) -> User {
        let mut user = User::create_user(self.get_pseudo().to_string(), self.get_pwd().to_string());
        user.set_token(self.get_token().to_string());
        return user;
    }
}

impl Message {
    /// Create a new message.
    fn new(user: User, to:String, content:String) -> Message {
        Message{
            from: user,
            to,
            content
        }
    }

    fn to_json(&self) -> String {
        let message:JsonValue = object!{
            from: self.from.to_json().clone(),
            to: self.to.clone(),
            content: self.content.clone(),
        };
        
        return json::stringify(message);
    }
}


/// Function to read a user entry.
/// Returns a String.
fn read_user_entry() -> String {
    let mut user_entry = String::new();
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut user_entry);

    user_entry = user_entry.trim().parse().unwrap_or(String::from(""));

    return user_entry;
}

/// Return an encoded string corresponding to the hash of the given one.
/// # Example
/// Get the hash of the string "Hello world !"
/// ``` rust
/// let input = String::from("Hello world !");
/// let hash = encode_pwd(input)
/// ```
fn encode_pwd(pwd:String) -> String{
    return argon2::hash_encoded(pwd.as_bytes(), String::from("rust_messaging").as_bytes(), &Config::default()).unwrap_or(String::from(""));
}

/// Verify the match between the pwd and the hash.
/// Returns true if match, else false.
fn verify_pwd(pwd:String, hash:&String) -> bool {
    return argon2::verify_encoded(&hash, pwd.as_bytes()).unwrap_or(false);
}

fn general_menu() {
    loop {
        println!("What do you want to do ?");
        println!("!c - connect");
        println!("!r - register");
        println!("!q- Quit");
        let entry:String = read_user_entry();
        let entry = entry.as_str();

        
        let user:User;

        match entry {
            "!q" | "!quit" => {
                println!("Quit");
                break;
            }
            "!c" | "!connect" => {
                let tmp = connect();
                if tmp.0 == true {
                    user = tmp.1;
                } else {
                    continue;
                }
            }
            "!r" | "!register" => {
                let tmp = register();
                if tmp.0 == true {
                    user = tmp.1;
                } else {
                    continue;
                }
            }
            "!h" | "!help" => {
                display_help();
                continue;
            }
            _ => {
                println!("Not a good option: {}", entry);
                continue;
            }
        }
        chat_menu(user);
    }  
}

/// Display help for user.
fn display_help() {
    println!("!h or !help       -> display the help");
    println!("!q or !quit       -> makes you quit the rust messaging program");
    println!("!c or !connect    -> (only on the menu) launch the connect program");
    println!("!r or !register   -> (only on the menu) launch the register program");
    println!("!p or !private    -> (only in chat menu or inside a chat) send private message to a user");
    println!("!l or !list       -> (only inside a chat) list all connected users");
    println!("!g or !general    -> (only in chat menu) connect to general chat");
}

fn connect() -> (bool, User) {
    let mut client = TcpStream::connect(CONNECT).expect("Failed to connect");
    client.set_nonblocking(true).expect("Non-blocking can't be initiate");

    println!("");
    println!("--------------------");
    print!("Enter username: ");
    let pseudo:String = read_user_entry();
    print!("Enter password: ");
    let pwd:String = read_user_entry();
    println!("--------------------");
    println!("");

    let mut user = User::create_user(pseudo, pwd);

    let mut full_message = String::new();
    full_message.push_str(user.to_json().as_str());
    
    let mut buff = full_message.clone().into_bytes();
    buff.resize(256, 0);
    client.write_all(&buff).expect("Unable to write into socket...");

    loop {
        let mut buff = vec![0; 30];

        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg_buffer = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg_ascii = String::from_utf8(msg_buffer).expect("Invalid UTF-8 sequence");

                
                if msg_ascii != "" {
                    user.set_token(msg_ascii);
                }
                
                break;
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Error ... Connection stopped");
                break;
            }
        }
    }

    if user.get_token() == "" {
        println!("Invalid login/pwd");
        return (false, user);
    } else {
        return (true, user);
    }
}

fn register() -> (bool, User) {
    println!("Register");

    print!("Enter username: ");
    let pseudo:String = read_user_entry();

    print!("Enter password: ");
    let pwd:String = read_user_entry();

    let mut user = User::create_user(pseudo, pwd);

    let mut client = TcpStream::connect(REGISTER).expect("Failed to connect");
    client.set_nonblocking(true).expect("Non-blocking can't be initiate");

    let mut full_message = String::new();
    full_message.push_str(user.to_json().as_str());
    // println!("{}", user.to_json().as_str());
    
    let mut buff = full_message.clone().into_bytes();
    buff.resize(256, 0);
    client.write_all(&buff).expect("Unable to write into socket...");

    loop {
        let mut buff = vec![0; 30];
        match client.read_exact(&mut buff) {
            Ok(_) => {
                let msg_buffer = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg_ascii = String::from_utf8(msg_buffer).expect("Invalid UTF-8 sequence");

                // println!("{}", msg_ascii);
                if msg_ascii != "" {
                    user.set_token(msg_ascii);
                }
                break;
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Error ... Connection stopped");
                break;
            }
        }
    }

    if user.get_token() == "" {
        return (false, user);
    } else {
        return (true, user);
    }
}

fn chat_menu(user: User) {
    println!("Welcome {}", user.get_pseudo());

    loop {
        println!("!g- Enter in general chat");
        println!("!q- Quit");

        let entry = read_user_entry();
        let entry = entry.as_str();

        match entry {
            "!g" | "!general" => {
                chat(String::from("general"), &user);
            }
            "!q" | "!quit" => {
                println!("Quit");
                break;
            }
            _ => {
                println!("Not a good option: {}", entry);
                continue;
            }
        }
    }
}

fn chat(chat_type:String, user:&User) {
    if chat_type == "general" {
        let mut client = TcpStream::connect(CHAT).expect("Failed to connect");
        client.set_nonblocking(true).expect("Non-blocking can't be initiate");

        // Sender / Received
        let (tx, rx) = mpsc::channel::<String>();
        let data_clone = user.clone();

        // Création d'un thread permettant la reception des données venant du client
        thread::spawn(move || loop {
            let mut buff = vec![0; 256];
            // Envoie des données au serveur
            match rx.try_recv() {
                Ok(msg) => {
                    if msg != "" {

                        let message:Message = Message::new(data_clone.clone(), String::from("general"), msg);
                        
                        let mut buff = message.to_json().into_bytes();
                        buff.resize(256, 0);
                        client.write_all(&buff).expect("Unable to write into socket...");
                    }
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break
            }
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

            // Raffraîchissement du thread toutes les 100ms
            thread::sleep(Duration::from_millis(100));
        });


        // Ecriture d'un message dans le terminal
        loop {
            let mut buff = String::new();
            io::stdin().read_line(&mut buff).expect("Failed to read stdin");
            let msg = buff.trim().to_string();

            // Commande pour quitter le chat
            if msg.clone() == "!quit" || msg.clone() == ":quit" || tx.send(msg.clone()).is_err() {
                break
            }
            if msg == "!help" || msg == "!h" {
                display_help();
            }
        }
    }
}

fn main() {
    let hash = encode_pwd(String::from("test"));
    verify_pwd(String::from("test"), &hash);
    verify_pwd(String::from("ok"), &hash);

    println!("Welcome on rust m3ss4g1ng by ESGI");
    general_menu();   
}

mod unit_testing {
    use super::*;
    
    #[test]
    fn test_get_pseudo() {
        let user = User::create_user(String::from("toto"), String::from(encode_pwd(String::from("toto"))));
        assert_eq!(user.get_pseudo().to_string(), String::from("toto"));
    }
}
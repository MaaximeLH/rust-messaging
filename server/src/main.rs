use std::{panic, thread};
use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::io::{Read, Write};
use std::str::from_utf8;
use argon2::{self, Config};


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
            pwd: encode_pwd(pwd)
        }
    }

    /// Function to update the socket of a user.
    fn update_socket(&mut self, socket: TcpStream) {
        self.socket = socket;
    }

    fn serialize(&self) -> String {
        // return String::from("User: {}, pwd: {}, socket: {}", self.pseudo, self.pwd, self.socket);
        let mut serie:String = String::from("User: \"");
        serie.push_str(self.pseudo.as_str());
        serie.push_str("\", pwd: \"");
        serie.push_str(self.pwd.as_str());
        serie.push_str("\"");
    
        return serie;
    }

    fn display() {
        println!("test");
    }
}


fn main() {
    let hash = encode_pwd(String::from("test"));
    verify_pwd(String::from("test"), &hash);
    verify_pwd(String::from("ok"), &hash);

    println!("Welcome on rust m3ss4g1ng by ESGI");
    general_menu();

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
/// 
/// Returns true if match, else false.
fn verify_pwd(pwd:String, hash:&String) -> bool {
    return argon2::verify_encoded(&hash, pwd.as_bytes()).unwrap();
}

fn general_menu() {
    loop {
        println!("What do you want to do ?");
        println!("1- Connect");
        println!("2- Register");
        println!("!q- Quit");
        let entry:String = read_user_entry();
        
        if entry == "!q" || entry == "!quit" {
            println!("Quit");
            break;
        }

        if entry.parse::<i8>().is_err() {
            continue;
        }

        let entry:i8 = entry.parse().unwrap();

        if entry != 1 && entry != 2 {
            println!("Not a good option: {}", entry);
        }

        if entry == 1 {
            connect();
        } else  if entry == 2{
            register();
            break;
        }
    }  
}

fn connect() {
    println!("Connect");
    print!("Enter username: ");
    let user:String = read_user_entry();
    print!("Enter password: ");
    let pwd:String = read_user_entry();
    let stream = TcpStream::connect("192.168.1.47:13796").expect("Can't connect to server");
    
    let mut user = User::create_user(user, pwd.clone(), stream);

    let hash = user.get_pwd();

    println!("{}", verify_pwd(pwd, hash));
    println!("{}", user.serialize());


    let mut user_list:Vec<User> = Vec::new();
    user_list.push(user);

    // println!("{}", user_list);
}

fn register() {
    println!("Register");
}
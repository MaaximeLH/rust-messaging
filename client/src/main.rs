// use std::{panic, thread};
// use std::net::{Shutdown, TcpListener, TcpStream, ToSocketAddrs};
use std::{io::{Write, stdout}, process::exit};
use argon2::{self, Config};
use json::{self, JsonValue, object};
use regex;


struct User {
    /// The pseudo the user will use inside the chat.
    pseudo: String,
    /// The user's password to authenticate on the chat.
    pwd: String,
}

struct Message {
    from: User,
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

    /// Function to create a new User.
    /// Returns an instance of User Structure
    fn create_user(pseudo: String, pwd: String) -> User {
        User {
            pseudo,
            pwd: encode_pwd(pwd),
        }
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

impl Message {
    fn new(user: User, content:String) -> Message {
        Message{
            from: user,
            content
        }
    }

    fn send() {
        // TODO: envoyer le message au server
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
            println!("Not a good option: {}", entry);
            continue;
        }

        let entry:i8 = entry.parse().unwrap();

        if entry != 1 && entry != 2 {
            println!("Not a good option: {}", entry);
            continue;
        }

        let user:User;

        if entry == 1 {
            user = connect();
        } else  /*if entry == 2*/{
            user = register();
        }

        chat_menu(user);
    }  
}

fn connect() -> User {
    println!("");
    println!("--------------------");
    print!("Enter username: ");
    let pseudo:String = read_user_entry();
    print!("Enter password: ");
    let pwd:String = read_user_entry();
    println!("--------------------");
    println!("");
    
    let user = User::create_user(pseudo, pwd.clone());

    // let hash = user.get_pwd();
    // println!("{}", verify_pwd(pwd, hash));
    // println!("{}", user.to_json());
    // println!("{}", user.to_string());

    //JSON extract field example: (json::parse(user.to_json()as_str())).unwrap
    // println!("{}", (json::parse(user.to_json().as_str())).unwrap()["username"]);

    // TODO: Send json over socket to verify user on server
    // TODO: Get the return of the server to verify if user exist and the password is good
    
    return user;
}

fn verify_pseudo(_pseudo: String) -> bool {
    // TODO: send the pseudo in json and get the return from server
    // Return true if pseudo is available, else false
    return true;
}

fn register() -> User{
    println!("Register");

    print!("Enter username: ");
    let pseudo:String = read_user_entry();
    
    if !verify_pseudo(pseudo.clone()) {
        println!("Pseudo already used");
        exit(1);
    }

    print!("Enter password: ");
    let pwd:String = read_user_entry();

    let user = User::create_user(pseudo, pwd);

    // TODO: send user json to server to register the user (use user.to_json() -> json)

    return user;
}

fn chat_menu(user: User) {
    println!("Welcome {}", user.get_pseudo());

    loop {
        println!("1- Enter in general chat");
        println!("!q- Quit");

        let entry = read_user_entry();

        if entry == "!q" || entry == "!quit" {
            println!("Quit");
            break;
        }

        if entry.parse::<i8>().is_err() {
            println!("Not a good option: {}", entry);
            continue;
        }

        let entry:i8 = entry.parse().unwrap();

        if entry != 1 {
            println!("Not a good option: {}", entry);
            continue;
        } else {
            // println!("Chat !!");
            // break;
            chat(String::from("general"));
        }
    }
}

fn chat(chat_type:String) {
    if chat_type == "general" {
        println!("");
        println!("");
        println!("");
        println!("--- General chat ---");
        println!("[PUBLCI] M4ss1m0: Vive l'ESGI");
        println!("[PUBLIC] D0rus: Vive le BDE de l'ESGI");

        let regex = regex::Regex::new("^![A-z0-9]+( [A-z0-9]*){0,1}$").unwrap();
        let entry = read_user_entry();

        if regex.is_match(entry.as_str()) {
            // let command:Option<usize> = entry.find(" ");

            let str_splitted = entry.split(" ");
            let vec = str_splitted.collect::<Vec<&str>>();
            let action = vec[0].split("!").collect::<Vec<&str>>()[1];
            // println!("{}", action);

            match action {
                "quit" | "q" => {
                    println!("You will quit");
                    //TODO: quit the thread
                },
                "l" | "list" => println!("You will list"),
                "p" | "private" => println!("You will talk in private to user"),
                _ => println!("Another stuff")
            }

        } else {
            println!("false");
        }
    }
}
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

impl Message {
    /// Create a new message.
    fn new(user: User, to:String, content:String) -> Message {
        Message{
            from: user,
            to,
            content
        }
    }

    /// Send the current message to the server.
    fn send(&self) {
        // TODO: envoyer le message au server
        println!("Sending message to server");
        println!("Message sent");
        println!("From:\"{}\", content:\"{}\"", self.from.get_pseudo(), self.content);
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
    println!("");
    println!("--------------------");
    print!("Enter username: ");
    let pseudo:String = read_user_entry();
    print!("Enter password: ");
    let pwd:String = read_user_entry();
    println!("--------------------");
    println!("");
    
    let user = User::create_user(pseudo, pwd);

    // let hash = user.get_pwd();
    // println!("{}", verify_pwd(pwd, hash));
    // println!("{}", user.to_json());
    // println!("{}", user.to_string());

    //JSON extract field example: (json::parse(user.to_json()as_str())).unwrap
    // println!("{}", (json::parse(user.to_json().as_str())).unwrap()["username"]);

    // TODO: Send json over socket to verify user on server
    // TODO: Get the return of the server to verify if user exist and the password is good
    
    return (true, user);
}

fn verify_pseudo(_pseudo: String) -> bool {
    // TODO: send the pseudo in json and get the return from server
    // Return true if pseudo is available, else false
    return true;
}

fn register() -> (bool, User) {
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

    return (true, user);
}

fn get_connected_users() -> String {

    //TODO: get all connected users from server

    let users = String::from("{\"users\":[\"toto\",\"marco\",\"massimo\",\"dorus\"]}");
    return users;
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
        println!("");
        println!("");
        println!("");
        println!("--- General chat ---");
        println!("[PUBLIC] M4ss1m0: Vive l'ESGI");
        println!("[PUBLIC] D0rus: Vive le BDE de l'ESGI");

        let regex = regex::Regex::new("^![A-z0-9]+( [A-z0-9]*){0,1}$").unwrap();
        let entry = read_user_entry();

        if regex.is_match(entry.as_str()) {
            let str_splitted = entry.split(" ");
            let vec = str_splitted.collect::<Vec<&str>>();
            let action = vec[0].split("!").collect::<Vec<&str>>()[1];

            match action {
                "q" | "quit" => {
                    println!("You will quit");
                    //TODO: quit the thread
                },
                "l" | "list" => {
                    println!("You will list");
                    let users = get_connected_users();
                    let users = json::parse(users.as_str()).unwrap_or(object!{});
                    println!("{:?}", users);
                    for x in users["users"].members() {
                        println!("{}", x);
                    }
                },
                 "!h" | "!help" => {
                display_help();
                //continue;
            }
                "p" | "private" => println!("You will talk in private to user"),
                _ => println!("Another stuff")
            }

        } else {
            println!("this is a standard message");
            let message:Message = Message::new(user.to_user(), String::from("general"), entry);
            message.send();
        }
    }
}
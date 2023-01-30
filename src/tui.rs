use crate::mail_app::{MailApp, MailId, Session, SessionId};
use std::{io::Write, rc::Rc};

fn read_line() -> String {
    std::io::stdout().flush().expect("couldn't flush stdout");
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("couldn't read line from stdin");
    buffer.trim_end().to_string()
}

fn ask_yes_or_no_question(question: String) -> bool {
    print!("{question} [y/n] ");
    loop {
        match read_line().as_str() {
            "Y" | "y" => break true,
            "N" | "n" => break false,
            answer => {
                println!("\"{answer}\" is not a valid answer to \"{question}? [y/n]\", try again",);
                print!("{question} [y/n]");
            }
        }
    }
}

enum Command {
    Logout,
    ListMails,
    ListUnreadMails,
    ReadMail(MailId),
    WriteMail,
    ReplyToMail(MailId),
    Help,
    CommandHelp(String),
}

impl Command {
    pub fn parse(command: &str) -> Result<Command, String> {
        let tokens_vec = Self::tokenize(command);
        let mut tokens = tokens_vec.iter().map(|s| s.as_str());
        match tokens.next() {
            Some("logout") => Self::last(Command::Logout, tokens.next()),
            Some("list") => match tokens.next() {
                Some("mails") => Self::last(Command::ListMails, tokens.next()),
                Some("unread") => Self::last(Command::ListUnreadMails, tokens.next()),
                None => Ok(Command::ListUnreadMails),
                Some(s) => Err(format!(
                    r#"expected "mails", "unread" or nothing, got "{s}""#
                )),
            },
            Some("read") => match tokens.next() {
                Some(value_string) => match value_string.parse::<MailId>() {
                    Ok(value) => Self::last(Command::ReadMail(value), tokens.next()),
                    Err(error_string) => Err(format!(
                        r#"invalid mail id "{value_string}": {error_string}"#
                    )),
                },
                None => Err("expected mail id, got nothing".to_string()),
            },
            Some("write") => Self::last(Command::WriteMail, tokens.next()),
            Some("reply") => match tokens.next() {
                Some(value_string) => match value_string.parse::<MailId>() {
                    Ok(value) => Self::last(Command::ReplyToMail(value), tokens.next()),
                    Err(error_string) => Err(format!(
                        r#"invalid mail id "{value_string}": {error_string}"#
                    )),
                },
                None => Err("expected mail id, got nothing".to_string()),
            },
            Some("help") => match tokens.next() {
                Some(c @ ("logout" | "list" | "read" | "write" | "reply" | "help")) => {
                    Self::last(Command::CommandHelp(c.to_string()), tokens.next())
                }
                None => Ok(Command::Help),
                Some(s) => Err(format!(r#"expected a command, got "{s}""#)),
            },
            Some(s) => Err(format!(r#"unrecognized command: "{s}""#)),
            None => Err("expected command, got nothing".to_string()),
        }
    }

    fn last(command: Command, next: Option<&str>) -> Result<Command, String> {
        match next {
            None => Ok(command),
            Some(s) => Err(format!(r#"expected nothing, got "{s}""#)),
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        let mut chars = text.chars();
        let mut tokens = Vec::<String>::new();
        loop {
            match chars.next() {
                Some(c) => {
                    let mut value = String::new();
                    value.push(c);
                    loop {
                        match chars.next() {
                            Some(' ') | None => break tokens.push(value),
                            Some(c) => value.push(c),
                        }
                    }
                }
                None => break tokens,
            }
        }
    }
}

pub struct Tui {
    mail_app: MailApp,
}

impl Tui {
    pub fn new(mail_app: MailApp) -> Self {
        Self { mail_app }
    }

    pub fn run(mut self) {
        println!("Mail app  in rust 🦀🔥 based 🦀🔥");
        loop {
            let session = self.login();
            self.run_cli(session);
        }
    }

    pub fn login(&mut self) -> SessionId {
        println!();
        print!("Username: ");
        let username = read_line();
        print!("Password: ");
        let password = read_line();
        match self.mail_app.login(username.clone(), password) {
            Ok(session) => session,
            Err(crate::mail_app::LoginError::WrongPassword) => {
                println!("Wrong username/password");
                self.login()
            }
            Err(crate::mail_app::LoginError::UserDoesntExist) => {
                println!("No user with username \"{username}\" exists",);
                if ask_yes_or_no_question("Would you like to create one?".to_string()) {
                    self.register();
                }
                self.login()
            }
        }
    }

    fn register(&mut self) {
        println!();
        println!("Creating new user");
        print!("Username: ");
        let username = read_line();
        print!("Password: ");
        let password = read_line();
        vec![
            "Email",
            "Birthday",
            "Credit card number",
            "Expiry date",
            "The three digits on the back (more information)",
            "Social security number",
            "Mothers maiden name",
            "League of legends username",
        ]
        .iter()
        .for_each(|question| {
            print!("{question}: ");
            let _ = read_line();
        });
        match self.mail_app.register(username.clone(), password) {
            Ok(_) => {
                println!("Created user with username \"{username}\"")
            }
            Err(crate::mail_app::RegisterError::UserAlreadyExists) => {
                println!("User with username \"{username}\" already exists.");
                if ask_yes_or_no_question("Would you like to try again?".to_string()) {
                    self.register()
                }
            }
        }
    }

    fn run_cli(&mut self, session: SessionId) {
        loop {
            println!();
            println!(r#"Type "help" for a list of commands"#);
            print!("> ");
            match Command::parse(read_line().as_str()) {
                Ok(Command::Logout) => break self.mail_app.logout(session),
                Ok(Command::ListMails) => {
                    println!("id\tsender\nsubject");
                    for mail in self.mail_app.list_mails(session) {
                        let info = self.mail_app.mail_info(mail).expect("mail not found");
                        println!("{}\n{}\n{}", info.id, info.sender, info.subject);
                    }
                }
                Ok(Command::ListUnreadMails) => {
                    println!("id\tsender\tsubject");
                    for mail in self.mail_app.list_unread_mails(session) {
                        let info = self.mail_app.mail_info(mail).expect("mail not found");
                        println!("{}\t{}\t{}", info.id, info.sender, info.subject);
                    }
                }
                Ok(Command::ReadMail(mail_id)) => match self.mail_app.mail_info(mail_id) {
                    Some(mail_info) => {
                        println!(
                            "\nmail id: {}\nsender: {}\nsubject: {}\n",
                            mail_info.id, mail_info.sender, mail_info.subject
                        );
                        let content = self.mail_app.read_mail(mail_id).expect("mail not found");
                        println!("{content}");
                    }
                    None => println!("mail not found"),
                },
                Ok(Command::WriteMail) => {
                    print!("subject: ");
                    let subject = read_line();
                    print!("reciever: ");
                    let reciever = read_line();
                    println!("Write mail content, when done type type \"END\" on a blank line");
                    let mut content = String::new();
                    loop {
                        match read_line().as_str() {
                            "END" => break,
                            line => content.push_str(line),
                        }
                    }
                    self.mail_app
                        .write_mail(session, reciever, subject, content)
                }
                Ok(Command::ReplyToMail(_)) => println!("not implemented, sorry"),
                Ok(Command::Help) => self.print_help(),
                Ok(Command::CommandHelp(command)) => self.print_command_help(command.as_str()),
                Err(message) => println!("parser error: {message}"),
            }
        }
    }

    fn print_help(&self) {
        println!("These are all available commands\n");
        println!("  logout              - logs user out of current session");
        println!("  list (mails|unread) - list all or unread mails");
        println!("  read <mail id>      - read mail");
        println!("  write               - write mail");
        println!("  reply <mail id>     - reply to mail");
        println!("  help <command>?     - prints this message or a command specific message");
    }

    fn print_command_help(&self, command: &str) {
        match command {
            "logout" => {
                println!("  logout - logs user out of the current session");
            }
            "list" => {
                println!("  list");
                println!("  list mails  - -||-");
                println!("  list unread - list all unread mails");
            }
            "read" => {
                println!(r#"  read <mail id> - read mail, get mail id using "list""#);
            }
            "write" => {
                println!(r#"  write - start writing a new mail"#);
            }
            "reply" => {
                println!(r#"  reply <mail id> - reply to mail, get mail id using "list""#);
            }
            "help" => {
                println!(r#"  help           - prints this message"#);
                println!(r#"  help <command> - prints a specific message for a command"#);
            }
            _ => panic!(),
        }
    }
}

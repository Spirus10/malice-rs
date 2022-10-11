use std::fmt;
use std::process;
use std::error::Error;

use crate::listener::Listener;
use crate::logger;

// ------------------- Command Error -----------------------------


// CommandError must derive Debug to implement Error
#[derive(Debug)]
pub struct CommandError {
    details: String
}

impl CommandError {
    pub fn new(msg: &str) -> CommandError {
        CommandError{details: msg.to_string()}
    }
    pub fn unwrap(&self) -> &str {
        return &self.details.as_str()
    }
}

impl Error for CommandError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

// ------------------- Command Error -----------------------------

// ------------------- Command Datatype and Handling -------------

#[derive(Debug)]
pub enum CommandType {
    Listener(String),
    Exit,
    Unknown
}

#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType
}

impl Command {

    pub fn from_string(cmd: String) -> Result<Command, Box<dyn Error>> {

        let mut command_words = cmd.split(" ");

        if let Some(t) = command_words.nth(0) {
            let command_string: String = String::from(
                command_words.collect::<Vec<&str>>()
                .as_slice()
                .join::<&str>(" ")
            );
            let command_type: CommandType = match t {
                "listener" => CommandType::Listener(command_string),
                "exit" => CommandType::Exit,
                _ => CommandType::Unknown,
            };

            return Ok( Command { command_type: command_type } );
        }

        else { return Err(Box::new(CommandError::new("Failed to parse command!"))); }


    }

    pub fn handle(&self) -> Result<(), CommandError>{

        match &self.command_type {
            CommandType::Listener(s) => {

                if s == "" {
                    logger::info("Usage: listener <name> <host> <port> <uri> <password>");
                    return Ok(());
                }

                let mut args = s.split(" ");

                let name = args.nth(0).unwrap().to_owned();
                let address = args.nth(0).unwrap().to_owned();
                let port = args.nth(0).unwrap().parse().unwrap();
                let uri = args.nth(0).unwrap().to_owned();
                let password = args.nth(0).unwrap().to_owned();

                Listener::new(&name, &address, port, &uri, &password);
                Ok(())
            },
            CommandType::Exit => { process::exit(0); }
            CommandType::Unknown => {
                Err(CommandError::new("Oh no! Command type unknown!"))
            },
        }

    }
}

// ------------------- Command Datatype and Handling -------------
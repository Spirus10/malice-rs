use std::io;
use std::io::Write;
use std::error::Error;

mod listener;
mod logger;
mod command;

fn main() -> Result<(), Box<dyn Error>> {

    loop {

        print!("[malice]> ");
        io::stdout().flush().unwrap();
        let mut cmd: String = String::new();

        let cmd_size: usize = std::io::stdin().read_line(&mut cmd).unwrap();
        cmd.pop();

        if cmd_size == 1 { continue; }

        else { 

            let cmd = command::Command::from_string(cmd)?;
            match cmd.handle() {
                Ok(_) => continue,
                Err(e) => logger::bad(e.unwrap()),
            }
        }
    } 
}

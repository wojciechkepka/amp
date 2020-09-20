use super::parser::parse_program;
use std::io::{self, Write};

pub struct AmpCli {}
impl AmpCli {
    pub fn run() {
        println!("AmpCli v0.1.0");
        loop {
            print!("=> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            if let Err(e) = io::stdin().read_line(&mut input) {
                println!("Invalid input line '{}' - '{}'", input, e);
                continue;
            }

            println!("{:?}", parse_program(&input))
        }
    }
}

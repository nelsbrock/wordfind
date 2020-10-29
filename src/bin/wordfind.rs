use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::{env, io};

use wordfind::commands::Command;
use wordfind::filters::Word;

fn main() -> Result<(), String> {
    let env_argc = env::args().len();
    if env_argc != 2 {
        return Err(format!(
            "Invalid number of arguments (expected 1, found {})",
            env_argc - 1
        ));
    }

    let filepath = env::args().nth(1).unwrap();
    let words = match read_words(&*filepath) {
        Ok(words) => words,
        Err(err) => return Err(format!("Error reading dictionary file: {}", err)),
    };

    let mut last_cmd: Option<Command> = None;

    loop {
        eprint!("> ");
        io::stderr().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(len) => {
                if len == 0 {
                    return Ok(());
                }
            }
            Err(err) => return Err(format!("Error reading stdin: {}", err)),
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let command = match Command::parse(input, last_cmd.as_ref()) {
            Some(command) => command,
            None => {
                eprintln!("Invalid command.");
                continue;
            }
        };

        last_cmd = Some(command);

        let result = last_cmd.as_ref().unwrap().result(words.iter());
        for r in result {
            println!("{}", r.iter().collect::<String>());
        }

        eprintln!();
    }
}

fn read_words(path: &str) -> std::io::Result<Vec<Word>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut words = Vec::new();
    for word in reader.lines() {
        match word {
            Ok(word) => words.push(word.chars().flat_map(|c| c.to_lowercase()).collect()),
            Err(err) => return Err(err),
        }
    }

    Ok(words)
}

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use wordfind::commands::Command;

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

    let mut rl = Editor::<()>::new();

    loop {
        let input = match rl.readline("> ") {
            Ok(mut line) => {
                line = line.trim().to_string();
                rl.add_history_entry(&line);
                line
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err.to_string()),
        };

        if input.is_empty() {
            continue;
        }

        let command = match Command::parse(&input, last_cmd.as_ref()) {
            Ok(command) => command,
            Err(err) => {
                eprintln!("Error: {}", err);
                eprintln!();
                continue;
            }
        };

        last_cmd = Some(command);

        let result = last_cmd.as_ref().unwrap().result(words.iter());
        for r in result {
            println!("{}", r.iter().collect::<String>());
        }

        println!();
    }

    Ok(())
}

fn read_words(path: &str) -> std::io::Result<Vec<Vec<char>>> {
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

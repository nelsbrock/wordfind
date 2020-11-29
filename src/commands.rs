use std::{fmt, slice};

use crate::filters::{Filter, LenFilter, MatchFilter, SeqFilter};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug)]
pub struct CommandParseError {
    description: String,
}

impl Display for CommandParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for CommandParseError {}

impl CommandParseError {
    fn new(description: String) -> CommandParseError {
        CommandParseError { description }
    }
}

pub struct CommandResult<'a> {
    command: &'a Command,
    words: slice::Iter<'a, Vec<char>>,
}

impl<'a> Iterator for CommandResult<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(match_word) = self.words.next() {
            if self.command.filters.iter().all(|f| f.check(match_word)) {
                return Some(match_word);
            }
        }
        None
    }
}

pub struct Command {
    filters: Vec<Rc<dyn Filter>>,
}

impl Command {
    fn new(filters: Vec<Rc<dyn Filter>>) -> Self {
        Self { filters }
    }

    pub fn parse(cstr: &str, last: Option<&Self>) -> Result<Self, CommandParseError> {
        let mut filters: Vec<Rc<dyn Filter>> = Vec::new();

        for (part_i, part) in cstr.split_ascii_whitespace().enumerate() {
            let mut part_c = part.chars();
            let modifier = part_c.next().unwrap(); // part cannot be empty

            if modifier == '%' {
                // parse history reference
                let index = part_c.as_str();
                let index = if index == "%" {
                    part_i
                } else {
                    match index.parse() {
                        Ok(index) => index,
                        Err(_) => {
                            return Err(CommandParseError::new(format!(
                                "unable to parse history reference `{}`",
                                part
                            )))
                        }
                    }
                };

                let last = match last {
                    Some(last) => last,
                    None => {
                        return Err(CommandParseError::new(format!(
                            "history reference `{}` without prior command",
                            part
                        )))
                    }
                };

                if index >= last.filters.len() {
                    return Err(CommandParseError::new(format!(
                        "invalid history reference `{}` (index out of bounds)",
                        part
                    )));
                }
                filters.push(Rc::clone(&last.filters[index]));
            } else {
                // parse filter
                match SeqFilter::from_str(part) {
                    Ok(filter) => filters.push(Rc::new(filter)),
                    Err(e_seq) => match LenFilter::from_str(part) {
                        Ok(filter) => filters.push(Rc::new(filter)),
                        Err(e_len) => match MatchFilter::from_str(part) {
                            Ok(filter) => filters.push(Rc::new(filter)),
                            Err(e_match) => {
                                return Err(CommandParseError::new(format!(
                                    "unable to parse filter string `{}`. \
                                (not a sequence filter: {}; \
                                 not a length filter: {}; \
                                 not a match filter: {})",
                                    part, e_seq, e_len, e_match
                                )))
                            }
                        },
                    },
                };
            }
        }

        Ok(Command::new(filters))
    }

    pub fn result<'a>(&'a self, words: slice::Iter<'a, Vec<char>>) -> CommandResult<'a> {
        CommandResult {
            command: self,
            words,
        }
    }
}

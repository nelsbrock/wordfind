use std::slice;

use crate::filters::{Filter, LenFilter, MatchFilter, SeqFilter, Word};
use std::rc::Rc;

type Filters = Vec<Rc<dyn Filter>>;

pub struct CommandResult<'a> {
    command: &'a Command,
    words: slice::Iter<'a, Word>,
}

impl<'a> Iterator for CommandResult<'a> {
    type Item = &'a Word;

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
    filters: Filters,
}

impl Command {
    fn new(filters: Filters) -> Self {
        Self { filters }
    }

    pub fn parse(cstr: &str, last: Option<&Self>) -> Option<Self> {
        let mut filters: Filters = Vec::new();

        for (part_i, part) in cstr.split_ascii_whitespace().enumerate() {
            let (modifier, index) = part.split_at(1);

            if modifier == "%" {
                // parse history reference
                let index = if index == "%" {
                    part_i
                } else {
                    match index.parse() {
                        Ok(index) => index,
                        Err(_) => return None,
                    }
                };

                let last = last?;
                if index >= last.filters.len() {
                    return None;
                }
                filters.push(Rc::clone(&last.filters[index]));
            } else {
                // parse filter
                match SeqFilter::parse(part) {
                    Some(filter) => filters.push(Rc::new(filter)),
                    None => match LenFilter::parse(part) {
                        Some(filter) => filters.push(Rc::new(filter)),
                        None => filters.push(Rc::new(MatchFilter::parse(part)?)),
                    },
                };
            }
        }

        Some(Command::new(filters))
    }

    pub fn result<'a>(&'a self, words: slice::Iter<'a, Word>) -> CommandResult<'a> {
        CommandResult {
            command: self,
            words,
        }
    }
}

use std::slice;

use crate::filters::{Filter, LenFilter, MatchFilter, SeqFilter, Word};

pub struct CommandResult<'a> {
    filters: Vec<Box<dyn Filter>>,
    words: slice::Iter<'a, Word>,
}

impl<'a> Iterator for CommandResult<'a> {
    type Item = &'a Word;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(match_word) = self.words.next() {
            if self.filters.iter().all(|f| f.check(match_word)) {
                return Some(match_word);
            }
        }
        None
    }
}

pub struct Command {
    filters: Vec<Box<dyn Filter>>,
}

impl Command {
    fn new(filters: Vec<Box<dyn Filter>>) -> Self {
        Self { filters }
    }

    pub fn parse(cstr: &str) -> Option<Self> {
        let mut filters: Vec<Box<dyn Filter>> = Vec::new();

        for part in cstr.split_ascii_whitespace() {
            match SeqFilter::parse(part) {
                Some(filter) => filters.push(Box::new(filter)),
                None => match LenFilter::parse(part) {
                    Some(filter) => filters.push(Box::new(filter)),
                    None => filters.push(Box::new(MatchFilter::parse(part)?)),
                },
            };
        }

        Some(Command::new(filters))
    }

    pub fn result(self, words: slice::Iter<Word>) -> CommandResult {
        CommandResult {
            filters: self.filters,
            words,
        }
    }
}

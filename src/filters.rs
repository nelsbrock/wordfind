use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug)]
pub struct FilterParseError {
    description: String,
}

impl Display for FilterParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for FilterParseError {}

impl FilterParseError {
    fn new(description: String) -> FilterParseError {
        FilterParseError { description }
    }
}

pub(crate) trait Filter {
    fn check(&self, word: &[char]) -> bool;
}

pub(crate) struct MatchFilter {
    chars: HashMap<char, usize>,
    wildcards: usize,
}

impl MatchFilter {
    fn new(chars: HashMap<char, usize>, wildcards: usize) -> Self {
        Self { chars, wildcards }
    }
}

impl FromStr for MatchFilter {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = HashMap::new();
        let mut wildcards = 0;
        for c in s.chars().flat_map(|c| c.to_lowercase()) {
            match c {
                '*' => wildcards += 1,
                c => {
                    let old = *chars.get(&c).unwrap_or(&0);
                    chars.insert(c, old + 1);
                }
            }
        }
        Ok(MatchFilter::new(chars, wildcards))
    }
}

impl Filter for MatchFilter {
    fn check(&self, word: &[char]) -> bool {
        let mut chars = self.chars.clone();
        let mut wildcards = self.wildcards;
        for wc in word {
            let old = *chars.get(wc).unwrap_or(&0);
            if old > 0 {
                chars.insert(*wc, old - 1);
            } else if wildcards > 0 {
                wildcards -= 1;
            } else {
                return false;
            }
        }
        true
    }
}

enum ComparisonType {
    Eq,
    Lt,
    Gt,
    Le,
    Ge,
}

pub(crate) struct LenFilter {
    ctype: ComparisonType,
    clen: usize,
}

impl LenFilter {
    fn new(ctype: ComparisonType, clen: usize) -> Self {
        Self { ctype, clen }
    }
}

impl FromStr for LenFilter {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ComparisonType::*;

        let fchars: Vec<char> = s.chars().collect();

        if s.is_empty() {
            return Err(FilterParseError::new("empty string".into()));
        }

        let ctype;
        let clen_start;
        if fchars[0] == '=' {
            ctype = Eq;
            clen_start = 1;
        } else if fchars[0] == '<' {
            if fchars[1] == '=' {
                ctype = Le;
                clen_start = 2;
            } else {
                ctype = Lt;
                clen_start = 1;
            }
        } else if fchars[0] == '>' {
            if fchars[1] == '=' {
                ctype = Ge;
                clen_start = 2;
            } else {
                ctype = Gt;
                clen_start = 1;
            }
        } else {
            return Err(FilterParseError::new("invalid comparison operator".into()));
        }
        let clen = match s.split_at(clen_start).1.parse() {
            Ok(clen) => clen,
            Err(_) => return Err(FilterParseError::new("invalid length".into())),
        };

        Ok(Self::new(ctype, clen))
    }
}

impl Filter for LenFilter {
    fn check(&self, word: &[char]) -> bool {
        use ComparisonType::*;
        let len = word.len();
        match self.ctype {
            Eq => len == self.clen,
            Lt => len < self.clen,
            Gt => len > self.clen,
            Le => len <= self.clen,
            Ge => len >= self.clen,
        }
    }
}

pub(crate) struct SeqFilter {
    start: usize,
    seq: Vec<char>,
    min_word_len: usize,
}

impl SeqFilter {
    fn new(start: usize, seq: Vec<char>) -> Self {
        let min_word_len = start + seq.len();
        Self {
            start,
            seq,
            min_word_len,
        }
    }
}

impl FromStr for SeqFilter {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fparts = s.splitn(2, ':');
        let start = fparts.next().unwrap(); // splitn always returns at least one item
        let start = match start.len() {
            0 => 0,
            _ => match start.parse() {
                Ok(start) => start,
                Err(_) => return Err(FilterParseError::new("invalid start index".into())),
            },
        };
        let seq = match fparts.next() {
            Some(seq) => seq,
            None => return Err(FilterParseError::new("missing sequence".into())),
        };

        Ok(Self::new(start, seq.chars().collect()))
    }
}

impl Filter for SeqFilter {
    fn check(&self, word: &[char]) -> bool {
        word.len() >= self.min_word_len
            && word
                .iter()
                .skip(self.start)
                .zip(&self.seq)
                .all(|(wc, sc)| wc == sc)
    }
}

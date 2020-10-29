use std::collections::HashMap;

pub type Word = Vec<char>;

pub(crate) trait Filter {
    fn parse(fstr: &str) -> Option<Self>
    where
        Self: Sized;
    fn check(&self, word: &Word) -> bool;
}

pub(crate) struct MatchFilter {
    chars: HashMap<char, usize>,
}

impl MatchFilter {
    fn new(chars: HashMap<char, usize>) -> Self {
        Self { chars }
    }
}

impl Filter for MatchFilter {
    fn parse(fstr: &str) -> Option<Self> {
        let mut chars = HashMap::new();
        for c in fstr.chars().flat_map(|c| c.to_lowercase()) {
            let old = *chars.get(&c).unwrap_or(&0);
            chars.insert(c, old + 1);
        }
        Some(MatchFilter::new(chars))
    }

    fn check(&self, word: &Word) -> bool {
        let mut chars = self.chars.clone();
        for wc in word {
            let old = *chars.get(wc).unwrap_or(&0);
            if old > 0 {
                chars.insert(*wc, old - 1);
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

impl Filter for LenFilter {
    fn parse(fstr: &str) -> Option<Self> {
        use ComparisonType::*;

        let fchars: Vec<char> = fstr.chars().collect();

        if fstr.is_empty() {
            return None;
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
            return None;
        }
        let clen = match fstr.split_at(clen_start).1.parse() {
            Ok(clen) => clen,
            Err(_) => return None,
        };

        Some(Self::new(ctype, clen))
    }

    fn check(&self, word: &Word) -> bool {
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
    seq: Word,
}

impl SeqFilter {
    fn new(start: usize, seq: Word) -> Self {
        Self { start, seq }
    }
}

impl Filter for SeqFilter {
    fn parse(fstr: &str) -> Option<Self> {
        let mut fparts = fstr.splitn(2, ':');
        let start = fparts.next()?;
        let start = match start.len() {
            0 => 0,
            _ => match start.parse() {
                Ok(start) => start,
                Err(_) => return None,
            },
        };
        let seq = fparts.next()?;

        Some(Self::new(start, seq.chars().collect()))
    }

    fn check(&self, word: &Word) -> bool {
        word.len() >= self.start + self.seq.len()
            && word
                .iter()
                .skip(self.start)
                .zip(self.seq.iter())
                .all(|(wc, sc)| wc == sc)
    }
}

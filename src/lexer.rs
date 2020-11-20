#[derive(Clone, Copy, Debug)]
pub enum Token<'a> {
    Ident(&'a str),
    Label(&'a str),
    Immediate(u16),
    Register(u8),
    Comma,
    Unknown(&'a str),
}

fn label_chars(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn take_while<P>(source: &str, mut predicate: P) -> (&str, &str)
    where P: FnMut(char) -> bool
{
    for (i, c) in source.char_indices() {
        if !predicate(c) {
            return source.split_at(i);
        }
    }
    (source, "")
}

pub struct Lexer<'a> {
    remain: &'a str
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { remain: source }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.remain = self.remain.trim();
        let mut iter = self.remain.char_indices();
        
        // Get into the good al lexin state
        let (token, r) = match iter.next() {
            Some((_, 'r')) | Some((_, 'R')) => {
                let (reg, remain) = take_while(&self.remain[1..], |c| c.is_ascii_digit());
                if let Ok(reg) = reg.parse::<u8>() {
                    (Token::Register(reg), remain)
                } else {
                    let (string, remain) = take_while(self.remain, label_chars);
                    if let Some(':') = remain.chars().next() {
                        (Token::Label(string), &remain[1..])
                    } else {
                        (Token::Ident(string), remain)
                    }
                }
            },
            
            Some((_, ',')) => {
                (Token::Comma, &self.remain[1..])
            }
            
            Some((_, '/')) => {
                if let Some((_, '/')) = iter.next() {
                    let (_, r) = take_while(self.remain, |_| true /* c != '\n' */);
                    self.remain = r;
                    (self.next()?, self.remain)
                } else if let Some((i, _)) = iter.next() {
                    let (t, r) = self.remain.split_at(i);
                    (Token::Unknown(t), r)
                } else {
                    (Token::Unknown(self.remain), "")
                }
            }
            
            Some((_, c)) if label_chars(c) => {
                let (string, remain) = take_while(self.remain, label_chars);
                if let Some(':') = remain.chars().next() {
                    (Token::Label(string), &remain[1..])
                } else {
                    (Token::Ident(string), remain)
                }
            },
            
            Some((_, c)) if c.is_ascii_digit() => {
                let (i, remain) = take_while(self.remain, |c| c.is_ascii_digit());
                let immediate = i.parse::<u16>().map(Token::Immediate).unwrap_or(Token::Unknown(i));
                (immediate, remain)
            },
            
            Some((_, _)) => {
                if let Some((i, _)) = iter.next() {
                    let (t, r) = self.remain.split_at(i);
                    (Token::Unknown(t), r)
                } else {
                    (Token::Unknown(self.remain), "")
                }
                
            },
            
            None => return None
        };
        
        self.remain = r;
        Some(token)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Token<'a> {
    Ident(&'a str),
    Immediate(u8),
    Register(u8),
    Comma,
    Unknown(&'a str),
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
                let register = reg.parse::<u8>().map(Token::Register).unwrap_or(Token::Unknown(reg));
                (register, remain)
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
            
            Some((_, c)) if c.is_ascii_alphabetic() => {
                let (t, c) = take_while(self.remain, |c| c.is_ascii_alphabetic());
                (Token::Ident(t), c)
            },
            
            Some((_, c)) if c.is_ascii_digit() => {
                let (i, remain) = take_while(self.remain, |c| c.is_ascii_digit());
                let immediate = i.parse::<u8>().map(Token::Immediate).unwrap_or(Token::Unknown(i));
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

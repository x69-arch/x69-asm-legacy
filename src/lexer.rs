#[derive(Clone, Copy, Debug)]
pub enum Token<'a> {
    Ident(&'a str),
    Register(u8),
    Comma,
    Comment(&'a str),
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
            Some((_, 'r')) => {
                let (reg, remain) = take_while(&self.remain[1..], |c| c.is_ascii_digit());
                let register = reg.parse::<u8>().map(Token::Register).unwrap_or(Token::Unknown(reg));
                (register, remain)
            },
            
            Some((_, ',')) => {
                (Token::Comma, &self.remain[1..])
            }
            
            Some((_, '/')) => {
                if let Some((_, '/')) = iter.next() {
                    let (c, r) = take_while(self.remain, |_| true /* c != '\n' */);
                    (Token::Comment(c), r)
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

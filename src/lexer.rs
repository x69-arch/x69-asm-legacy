use logos::Logos;

fn trim_string(string: &'_ str, begin: usize, end: usize) -> &'_ str {
    &string[begin..string.len()-end]
}

#[derive(Logos, Clone, Copy, Debug, PartialEq)]
pub enum Token<'a> {
    #[regex("[_a-zA-Z]\\w*")]
    Ident(&'a str),
    
    #[regex("\\w+:", |lex| trim_string(lex.slice(), 0, 1))]
    Label(&'a str),
    
    #[regex("\"[^\"]*\"", |lex| trim_string(lex.slice(), 1, 1))]
    String(&'a str),
    
    #[regex("\\.\\w+", |lex| trim_string(lex.slice(), 1, 0))]
    Directive(&'a str),
    
    #[regex("(0[xX][\\da-fA-F]+|0[bB][01]+|\\d+)")]
    Immediate(&'a str),
    
    #[regex("r[0-9]+", |lex| trim_string(lex.slice(), 1, 0))]
    Register(&'a str),
    
    #[token(",")]
    Comma,
    
    #[error]
    #[regex("[ \t]+", logos::skip)]
    Error,
}

pub fn new_lexer(source: &'_ str) -> logos::Lexer<'_, Token<'_>> {
    Token::lexer(source)
}

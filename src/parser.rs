use crate::lexer::Token;
use crate::codegen::Register;
use crate::instruction::{Instruction, OperandMode};

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Log {
    Warning(usize, String, Rc<String>),
    Error(usize, String, Rc<String>),
    IOError(String, String),
}

impl Log {
    pub fn is_error(&self) -> bool { matches!(self, Self::Error(..) | Self::IOError(..)) }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "no_color")]
            Self::Warning(line, msg, origin) => write!(f, "WARNING: {}:{}: {}", origin, line + 1, msg),
            #[cfg(not(feature = "no_color"))]
            Self::Warning(line, msg, origin) => write!(f, "\x1b[1;33mWARNING:\x1b[0m {}:{}: {}", origin, line + 1, msg),
            
            #[cfg(feature = "no_color")]
            Self::Error(line, msg, origin) => write!(f, "ERROR:   {}:{}: {}", origin, line + 1, msg),
            #[cfg(not(feature = "no_color"))]
            Self::Error(line, msg, origin) => write!(f, "\x1b[1;31mERROR:\x1b[0m   {}:{}: {}", origin, line + 1, msg),
            
            #[cfg(feature = "no_color")]
            Self::IOError(msg, origin) => write!(f, "ERROR:   {}: {}", origin, msg),
            #[cfg(not(feature = "no_color"))]
            Self::IOError(msg, origin) => write!(f, "\x1b[1;31mERROR:\x1b[0m   {}: {}", origin, msg),
        }
    }
}

// TODO Immediate struct and allow labels and immediates

#[derive(Clone, Debug)]
pub enum Parameters {
    None,
    Label(String),
    LongImmediate(u16),
    OneRegister(Register),
    TwoRegisters(Register, Register),
    OneRegisterImmediate(Register, u8),
    TwoRegistersImmedaite(Register, Register, u8),
}

#[derive(Clone, Debug)]
pub enum DataByte {
    Label(String),
    Byte(u8),
}

#[derive(Clone, Debug)]
pub enum Directive {
    Line(u16),
    DB(Vec<DataByte>),
}

#[derive(Clone, Debug)]
pub enum LineData {
    Label(String),
    Directive(Directive),
    Instruction {
        name: Instruction,
        params: Parameters,
    },
}

#[derive(Clone, Debug)]
pub struct Line {
    pub origin: Rc<String>,
    pub line: usize,
    pub data: LineData,
}

pub struct ParseOptions {
    pub origin: PathBuf,
    pub include_paths: Vec<PathBuf>,
}

fn pathbuf_to_string(path: &Path) -> String {
    match path.to_owned().into_os_string().into_string() {
        Ok(string) => string,
        Err(string) => panic!("Error turning into string: {:?}", string)
    }
}

pub fn parse_file(options: &ParseOptions) -> (Vec<Line>, Vec<Log>) {
    let mut file = match File::open(&options.origin) {
        Ok(file) => file,
        Err(err) => return (vec![], vec![Log::IOError(err.to_string(), pathbuf_to_string(&options.origin))])
    };
    
    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents) {
        return (vec![], vec![Log::IOError(err.to_string(), pathbuf_to_string(&options.origin))])
    }
    
    parse_raw(&contents, Some(options))
}

pub fn parse_raw(source: &str, options: Option<&ParseOptions>) -> (Vec<Line>, Vec<Log>) {
    let mut lines = Vec::new();
    let mut logs  = Vec::new();
    
    let file_name = match options {
        Some(opts) => pathbuf_to_string(&opts.origin),
        None => String::from("[unknown]")
    };
    
    // Stupid idea but fuck you
    let origin = Rc::new(file_name);
    
    for (line, source) in source.lines().enumerate() {
        // Pushes new instruction to the lines list
        macro_rules! push_instruction {
            ($name:ident, $ins:expr) => {{
                lines.push(Line {
                    origin: origin.clone(),
                    line,
                    data: LineData::Instruction {
                        $name, params: $ins
                    }
                });
                continue;
            }}
        }
        // Will push an error and then loop back to the start
        macro_rules! log {
            ($kind:ident, $msg:expr) => {{
                logs.push(Log::$kind(line, format!($msg), origin.clone()));
                continue;
            }};
            ($kind:ident, $msg:expr, $($params:expr),+) => {{
                logs.push(Log::$kind(line, format!($msg, $($params),+), origin.clone()));
                continue;
            }};
        }
        // Will log the error or warning without looping back to the top
        macro_rules! log_only {
            ($kind:ident, $msg:expr) => {{
                logs.push(Log::$kind(line, format!($msg), origin.clone()));
            }};
            ($kind:ident, $msg:expr, $($params:expr),+) => {{
                logs.push(Log::$kind(line, format!($msg, $($params),+), origin.clone()));
            }};
        }
        
        // Creates a register or logs and error and returns to start
        macro_rules! make_register {
            ($reg:ident) => {{
                match $reg.parse::<u8>() {
                    Ok(reg) => {
                        match Register::from_u8(reg) {
                            Some(r) => r,
                            None => log!(Error, "register out of bounds: {}", $reg),
                        }
                    },
                    Err(..) => log!(Error, "register out of bounds: {}", $reg),
                }
            }}
        }
        // Turn immediate token into the integer of type `int`
        macro_rules! make_int {
            ($im:ident, $int:ident) => {{
                const BITS: usize = std::mem::size_of::<$int>() * 8;
                let mut chars = $im.chars();
                let parsed = if let Some('0') = chars.next() {
                    let mut offset = 2;
                    match chars.next() {
                        Some('x') => {
                            // String truncation logic
                            if $im.len() > BITS / 4 + 2 {
                                offset += $im.len() - BITS / 4 - 2;
                                // Grammar is very important to me
                                let bits = BITS.to_string();
                                let indefinite = match bits.as_bytes()[0] {
                                    b'8' => "an",
                                    _ => "a",
                                };
                                log_only!(Warning, "immediate {} will be truncated to {} {}-bit value", $im, indefinite, bits);
                            }
                            $int::from_str_radix(&$im[offset..], 16)
                        },
                        
                        Some('b') => {
                            // String trunctation logic
                            if $im.len() > BITS + 2 {
                                offset += $im.len() - BITS - 2;
                                // Grammar is very important to me
                                let bits = format!("{}", BITS);
                                let indefinite = match bits.as_bytes()[0] {
                                    b'8' => "an",
                                    _ => "a",
                                };
                                log_only!(Warning, "immediate {} will be truncated to {} {}-bit value", $im, indefinite, bits);
                            }
                            $int::from_str_radix(&$im[offset..], 2)
                        },
                        
                        _ => $im.parse::<$int>(),
                    }
                } else {
                    $im.parse::<$int>()
                };
                
                match parsed {
                    Ok(i) => i,
                    Err(err) => log!(Error, "could not parse {}: {}", $im, err)
                }
            }}
        }
        
        let mut lexer = crate::lexer::new_lexer(source);
        let mut first_token = lexer.next();
        
        // Parsing label
        if let Some(Token::Label(l)) = first_token {
            let data = LineData::Label(l.to_owned());
            lines.push(Line {origin: origin.clone(), line, data});
            first_token = lexer.next();
        }
        
        // Match first token and go from there
        match first_token {
            // Parsing directives
            Some(Token::Directive(dir)) => {
                match dir {
                    
                    // syntax: .include "hello.h"
                    "include" => {
                        match lexer.next() {
                            Some(Token::String(path)) => {
                                // Test path relative to the input file first
                                let parent = match options {
                                    Some(options) => options.origin.parent(),
                                    None => Some(Path::new("")),
                                }.unwrap_or_else(|| Path::new(""));
                                let file_name = parent.join(path);
                                
                                let options = ParseOptions {
                                    origin: file_name,
                                    include_paths: vec![]
                                };
                                let (include_lines, include_logs) = parse_file(&options);
                                lines.extend(include_lines);
                                logs.extend(include_logs);
                                // TODO: test paths in include_paths!
                            },
                            Some(token) => log!(Error, "expected a string file path, got: {:?}", token),
                            None => log!(Error, "expected a string file path"),
                        }
                    },
                    
                    "line" => {
                        match lexer.next() {
                            Some(Token::Immediate(offset)) => {
                                match lexer.next() {
                                    None => {
                                        let data = LineData::Directive(Directive::Line(make_int!(offset, u16)));
                                        lines.push(Line {origin: origin.clone(), line, data});
                                    },
                                    Some(token) => log!(Error, "unexpected token after line offset: {:?}", token),
                                }
                            },
                            Some(token) => log!(Error, "expected an immediate for line offset, got: {:?}", token),
                            None => log!(Error, "expected an immediate for line offset"),
                        }
                    },
                    
                    "db" => {
                        let mut data_bytes = Vec::new();
                        loop {
                            match lexer.next() {
                                Some(Token::Immediate(byte)) => data_bytes.push(DataByte::Byte(make_int!(byte, u8))),
                                Some(Token::Ident(l)) => data_bytes.push(DataByte::Label(l.to_owned())),
                                Some(Token::String(s)) => data_bytes.extend(s.as_bytes().iter().map(|b| DataByte::Byte(*b))),
                                Some(token) => log!(Error, "unexpected token in db field: {:?}", token),
                                None => {
                                    if data_bytes.is_empty() {
                                        log!(Warning, "empty db field");
                                    }
                                    lines.push(Line {origin: origin.clone(), line, data: LineData::Directive(Directive::DB(data_bytes))});
                                    break;
                                }
                            }
                        }
                    },
                    
                    _ => log!(Error, "unknown directive: {}", dir)
                }
            },
            
            // Parsing instructions
            Some(Token::Ident(ins)) => {
                let name: Instruction = match Instruction::from_str(&ins.to_uppercase()) {
                    Some(ins) => ins,
                    None => log!(Error, "unknown instruction: {}", ins),
                };
                
                let asm_info = name.assemble_info();
                match asm_info.1 {
                    OperandMode::NoParams => match lexer.next() {
                        None => push_instruction!(name, Parameters::None),
                        Some(token) => log!(Error, "{} expects zero parameters, got: {:?}", name.to_str(), token),
                    },
                    
                    OperandMode::OneRegister => {
                        let reg = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "{} expectes one register, got: {:?}", name.to_str(), token),
                            None => log!(Error, "{} requires one register", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::OneRegister(reg)),
                            Some(token) => log!(Error, "unexpected token after register: {:?}", token),
                        }
                    },
                    
                    OperandMode::OneOrTwoRegisters => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "{} expects at leat one register, got: {:?}", name.to_str(), token),
                            None => log!(Error, "{} expects at least one register", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::OneRegister(reg1)),
                            Some(Token::Comma) => {},
                            Some(token) => log!(Error, "expected ',' after first register, got: {:?}", token),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "expected a register, got: {:?}", token),
                            None => log!(Error, "trailing ','s are not allowed"),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(token) => log!(Error, "unexpected token after second register: {:?}", token),
                        }
                    },
                    
                    OperandMode::OneRegisterAndImmediate => {
                        let reg = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "{} expects one register and an immediate, got: {:?}", name.to_str(), token),
                            None => log!(Error, "{} expects one register and an immediate", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log!(Error, "expected ',' after register, got: {:?}", token),
                            None => log!(Error, "{} expects one register and an immediate", name.to_str()),
                        }
                        let i = match lexer.next() {
                            Some(Token::Immediate(i)) => make_int!(i, u8),
                            Some(token) => log!(Error, "expected a regsiter, got: {:?}", token),
                            None => log!(Error, "trailing ','s are not allowed"),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::OneRegisterImmediate(reg, i)),
                            Some(token) => log!(Error, "unexpected token after immediate: {:?}", token),
                        }
                    },
                    
                    OperandMode::TwoRegisters => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "{} expects two registers, got: {:?}", name.to_str(), token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log!(Error, "expected ',' after first register, got: {:?}", token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "expected a regsiter, got: {:?}", token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(token) => log!(Error, "unexpected token after second register: {:?}", token),
                        }
                    },
                    
                    OperandMode::TwoRegistersOrImmediate => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "{} expects at least two parameters, got: {:?}", name.to_str(), token),
                            None => log!(Error, "{} expects at least two parameters", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log!(Error, "expected ',' after first register, got: {:?}", token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(Token::Immediate(i)) => match lexer.next() {
                                None => push_instruction!(name, Parameters::OneRegisterImmediate(reg1, make_int!(i, u8))),
                                Some(token) => log!(Error, "unexpected token after immediate: {:?}", token),
                            },
                            Some(token) => log!(Error, "expected a regsiter or an immediate, got: {:?}", token),
                            None => log!(Error, "{} expects as least two parameters", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(Token::Comma) => {},
                            Some(token) => log!(Error, "expected ',' after second register, got: {:?}", token),
                        }
                        let i = match lexer.next() {
                            Some(Token::Immediate(i)) => make_int!(i, u8),
                            Some(token) => log!(Error, "expected an immediate, got: {:?}", token),
                            None => log!(Error, "{} expects two registers and an immediate", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegistersImmedaite(reg1, reg2, i)),
                            Some(token) => log!(Error, "unexpected token after immediate: {:?}", token),
                        }
                    },
                    
                    OperandMode::TwoRegistersOrLongImmediate => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(Token::Immediate(i)) => match lexer.next() {
                                None => push_instruction!(name, Parameters::LongImmediate(make_int!(i, u16))),
                                Some(token) => log!(Error, "unexpected token after immediate: {:?}", token)
                            },
                            Some(Token::Ident(l)) => match lexer.next() {
                                None => push_instruction!(name, Parameters::Label(l.to_owned())),
                                Some(token) => log!(Error, "unexpected token after label: {:?}", token)
                            },
                            Some(token) => log!(Error, "{} expects two registers, got: {:?}", name.to_str(), token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log!(Error, "expected ',' after first register, got: {:?}", token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log!(Error, "expected a regsiter, got: {:?}", token),
                            None => log!(Error, "{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(token) => log!(Error, "unexpected token after second register: {:?}", token),
                        }
                    },
                }
            },
            
            Some(token) => log!(Error, "unexpected token: {:?}", token),
            
            // Should not get here lol
            // None => { panic!("Should never get here, contact your local assembler dev") }
            
            // Can get here now lmao
            None => continue,
        }
    }
    
    (lines, logs)
}

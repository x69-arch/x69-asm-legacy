use crate::lexer::{Lexer, Token};
use crate::codegen::Register;
use crate::instruction::{Instruction, OperandMode};

#[derive(Clone, Debug)]
pub enum Log {
    Warning(usize, String),
    Error(usize, String),
}

impl Log {
    pub fn is_error(&self) -> bool { matches!(self, Self::Error(..)) }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Warning(line, msg) => write!(f, "WARNING: Line {}: {}", line + 1, msg),
            Self::Error(line, msg) => write!(f, "ERROR: Line {}: {}", line + 1, msg),
        }
    }
}

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
pub enum LineData {
    Label(String),
    Instruction {
        name: Instruction,
        params: Parameters,
    },
}

#[derive(Clone, Debug)]
pub struct Line {
    pub line: usize,
    pub data: LineData,
}

pub fn parse(source: &str) -> (Vec<Line>, Vec<Log>) {
    let mut lines = Vec::new();
    let mut logs  = Vec::new();
    
    for (line, source) in source.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        // Pushes new instruction to the lines list
        macro_rules! push_instruction {
            ($name:ident, $ins:expr) => {{
                lines.push(Line {
                    line,
                    data: LineData::Instruction {
                        $name, params: $ins
                    }
                });
                continue;
            }}
        }
        // Will push an error and then loop back to the start
        macro_rules! log_error {
            ($msg:expr) => {{
                logs.push(Log::Error(line, format!($msg)));
                continue;
            }};
            ($msg:expr, $($params:expr),+) => {{
                logs.push(Log::Error(line, format!($msg, $($params),+)));
                continue;
            }};
        };
        // Creates a register or logs and error and returns to start
        macro_rules! make_register {
            ($reg:ident) => {
                match Register::from_u8($reg) {
                    Some(reg) => reg,
                    None => log_error!("register out of bounds: {}", $reg),
                }
            }
        }
        // Create an 8bit immediate, warning of truncation
        macro_rules! make_short {
            ($im:ident) => {{
                if $im > u8::MAX as u16 {
                    logs.push(Log::Warning(line, format!("immediate value {} will be truncated to 8 bits", $im)));
                }
                ($im & 0xFF) as u8
            }}
        }
        
        let mut lexer = Lexer::new(source);
        
        // Match first token and go from there
        match lexer.next() {
            // Parsing label
            Some(Token::Label(l)) => {
                // Takes the name of the label without the trailing ':'
                let data = LineData::Label(l.to_owned());
                match lexer.next() {
                    None => lines.push(Line {line, data}),
                    Some(token) => log_error!("unexpected token after label: {:?}", token),
                }
                continue;
            },
            
            // Parsing instructions
            Some(Token::Ident(ins)) => {
                let name: Instruction = match Instruction::from_str(&ins.to_uppercase()) {
                    Some(ins) => ins,
                    None => log_error!("unknown instruction: {}", ins),
                };
                
                let asm_info = name.assemble_info();
                match asm_info.1 {
                    OperandMode::NoParams => match lexer.next() {
                        None => push_instruction!(name, Parameters::None),
                        Some(token) => log_error!("{} expects zero parameters, got: {:?}", name.to_str(), token),
                    },
                    
                    OperandMode::OneRegister => {
                        let reg = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("{} expectes one register, got: {:?}", name.to_str(), token),
                            None => log_error!("{} requires one register", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::OneRegister(reg)),
                            Some(token) => log_error!("unexpected token after register: {:?}", token),
                        }
                    },
                    
                    OperandMode::OneOrTwoRegisters => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("{} expects at leat one register, got: {:?}", name.to_str(), token),
                            None => log_error!("{} expects at least one register", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::OneRegister(reg1)),
                            Some(Token::Comma) => {},
                            Some(token) => log_error!("expected ',' after first register, got: {:?}", token),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("expected a regsiter, got: {:?}", token),
                            None => log_error!("trailing ','s are not allowed"),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(token) => log_error!("unexpected token after second register: {:?}", token),
                        }
                    },
                    
                    OperandMode::OneRegisterAndImmediate => {
                        let reg = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("{} expects one register and an immediate, got: {:?}", name.to_str(), token),
                            None => log_error!("{} expects one register and an immediate", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log_error!("expected ',' after register, got: {:?}", token),
                            None => log_error!("{} expects one register and an immediate", name.to_str()),
                        }
                        let i = match lexer.next() {
                            Some(Token::Immediate(i)) => make_short!(i),
                            Some(token) => log_error!("expected a regsiter, got: {:?}", token),
                            None => log_error!("trailing ','s are not allowed"),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::OneRegisterImmediate(reg, i)),
                            Some(token) => log_error!("unexpected token after immediate: {:?}", token),
                        }
                    },
                    
                    OperandMode::TwoRegisters => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("{} expects two registers, got: {:?}", name.to_str(), token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log_error!("expected ',' after first register, got: {:?}", token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("expected a regsiter, got: {:?}", token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(token) => log_error!("unexpected token after second register: {:?}", token),
                        }
                    },
                    
                    OperandMode::TwoRegistersOrImmediate => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("{} expects at least two parameters, got: {:?}", name.to_str(), token),
                            None => log_error!("{} expects at least two parameters", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log_error!("expected ',' after first register, got: {:?}", token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(Token::Immediate(i)) => match lexer.next() {
                                None => push_instruction!(name, Parameters::OneRegisterImmediate(reg1, make_short!(i))),
                                Some(token) => log_error!("unexpected token after immediate: {:?}", token),
                            },
                            Some(token) => log_error!("expected a regsiter or an immediate, got: {:?}", token),
                            None => log_error!("{} expects as least two parameters", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(Token::Comma) => {},
                            Some(token) => log_error!("expected ',' after second register, got: {:?}", token),
                        }
                        let i = match lexer.next() {
                            Some(Token::Immediate(i)) => make_short!(i),
                            Some(token) => log_error!("expected an immediate, got: {:?}", token),
                            None => log_error!("{} expects two registers and an immediate", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegistersImmedaite(reg1, reg2, i)),
                            Some(token) => log_error!("unexpected token after immediate: {:?}", token),
                        }
                    },
                    
                    OperandMode::TwoRegistersOrLongImmediate => {
                        let reg1 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(Token::Immediate(i)) => match lexer.next() {
                                None => push_instruction!(name, Parameters::LongImmediate(i)),
                                Some(token) => log_error!("unexpected token after immediate: {:?}", token)
                            },
                            Some(Token::Ident(l)) => match lexer.next() {
                                None => push_instruction!(name, Parameters::Label(l.to_owned())),
                                Some(token) => log_error!("unexpected token after label: {:?}", token)
                            },
                            Some(token) => log_error!("{} expects two registers, got: {:?}", name.to_str(), token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            Some(Token::Comma) => {},
                            Some(token) => log_error!("expected ',' after first register, got: {:?}", token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        }
                        let reg2 = match lexer.next() {
                            Some(Token::Register(r)) => make_register!(r),
                            Some(token) => log_error!("expected a regsiter, got: {:?}", token),
                            None => log_error!("{} expects two registers", name.to_str()),
                        };
                        match lexer.next() {
                            None => push_instruction!(name, Parameters::TwoRegisters(reg1, reg2)),
                            Some(token) => log_error!("unexpected token after second register: {:?}", token),
                        }
                    },
                }
            },
            
            Some(token) => log_error!("unexpected token: {:?}", token),
            
            // Should not get here lol
            None => { panic!("Should never get here, contact your local assembler dev") }
        }
    }
    
    (lines, logs)
}

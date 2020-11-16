use crate::lexer::{Lexer, Token};
pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug)]
pub struct Register(u8);
impl Register {
    pub fn from_u8(r: u8) -> Option<Self> {
        // Only allow 0..15
        if r <= 15 {
            Some(Self(r))
        } else {
            None
        }
    }
}

enum OperandMode {
    NoParams,
    OneRegister,
    OneOrTwoRegisters,
    OneRegisterAndImmediate,
    TwoRegisters,
    TwoRegistersOrImmediate,
}

enum RegisterMap {
    AB,
    BA,
    AA,
    BB,
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    NOP,
    CLR,
    SER,
    NOT,
    TWO,
    AND,
    NND,
    ORR,
    NOR,
    XOR,
    XNR,
    ADD,
    ADC,
    SUB,
    SBC,
    INC,
    DEC,
    MOV,
    MVN,
}
impl Instruction {
    #[inline(always)]
    fn from_str(name: &str) -> Option<Self> {
        match name {
            "NOP" => Some(Self::NOP),
            "CLR" => Some(Self::CLR),
            "SER" => Some(Self::SER),
            "NOT" => Some(Self::NOT),
            "TWO" => Some(Self::TWO),
            "AND" => Some(Self::AND),
            "NND" => Some(Self::NND),
            "ORR" => Some(Self::ORR),
            "NOR" => Some(Self::NOR),
            "XOR" => Some(Self::XOR),
            "XNR" => Some(Self::XNR),
            "ADD" => Some(Self::ADD),
            "ADC" => Some(Self::ADC),
            "SUB" => Some(Self::SUB),
            "SBC" => Some(Self::SBC),
            "INC" => Some(Self::INC),
            "DEC" => Some(Self::DEC),
            "MOV" => Some(Self::MOV),
            "MVN" => Some(Self::MVN),
            _ => None
        }
    }
    #[inline(always)]
    fn to_str(&self) -> &str {
        match self {
            Self::NOP => "NOP",
            Self::CLR => "CLR",
            Self::SER => "SER",
            Self::NOT => "NOT",
            Self::TWO => "TWO",
            Self::AND => "AND",
            Self::NND => "NND",
            Self::ORR => "ORR",
            Self::NOR => "NOR",
            Self::XOR => "XOR",
            Self::XNR => "XNR",
            Self::ADD => "ADD",
            Self::ADC => "ADC",
            Self::SUB => "SUB",
            Self::SBC => "SBC",
            Self::INC => "INC",
            Self::DEC => "DEC",
            Self::MOV => "MOV",
            Self::MVN => "MVN",
        }
    }
    
    #[inline(always)]
    fn assemble_info(&self) -> (u8, OperandMode, RegisterMap) {
        use OperandMode::*;
        use RegisterMap::*;
        
        match self {
            Self::CLR => (0b00100000, OneRegister, AA),
            Self::SER => (0b00110000, OneRegister, AA),
            
            Self::NOT => (0b00100001, TwoRegistersOrImmediate, BA),
            Self::TWO => (0b00110001, TwoRegistersOrImmediate, BA),
            Self::AND => (0b00100010, TwoRegistersOrImmediate, BA),
            Self::NND => (0b00110010, TwoRegistersOrImmediate, BA),
            Self::ORR => (0b00100011, TwoRegistersOrImmediate, BA),
            Self::NOR => (0b00110011, TwoRegistersOrImmediate, BA),
            Self::XOR => (0b00100100, TwoRegistersOrImmediate, BA),
            Self::XNR => (0b00110100, TwoRegistersOrImmediate, BA),
            Self::ADD => (0b00100101, TwoRegistersOrImmediate, BA),
            Self::ADC => (0b00110101, TwoRegistersOrImmediate, BA),
            Self::SUB => (0b00100110, TwoRegistersOrImmediate, BA),
            Self::SBC => (0b00110110, TwoRegistersOrImmediate, BA),
            Self::INC => (0b00100111, OneOrTwoRegisters,       BA),
            Self::DEC => (0b00110111, OneOrTwoRegisters,       BA),
            Self::MOV => (0b00101000, TwoRegistersOrImmediate, BA),
            Self::MVN => (0b00111000, TwoRegistersOrImmediate, BA),
            
            _ => (0, OperandMode::NoParams, RegisterMap::AB)
        }
    }
}

#[derive(Debug)]
pub struct InstructionBuilder {
    line: usize,
    name: Instruction,
    a: Option<Register>,
    b: Option<Register>,
    immediate: Option<u8>
}

impl InstructionBuilder {
    fn assemble(&self, buffer: &mut Vec<u8>) -> Result<u8> {
        let asm_info = self.name.assemble_info();
        
        let (a, b, i) = match asm_info.1 {
            OperandMode::OneRegister => match (self.a, self.b, self.immediate) {
                (Some(a), None, None) => (a, a, None),
                _ => return Err(format!("Line {}: {} requires only one register", self.line, self.name.to_str())),
            },
            
            OperandMode::OneOrTwoRegisters => match (self.a, self.b, self.immediate) {
                (Some(a), Some(b), None) => (a, b, None),
                (Some(a), None, None) => (a, a, None),
                (.., Some(_)) => return Err(format!("Line {}: {} cannot accept an immediate", self.line, self.name.to_str())),
                _ => return Err(format!("Line {}: {} requires at least one register", self.line, self.name.to_str())),
            },
            
            OperandMode::TwoRegistersOrImmediate => match self.a {
                Some(a) => match self.b {
                    Some(b) => (a, b, self.immediate),
                    None => match self.immediate {
                        Some(_) => (a, a, self.immediate),
                        None => return Err(format!("Line {}: {} requires at least two operands", self.line, self.name.to_str())),
                    }
                },
                _ => return Err(format!("Line {}: {} requires at least two operands", self.line, self.name.to_str())),
            },
            
            _ => return Err(format!("Line {}: {} NOT IMPLEMENTED LOL", self.line, self.name.to_str())),
        };
        
        let (a, b) = match asm_info.2 {
            RegisterMap::AA => (a, a),
            RegisterMap::AB => (a, b),
            RegisterMap::BA => (b, a),
            RegisterMap::BB => (b, b),
        };
        
        let lo = asm_info.0;
        let hi = (a.0 & 0x0f) | (b.0 << 4 & 0xF0);
        
        if let Some(immediate) = i {
            buffer.push(lo | 0b10000000);
            buffer.push(hi);
            buffer.push(immediate);
            Ok(3)
        } else {
            buffer.push(lo);
            buffer.push(hi);
            Ok(2)
        }
    }
}

impl Default for InstructionBuilder {
    fn default() -> Self {
        Self {
            line: 0,
            name: Instruction::NOP,
            a: None,
            b: None,
            immediate: None
        }
    }
}

pub fn parse_line(lexer: &mut Lexer, line: usize) -> Result<InstructionBuilder> {
    match lexer.next() {
        Some(Token::Ident(ins)) => {
            let ins = ins.to_ascii_uppercase();
            
            let name = match Instruction::from_str(ins.as_str()) {
                Some(name) => Ok(name),
                None => Err(format!("Line {}: Unknown instruction: {}", line, ins))
            }?;
            
            let mut output = InstructionBuilder {
                line, name, .. Default::default()
            };
            
            // Take first operand, register a
            match lexer.next() {
                Some(Token::Register(r)) => {
                    output.a.replace(Register::from_u8(r)
                        .ok_or_else(|| format!("Line {}: Register out of bounds: {}", line, r))?);
                },
                Some(token) => return Err(format!("Line {}: Expected register as first operand, got: {:?}", line, token)),
                None => return Ok(output),
            }
            
            // Match comma or exit
            match lexer.next() {
                Some(Token::Comma) => {}
                Some(token) => return Err(format!("Line {}: Expected a ',' after the first operand, got: {:?}", line, token)),
                None => return Ok(output),
            }
            
            // Take second operand, either register b or immediate
            match lexer.next() {
                Some(Token::Register(r)) => {
                    output.b.replace(Register::from_u8(r)
                        .ok_or_else(|| format!("Line {}: Register out of bounds: {}", line, r))?);
                },
                Some(Token::Immediate(i)) => {
                    output.immediate.replace(i);
                    match lexer.next() {
                        Some(token) => return Err(format!("Line {}: Unexpected token after immediate operand: {:?}", line, token)),
                        None => return Ok(output),
                    }
                },
                Some(token) => return Err(format!("Line {}: Expected register or immediate as second operand, got: {:?}", line, token)),
                None => return Err(format!("Line {}: Unexpected trailing ','", line)),
            }
            
            // Match optional comma
            match lexer.next() {
                Some(Token::Comma) => {}
                Some(token) => return Err(format!("Line {}: Expected a ',' after the second operand, got: {:?}", line, token)),
                None => return Ok(output),
            }
            
            match lexer.next() {
                Some(Token::Immediate(i)) => {
                    output.immediate.replace(i);
                    match lexer.next() {
                        Some(token) => Err(format!("Line {}: Unexpected token after third operand: {:?}", line, token)),
                        None => Ok(output),
                    }
                },
                Some(token) => Err(format!("Line {}: Expected immediate as third operand, got: {:?}", line, token)),
                None => Err(format!("Line {}: Unexpected trailing ','", line)),
            }
        },
        Some(token) => Err(format!("Line {}: Unexpected token: {:?}", line, token)),
        None => Err(format!("Line {} was parsed as empty, contact your local assembler dev...", line))
    }
}

pub fn assemble(source: &str, buffer: &mut Vec<u8>) -> Result<u8> {
    let mut written = 0;
    for (i, line) in source.lines().enumerate().filter(|(_, line)| !line.trim().is_empty()) {
        let instruction = parse_line(&mut Lexer::new(line), i)?;
        written += instruction.assemble(buffer)?;
    }
    Ok(written)
}

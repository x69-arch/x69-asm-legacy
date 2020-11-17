use crate::instruction::*;
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

#[derive(Debug)]
pub struct InstructionBuilder {
    line: usize,
    name: Instruction,
    a: Option<Register>,
    b: Option<Register>,
    immediate: Option<u16>
}

impl InstructionBuilder {
    fn assemble(&self, buffer: &mut Vec<u8>) -> Result<u8> {
        let asm_info = self.name.assemble_info();
        
        let (regs, i) = match asm_info.1 {
            OperandMode::NoParams => (Some((Register(0), Register(0))), None),
            
            OperandMode::OneRegister => match (self.a, self.b, self.immediate) {
                (Some(a), None, None) => (Some((a, a)), None),
                _ => return Err(format!("Line {}: {} requires only one register", self.line, self.name.to_str())),
            },
            
            OperandMode::OneRegisterAndImmediate => match (self.a, self.b, self.immediate) {
                (Some(a), None, i) if i.is_some() => (Some((a, a)), i),
                _ => return Err(format!("Line {}: {} requires only one register and one immediate", self.line, self.name.to_str())),
            },
            
            OperandMode::OneOrTwoRegisters => match (self.a, self.b, self.immediate) {
                (Some(a), Some(b), None) => (Some((a, b)), None),
                (Some(a), None, None) => (Some((a, a)), None),
                (.., Some(_)) => return Err(format!("Line {}: {} cannot accept an immediate", self.line, self.name.to_str())),
                _ => return Err(format!("Line {}: {} requires at least one register", self.line, self.name.to_str())),
            },
            
            OperandMode::TwoRegisters => match (self.a, self.b, self.immediate) {
                (Some(a), Some(b), None) => (Some((a, b)), None),
                _ => return Err(format!("Line {}: {} requires only two registers", self.line, self.name.to_str())),
            },
            
            OperandMode::TwoRegistersOrImmediate => match self.a {
                Some(a) => match self.b {
                    Some(b) => (Some((a, b)), self.immediate),
                    None => match self.immediate {
                        Some(i) => (Some((a, a)), Some(i)),
                        None => return Err(format!("Line {}: {} requires at least two operands", self.line, self.name.to_str())),
                    }
                },
                _ => return Err(format!("Line {}: {} requires at least two operands", self.line, self.name.to_str())),
            },
            
            //_ => return Err(format!("Line {}: {} NOT IMPLEMENTED LOL", self.line, self.name.to_str())),
        };
        
        let lo_byte = asm_info.0;
        let (mid_byte, hi_byte) = match regs {
            Some((a, b)) => {
                let (a, b) = match asm_info.2 {
                    RegisterMap::AA => (a, a),
                    RegisterMap::AB => (a, b),
                    RegisterMap::BA => (b, a),
                    // RegisterMap::BB => (b, b),
                };
                ((a.0 & 0x0f) | (b.0 << 4 & 0xF0), i.map(|i| i as u8))
            },
            None => {
                match i {
                    // Split high and low of 16 bit immediate
                    Some(i) => ((i & 0x00FF) as u8, Some((i & 0xFF00 >> 4) as u8)),
                    None => return Err(format!("Line {}: {} requires a 16 bit immedaite", self.line, self.name.to_str()))
                }
            }
        };
        
        if let Some(hi_byte) = hi_byte {
            // Set the immediate bit
            buffer.push(lo_byte | 0b10000000);
            buffer.push(mid_byte);
            buffer.push(hi_byte);
            Ok(3)
        } else {
            buffer.push(lo_byte);
            buffer.push(mid_byte);
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
                Some(Token::Immediate(i)) => {
                    output.immediate.replace(i);
                    match lexer.next() {
                        Some(token) => return Err(format!("Line {}: Unexpected token after immediate: {:?}", line, token)),
                        None => return Ok(output),
                    }
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
                    if i > u8::MAX as u16 {
                        println!("WARNING: Line {}: immediate will be truncated to 8 bit value", line);
                    }
                    output.immediate.replace(i & 0xFF);
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
                    if i > u8::MAX as u16 {
                        println!("WARNING: Line {}: immediate will be truncated to 8 bit value", line);
                    }
                    output.immediate.replace(i & 0xFF);
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

#[cfg(test)]
mod tests {
    use super::*;
    
    fn assemble_string(source: &str) -> Vec<u8> {
        let mut buffer = Vec::new();
        assemble(source, &mut buffer).unwrap();
        buffer
    }
    
    #[test]
    fn simple_add() {
        let buffer = assemble_string("add r15, r0, 123");
        assert_eq!(buffer[0], 0b10100101);
        assert_eq!(buffer[1], 0xF0);
        assert_eq!(buffer[2], 123);
        
        let buffer = assemble_string("ADD r1, 69");
        assert_eq!(buffer[0], 0b10100101);
        assert_eq!(buffer[1], 0x11);
        assert_eq!(buffer[2], 69);
        
        let buffer = assemble_string("AdD r1, r2");
        assert_eq!(buffer[0], 0b00100101);
        assert_eq!(buffer[1], 0x12);
    }
    
    #[test]
    fn nop() {
        let buffer = assemble_string("nop");
        assert_eq!(buffer[0], 0b00101001);
        assert_eq!(buffer[1], 0x00);
    }
    
    #[test]
    fn lpc() {
        let buffer = assemble_string("lpc r15, r0");
        assert_eq!(buffer[0], 0b01000100);
        assert_eq!(buffer[1], 0x0F);
    }
}

use crate::instruction::RegisterMap;
use crate::parser::{Line, LineData, Parameters};

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

pub fn assemble_lines(lines: &[Line], buffer: &mut Vec<u8>) {
    for line in lines {
        match &line.data {
            // TODO: Create link table
            LineData::Label(_) => {},
            
            LineData::Instruction {name, params} => {
                let asm_info = name.assemble_info();
                
                enum Usage {
                    Register(Register, Register, Option<u8>),
                    LongImmidiate(u16),
                };
                
                let usage: Usage = match *params {
                    Parameters::None => Usage::Register(Register(0), Register(0), None),
                    Parameters::OneRegister(a) => Usage::Register(a, a, None),
                    Parameters::LongImmediate(i) => Usage::LongImmidiate(i),
                    Parameters::TwoRegisters(a, b) => Usage::Register(a, b, None),
                    Parameters::OneRegisterImmediate(a, i) => Usage::Register(a, a, Some(i)),
                    Parameters::TwoRegistersImmedaite(a, b, i) => Usage::Register(a, b, Some(i)),
                };
                
                match usage {
                    Usage::Register(a, b, maybe_i) => {
                        // Swap A and B according to register map
                        let (Register(a), Register(b)) = match asm_info.2 {
                            RegisterMap::AA => (a, a),
                            RegisterMap::AB => (a, b),
                            RegisterMap::BA => (b, a),
                        };
                        let mid = (a & 0x0F) | (b << 4 & 0xF0);
                        if let Some(i) = maybe_i {
                            buffer.push(asm_info.0 | 0b10000000);
                            buffer.push(mid);
                            buffer.push(i);
                        } else {
                            buffer.push(asm_info.0);
                            buffer.push(mid);
                        }
                    },
                    
                    Usage::LongImmidiate(i) => {
                        buffer.push(asm_info.0 | 0b10000000);
                        buffer.push((i & 0xFF) as u8);
                        buffer.push((i >> 8) as u8);
                    }
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use super::assemble_lines;
    fn assemble_string(source: &str) -> Vec<u8> {
        let (lines, logs) = parse(source);
        
        // Print out for debugging purposes
        for log in logs {
            println!("{}", log);
        }
        
        let mut buffer = Vec::new();
        assemble_lines(&lines, &mut buffer);
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
    
    #[test]
    fn jmp() {
        let buffer = assemble_string("jmp r0, r15");
        assert_eq!(buffer[0], 0b01000000);
        assert_eq!(buffer[1], 0xF0);
        
        let buffer = assemble_string("rjmp 6969");
        assert_eq!(buffer[0], 0b11000010);
        assert_eq!(buffer[1], (6969 & 0xFF) as u8);
        assert_eq!(buffer[2], (6969 >> 8)   as u8);
    }
}

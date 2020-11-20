use crate::instruction::RegisterMap;
use crate::parser::{Line, LineData, Log, Parameters};

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

pub fn assemble_lines(lines: &[Line], logs: &mut Vec<Log>) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut link_table = std::collections::HashMap::<String, usize>::new();
    let mut unresolved = Vec::new();
    
    for line in lines {
        match &line.data {
            // TODO: Create link table
            LineData::Label(name) => {
                if let Some(_label) = link_table.insert(name.clone(), buffer.len()) {
                    logs.push(Log::Error(line.line, format!("symbol {} declared multiple times", name)));
                }
            },
            
            LineData::Instruction {name, params} => {
                let asm_info = name.assemble_info();
                
                enum Usage {
                    Register(Register, Register, Option<u8>),
                    LongImmidiate(u16),
                    Unresolved(String),
                };
                
                let usage: Usage = match *params {
                    Parameters::None => Usage::Register(Register(0), Register(0), None),
                    Parameters::Label(ref label) => Usage::Unresolved(label.clone()),
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
                    },
                    
                    // Support for labels
                    Usage::Unresolved(label) => {
                        buffer.push(asm_info.0 | 0b10000000);
                        // Temporary data
                        unresolved.push((label, buffer.len(), line.line));
                        buffer.push(0);
                        buffer.push(0);
                    },
                };
            }
        }
    }
    
    for link in unresolved {
        if let Some(location) = link_table.get(&link.0) {
            let offset = *location as u16;
            let lo = (offset & 0xFF) as u8;
            let hi = (offset >> 8) as u8;
            buffer[link.1] = lo;
            buffer[link.1 + 1] = hi;
        } else {
            logs.push(Log::Error(link.2, format!("unresolved symbol: {}", link.0)));
        }
    }
    
    buffer
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use crate::codegen::assemble_lines;
    fn assemble_string(source: &str) -> Vec<u8> {
        let (lines, mut logs) = parse(source);
        let assembly = assemble_lines(&lines, &mut logs);
        
        // Print out for debugging purposes
        for log in logs {
            println!("{}", log);
        }
        
        assembly
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
    
    #[test]
    fn label() {
        let labels = assemble_string("
            set r0, 1
            mov r1, r0
        _loop:
            add r1, r0
            add r0, r1
            call _loop
        ");
        
        let basic = assemble_string("
            set r0, 1
            mov r1, r0
            add r1, r0
            add r0, r1
            call 5
        ");
        
        // Both codes should output identical binaries
        assert_eq!(basic, labels);
    }
}

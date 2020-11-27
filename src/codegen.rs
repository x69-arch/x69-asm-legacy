use crate::instruction::RegisterMap;
use crate::parser::{Line, LineData, Log, Parameters, DataByte, Directive};

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
                if let Some(_overriden_label) = link_table.insert(name.clone(), buffer.len()) {
                    logs.push(Log::Error(line.line, format!("symbol {} declared multiple times", name)));
                }
            },
            
            LineData::Directive(dir) => {
                match dir {
                    Directive::Line(offset) => {
                        if *offset < buffer.len() as u16 {
                            logs.push(Log::Error(line.line, format!("line offset is less than current offset: {:x}", buffer.len())));
                        } else {
                            let padding = offset - buffer.len() as u16;
                            if padding % 2 == 1 {
                                logs.push(Log::Warning(line.line, "line offset will not guarantee instruction alignment".to_owned()));
                            }
                            buffer.resize(buffer.len() + padding as usize, 0);
                        }
                    },
                    
                    Directive::DB(data_byte) => {
                        for db in data_byte {
                            match db {
                                DataByte::Byte(byte) => buffer.push(*byte),
                                DataByte::Label(label) => {
                                    unresolved.push((label.clone(), buffer.len(), line.line));
                                    buffer.push(0xDE);
                                    buffer.push(0xAD);
                                }
                            }
                        }
                    }
                }
            }
            
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
                        buffer.push(0xDE);
                        buffer.push(0xAD);
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
        let buffer = assemble_string("add r15, r0, 0b10101");
        assert_eq!(buffer[0], 0b10100101);
        assert_eq!(buffer[1], 0xF0);
        assert_eq!(buffer[2], 0b10101);
        
        let buffer = assemble_string("ADD r1, 0xDEAD");
        assert_eq!(buffer[0], 0b10100101);
        assert_eq!(buffer[1], 0x11);
        // Checking that the hex literal was properly truncated
        assert_eq!(buffer[2], 0xAD);
        
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
        assert_eq!(buffer[0], 0b01001000);
        assert_eq!(buffer[1], 0x0F);
    }
    
    #[test]
    fn jmp() {
        let buffer = assemble_string("jmp r0, r15");
        assert_eq!(buffer[0], 0b01000100);
        assert_eq!(buffer[1], 0xF0);
        
        let buffer = assemble_string("rjmp 6969");
        assert_eq!(buffer[0], 0b11000110);
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
            jmp _loop
        ");
        let basic = assemble_string("
            set r0, 1
            mov r1, r0
            add r1, r0
            add r0, r1
            jmp 5
        ");
        
        // Both codes should output identical binaries
        assert_eq!(basic, labels);
        
        let halt = assemble_string("halt: jmp halt");
        assert_eq!(halt[0], 0b11000100);
        assert_eq!(halt[1], 0);
        assert_eq!(halt[2], 0);
    }
    
    #[test]
    fn db() {
        let bytes = assemble_string("array: .db 0 1 array \"hello\" 3 4");
        assert_eq!(bytes, vec![0, 1, 0, 0, b'h', b'e', b'l', b'l', b'o', 3, 4]);
    }
    
    #[test]
    fn line_offset() {
        let buffer = assemble_string("
            add r1, r2
            .line 0x1234
        _halt:
            jmp _halt");
        
        assert_eq!(buffer.len(), 0x1237);
        assert_eq!(buffer[0x1235], 0x34);
        assert_eq!(buffer[0x1236], 0x12);
    } 
    
    #[test]
    fn ldr_sdr() {
        let buffer = assemble_string("ldr r0, 15");
        assert_eq!(buffer[0], 0b10010000);
        assert_eq!(buffer[1], 0);
        assert_eq!(buffer[2], 15);
        
        let buffer = assemble_string("str r0, 150");
        assert_eq!(buffer[0], 0b10010001);
        assert_eq!(buffer[1], 0);
        assert_eq!(buffer[2], 150);
    }
}

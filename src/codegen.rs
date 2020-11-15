use crate::lexer::{Lexer, Token};
pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug)]
pub struct Register(u8);
impl Register {
    pub fn from_u8(r: u8) -> Result<Self> {
        // Only allow 0..15
        if r <= 15 {
            Ok(Self(r))
        } else {
            Err(format!("Register out of range: {}", r))
        }
    }
}

#[derive(Debug)]
pub enum Instrinsic {
    CLR(Register),
    SER(Register),
    NOT(Register, Register),
    TWO(Register, Register),
    AND(Register, Register),
    NND(Register, Register),
    ORR(Register, Register),
    NOR(Register, Register),
    XOR(Register, Register),
    XNR(Register, Register),
    ADD(Register, Register),
    ADC(Register, Register),
    SUB(Register, Register),
    SBC(Register, Register),
    INC(Register, Register),
    DEC(Register, Register),
    MOV(Register, Register),
    MVN(Register, Register),
}
impl Instrinsic {
    #[inline(always)]
    pub fn op_regs(&self) -> (u8, u8, Option<u8>) {
        let make_byte = |a: &Register, b: &Register| -> u8 {
            let a = a.0 & 0x0F;
            let b = (b.0 << 4) & 0xF0;
            a | b
        };
        match self {
            Instrinsic::CLR(b) => (0b00100000, b.0 << 4, None),
            Instrinsic::SER(b) => (0b00110000, b.0 << 4, None),
            Instrinsic::NOT(b, a) => (0b00100001, make_byte(a, b), None),
            Instrinsic::TWO(b, a) => (0b00110001, make_byte(a, b), None),
            Instrinsic::AND(b, a) => (0b00100010, make_byte(a, b), None),
            Instrinsic::NND(b, a) => (0b00110010, make_byte(a, b), None),
            Instrinsic::ORR(b, a) => (0b00100011, make_byte(a, b), None),
            Instrinsic::NOR(b, a) => (0b00110011, make_byte(a, b), None),
            Instrinsic::XOR(b, a) => (0b00100100, make_byte(a, b), None),
            Instrinsic::XNR(b, a) => (0b00110100, make_byte(a, b), None),
            Instrinsic::ADD(b, a) => (0b00100101, make_byte(a, b), None),
            Instrinsic::ADC(b, a) => (0b00110101, make_byte(a, b), None),
            Instrinsic::SUB(b, a) => (0b00100110, make_byte(a, b), None),
            Instrinsic::SBC(b, a) => (0b00110110, make_byte(a, b), None),
            Instrinsic::INC(b, a) => (0b00100111, make_byte(a, b), None),
            Instrinsic::DEC(b, a) => (0b00110111, make_byte(a, b), None),
            Instrinsic::MOV(b, a) => (0b00101000, make_byte(a, b), None),
            Instrinsic::MVN(b, a) => (0b00111000, make_byte(a, b), None),
        }
    }
    
    pub fn assemble(&self, output: &mut Vec<u8>) {
        let (l, h, i) = self.op_regs();
        output.push(l);
        output.push(h);
        if let Some(i) = i {
            output.push(i);
        }
    }
}

fn parse(lexer: &mut Lexer, line: usize) -> Result<Instrinsic> {
    macro_rules! single_operand {
        ($name:expr, $instruction:expr) => {
            {
                let maybe_a = lexer.next();
                
                let a = match maybe_a {
                    Some(Token::Register(r)) => Register::from_u8(r),
                    _ => Err(format!("Line {}: Expected register after instruction: {}", line, $name))
                }?;
                
                Ok($instruction(a))
            }
        }
    }
    
    macro_rules! twin_operand {
        ($name:expr, $instruction:expr) => {
            {
                let maybe_a = lexer.next();
                let maybe_comma = lexer.next();
                let maybe_b = lexer.next();
                
                let a = match maybe_a {
                    Some(Token::Register(r)) => Register::from_u8(r),
                    _ => Err(format!("Line {}: Expected register after instruction: {}", line, $name))
                }?;
                
                match maybe_comma {
                    Some(Token::Comma) => Ok(()),
                    Some(Token::Register(_)) => Err(format!("Line {}: Expected ',' in between two operands for instruction: {}", line, $name)),
                    _ => Err(format!("Line {}: Expected a ',' after the first operand in instuction: {}", line, $name))
                }?;
                
                let b = match maybe_b {
                    Some(Token::Register(r)) => Register::from_u8(r),
                    _ => Err(format!("Line {}: Expected a second operand for instruction: {}", line, $name))
                }?;
                
                Ok($instruction(a, b))
            }
        }
    }
    
    if let Some(Token::Ident(ins)) = lexer.next() {
        let ins = ins.to_uppercase();
        match ins.as_str() {
            "CLR" => single_operand!("CLR", Instrinsic::CLR),
            "SER" => single_operand!("SER", Instrinsic::SER),
            "NOT" => twin_operand!("NOT", Instrinsic::NOT),
            "TWO" => twin_operand!("TWO", Instrinsic::TWO),
            "AND" => twin_operand!("AND", Instrinsic::AND),
            "NND" => twin_operand!("NND", Instrinsic::NND),
            "ORR" => twin_operand!("ORR", Instrinsic::ORR),
            "NOR" => twin_operand!("NOR", Instrinsic::NOR),
            "XOR" => twin_operand!("XOR", Instrinsic::XOR),
            "XNR" => twin_operand!("XNR", Instrinsic::XNR),
            "ADD" => twin_operand!("ADD", Instrinsic::ADD),
            "ADC" => twin_operand!("ADC", Instrinsic::ADC),
            "SUB" => twin_operand!("SUB", Instrinsic::SUB),
            "SBC" => twin_operand!("SBC", Instrinsic::SBC),
            "INC" => twin_operand!("INC", Instrinsic::INC),
            "DEC" => twin_operand!("DEC", Instrinsic::DEC),
            "MOV" => twin_operand!("MOV", Instrinsic::MOV),
            "MVN" => twin_operand!("MVN", Instrinsic::MVN),
            
            _ => Err(format!("Line {}: Invalid instruction: {}", line, ins))
        }
    } else {
        Err("Attempting to parse an empty string".into())
    }
}

pub fn assemble(source: &str, buffer: &mut Vec<u8>) -> Result<usize> {
    let start = buffer.len();
    
    for (line, string) in  source.lines().enumerate().filter(|(_, line)| !line.trim().is_empty()) {
        let mut lexer = Lexer::new(string);
        let intrinsic = parse(&mut lexer, line + 1)?;
        intrinsic.assemble(buffer);
    }
    
    Ok(buffer.len() - start)
}

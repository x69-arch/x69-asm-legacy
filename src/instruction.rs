use utils::{ToFromString, Iter};

#[derive(Clone, Copy, Debug)]
pub enum OperandMode {
    NoParams,                // NOP
    OneRegister,             // CLR R1
    OneOrTwoRegisters,       // INC R1, R1;  INC R1
    OneRegisterAndImmediate, // SET R1, 69
    TwoRegisters,            // LPC R0, R1
    TwoRegistersOrImmediate, // ADD R1, R2;  ADD R1, 69;  ADD R0, R1, 123
    
    // LongImmediate, // JMP 1234
    TwoRegistersOrLongImmediate, // JMP 1234;  JMP R1, R2
}

#[derive(Clone, Copy, Debug)]
pub enum RegisterMap {
    AB,
    BA,
    AA,
    // This is an odd case that will probably never exist
    // BB,
}

#[derive(Clone, Copy, Debug, ToFromString, Iter)]
pub enum Instruction {
    // ALU Operations
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
    SET,
    STN,
    CMP,
    
    // Memory operations
    LDR,
    SDR,
    
    // CPU Operations
    // RET,
    LPC,
    JMP,
    
    LLR,
    SLR,
    LSP,
    SSP,
    LADR,
    SADR,
    
    JMPZ,
    JMPNZ,
    JMPC,
    JMPNC,
    RJMPZ,
    RJMPNZ,
    RJMPC,
    RJMPNC,
    
    CALLZ,
    CALLNZ,
    CALLC,
    CALLNC,
    RCALLZ,
    RCALLNZ,
    RCALLC,
    RCALLNC,
}

// CPU Special Registers
const PC:  u8 = 0b00;
const LR:  u8 = 0b01;
const SP:  u8 = 0b10;
const ADR: u8 = 0b11;
// 0b01001100
const fn rw_builder(write: bool, register: u8) -> u8 {
    let mut rw = 0b01001000 | register;
    if write {
        rw |= 0b100;
    }
    rw
}

// ALU Flags
const ZERO:  u8 = 0;
const CARRY: u8 = 1;
// Twos compliment overflow
// const TWOS:  u8 = 2;

const fn jump_builder(relative: bool, check_true: bool, alu_flag: u8) -> u8 {
    let mut jmp = 0b01100000 | alu_flag << 2;
    if relative {
        jmp |= 0b00000010;
    }
    if check_true {
        jmp |= 0b00010000;
    }
    jmp
}

const fn call_builder(relative: bool, check_true: bool, alu_flag: u8) -> u8 {
    let mut call = 0b01100001 | alu_flag << 2;
    if relative {
        call |= 0b00000010;
    }
    if check_true {
        call |= 0b00010000;
    }
    call
}

impl Instruction {    
    #[inline(always)]
    pub fn assemble_info(&self) -> (u8, OperandMode, RegisterMap) {
        use OperandMode::*;
        use RegisterMap::*;
        match self {
            Self::NOP => (0b00101001, NoParams,    AB),
            Self::CLR => (0b00100000, OneRegister, AA),
            Self::SER => (0b00110000, OneRegister, AA),
            Self::NOT => (0b00100001, OneOrTwoRegisters,       BA),
            Self::TWO => (0b00110001, OneOrTwoRegisters,       BA),
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
            Self::SET => (0b00101001, OneRegisterAndImmediate, AA),
            Self::STN => (0b00111001, OneRegisterAndImmediate, AA),
            Self::CMP => (0b00101010, TwoRegisters,            AB),
            
            // Will need custom parsing eventually
            Self::LDR => (0b00010000, OneRegisterAndImmediate, AA),
            Self::SDR => (0b00010001, OneRegisterAndImmediate, AA),
            
            // Self::RET    => (0b01000000, NoParams, AB),
            Self::LPC => (rw_builder(false, PC), TwoRegisters, AB),
            Self::JMP => (rw_builder(true,  PC), TwoRegistersOrLongImmediate, AB),
            
            Self::LLR  => (rw_builder(false, LR),   TwoRegisters,                AB),
            Self::SLR  => (rw_builder(true,  LR),   TwoRegistersOrLongImmediate, AB),
            Self::LSP  => (rw_builder(false, SP),   TwoRegisters,                AB),
            Self::SSP  => (rw_builder(true,  SP),   TwoRegistersOrLongImmediate, AB),
            Self::LADR => (rw_builder(false, ADR),  TwoRegisters,                AB),
            Self::SADR => (rw_builder(true,  ADR),  TwoRegistersOrLongImmediate, AB),
            
            Self::JMPZ   => (jump_builder(false, true,  ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::JMPNZ  => (jump_builder(false, false, ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::JMPC   => (jump_builder(false, true,  CARRY), TwoRegistersOrLongImmediate, AB),
            Self::JMPNC  => (jump_builder(false, false, CARRY), TwoRegistersOrLongImmediate, AB),
            Self::RJMPZ  => (jump_builder(true,  true,  ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::RJMPNZ => (jump_builder(true,  false, ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::RJMPC  => (jump_builder(true,  true,  CARRY), TwoRegistersOrLongImmediate, AB),
            Self::RJMPNC => (jump_builder(true,  false, CARRY), TwoRegistersOrLongImmediate, AB),
            
            Self::CALLZ   => (call_builder(false, true,  ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::CALLNZ  => (call_builder(false, false, ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::CALLC   => (call_builder(false, true,  CARRY), TwoRegistersOrLongImmediate, AB),
            Self::CALLNC  => (call_builder(false, false, CARRY), TwoRegistersOrLongImmediate, AB),
            Self::RCALLZ  => (call_builder(true,  true,  ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::RCALLNZ => (call_builder(true,  false, ZERO),  TwoRegistersOrLongImmediate, AB),
            Self::RCALLC  => (call_builder(true,  true,  CARRY), TwoRegistersOrLongImmediate, AB),
            Self::RCALLNC => (call_builder(true,  false, CARRY), TwoRegistersOrLongImmediate, AB),
        }
    }
    
    pub fn print_usage(&self) {
        let name = self.to_str();
        let ops = self.assemble_info().1;
        
        // This exists so that instructions can override their usage printout in special cases
        #[allow(clippy::match_single_binding)]
        match self {
            _ => match ops {
                OperandMode::NoParams                => println!("{}",          name),
                OperandMode::OneRegister             => println!("{}\tR0",      name),
                OperandMode::OneOrTwoRegisters       => println!("{}\tR0 [R1]", name),
                OperandMode::OneRegisterAndImmediate => println!("{}\tR0, IM8", name),
                OperandMode::TwoRegisters            => println!("{}\tR0, R1",  name),
                OperandMode::TwoRegistersOrImmediate => {
                    println!("{}\tR0, IM8", name);
                    println!("{}\tR0, R1 [IM8]", name);
                },
                OperandMode::TwoRegistersOrLongImmediate => {
                    println!("{}\tR0, R1", name);
                    println!("{}\tIM16", name);
                },
            }
        };
    }
}

pub fn print_all() {
    println!("Instruction usage:");
    println!("R0: Register (0-15)");
    println!("[]: Optional parameter");
    Instruction::iter().for_each(Instruction::print_usage);
}

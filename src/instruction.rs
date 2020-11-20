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

#[derive(Clone, Copy, Debug, utils::ToFromString)]
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
    
    // CPU Operations
    LPC,
    JMP,
    RJMP,
    CALL,
    RCLL,
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
            
            Self::LPC =>  (0b01000100, TwoRegisters, AB),
            Self::JMP =>  (0b01000000, TwoRegistersOrLongImmediate, AB),
            Self::RJMP => (0b01000010, TwoRegistersOrLongImmediate, AB),
            Self::CALL => (0b01000001, TwoRegistersOrLongImmediate, AB),
            Self::RCLL => (0b01000011, TwoRegistersOrLongImmediate, AB),
        }
    }
}

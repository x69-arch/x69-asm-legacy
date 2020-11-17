
pub enum OperandMode {
    NoParams,                // NOP
    OneRegister,             // CLR R1
    OneOrTwoRegisters,       // INC R1, R1; INC R1
    OneRegisterAndImmediate, // SET R1, 69
    TwoRegisters,            // LPC R0, R1
    TwoRegistersOrImmediate, // ADD R1, R2; ADD R1, 69
}

pub enum RegisterMap {
    AB,
    BA,
    AA,
    // This is an odd case that will probably never exist
    // BB,
}

#[derive(Clone, Copy, Debug)]
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
}
impl Instruction {
    #[inline(always)]
    pub fn from_str(name: &str) -> Option<Self> {
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
            "SET" => Some(Self::SET),
            "STN" => Some(Self::STN),
            "CMP" => Some(Self::CMP),
            
            "LPC" => Some(Self::LPC),
            _ => None
        }
    }
    
    #[inline(always)]
    pub fn to_str(&self) -> &str {
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
            Self::SET => "SET",
            Self::STN => "STN",
            Self::CMP => "CMP",
            
            Self::LPC => "LPC",
        }
    }
    
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
            
            Self::LPC => (0b01000100, TwoRegisters, AB)
        }
    }
}

use std::collections::HashMap;
use crate::parse_utils::OpcodeFormat;
use crate::utils::i32_to_hex_string;

#[derive(Clone, Copy, Debug)]
pub struct OpcodeDetail {
    pub opcode: u8,
    pub format: OpcodeFormat,
}
impl OpcodeDetail {
    fn new(format: OpcodeFormat, opcode: u8) -> Self {
        Self { opcode, format }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Constant {
    SicString(String),
    Hex(i32),
}

impl Constant {
    pub fn get_len(&self) -> usize {
        match self {
            Constant::SicString(string) => {
                string.len()
            }
            Constant::Hex(val) => i32_to_hex_string(*val, 0).len().div_ceil(2)
        }
    }
}


#[derive(Debug)]
pub struct GlobalMap {
    pub opcode_map: HashMap<String, OpcodeDetail>,
    pub register_map: HashMap<String, i32>,
    pub label_map: HashMap<String, usize>,
    pub constant_map: HashMap<String, Constant>,
    pub literal_pool: Vec<Constant>,
    pub literal_map: HashMap<Constant, usize>,
}

impl GlobalMap {
    pub fn init() -> Self {
        let mut map = GlobalMap {
            opcode_map: HashMap::new(),
            register_map: HashMap::new(),
            label_map: HashMap::new(),
            constant_map: HashMap::new(),
            literal_pool: Vec::new(),
            literal_map: HashMap::new(),
        };

        let codes: Vec<(&'static str, OpcodeDetail)> = vec![
            Self::produce_map_member("ADD", OpcodeFormat::Three, 0x18),
            Self::produce_map_member("ADDF", OpcodeFormat::Three, 0x58),
            Self::produce_map_member("ADDR", OpcodeFormat::Two, 0x90),
            Self::produce_map_member("AND", OpcodeFormat::Three, 0x40),
            Self::produce_map_member("CLEAR", OpcodeFormat::Two, 0xB4),
            Self::produce_map_member("COMP", OpcodeFormat::Three, 0x28),
            Self::produce_map_member("COMPF", OpcodeFormat::Three, 0x88),
            Self::produce_map_member("COMPR", OpcodeFormat::Two, 0xA0),
            Self::produce_map_member("DIV", OpcodeFormat::Three, 0x24),
            Self::produce_map_member("DIVF", OpcodeFormat::Three, 0x64),
            Self::produce_map_member("DIVR", OpcodeFormat::Two, 0x9C),
            Self::produce_map_member("FIX", OpcodeFormat::One, 0xC4),
            Self::produce_map_member("FLOAT", OpcodeFormat::One, 0xC0),
            Self::produce_map_member("HIO", OpcodeFormat::One, 0xF4),
            Self::produce_map_member("J", OpcodeFormat::Three, 0x3C),
            Self::produce_map_member("JEQ", OpcodeFormat::Three, 0x30),
            Self::produce_map_member("JGT", OpcodeFormat::Three, 0x34),
            Self::produce_map_member("JLT", OpcodeFormat::Three, 0x38),
            Self::produce_map_member("JSUB", OpcodeFormat::Three, 0x48),
            Self::produce_map_member("LDA", OpcodeFormat::Three, 0x00),
            Self::produce_map_member("LDB", OpcodeFormat::Three, 0x68),
            Self::produce_map_member("LDCH", OpcodeFormat::Three, 0x50),
            Self::produce_map_member("LDF", OpcodeFormat::Three, 0x70),
            Self::produce_map_member("LDL", OpcodeFormat::Three, 0x08),
            Self::produce_map_member("LDS", OpcodeFormat::Three, 0x6C),
            Self::produce_map_member("LDT", OpcodeFormat::Three, 0x74),
            Self::produce_map_member("LDX", OpcodeFormat::Three, 0x04),
            Self::produce_map_member("LPS", OpcodeFormat::Three, 0xD0),
            Self::produce_map_member("MUL", OpcodeFormat::Three, 0x20),
            Self::produce_map_member("MULF", OpcodeFormat::Three, 0x60),
            Self::produce_map_member("MULR", OpcodeFormat::Two, 0x98),
            Self::produce_map_member("NORM", OpcodeFormat::One, 0xC8),
            Self::produce_map_member("OR", OpcodeFormat::Three, 0x44),
            Self::produce_map_member("RD", OpcodeFormat::Three, 0xD8),
            Self::produce_map_member("RMO", OpcodeFormat::Two, 0xAC),
            Self::produce_map_member("RSUB", OpcodeFormat::Three, 0x4C),
            Self::produce_map_member("SHIFTL", OpcodeFormat::Two, 0xA4),
            Self::produce_map_member("SHIFTR", OpcodeFormat::Two, 0xA8),
            Self::produce_map_member("SIO", OpcodeFormat::One, 0xF0),
            Self::produce_map_member("SSK", OpcodeFormat::Three, 0xEC),
            Self::produce_map_member("STA", OpcodeFormat::Three, 0x0C),
            Self::produce_map_member("STB", OpcodeFormat::Three, 0x78),
            Self::produce_map_member("STCH", OpcodeFormat::Three, 0x54),
            Self::produce_map_member("STF", OpcodeFormat::Three, 0x80),
            Self::produce_map_member("STI", OpcodeFormat::Three, 0xD4),
            Self::produce_map_member("STL", OpcodeFormat::Three, 0x14),
            Self::produce_map_member("STS", OpcodeFormat::Three, 0x7C),
            Self::produce_map_member("STSW", OpcodeFormat::Three, 0xE8),
            Self::produce_map_member("STT", OpcodeFormat::Three, 0x84),
            Self::produce_map_member("STX", OpcodeFormat::Three, 0x10),
            Self::produce_map_member("SUB", OpcodeFormat::Three, 0x1C),
            Self::produce_map_member("SUBF", OpcodeFormat::Three, 0x5C),
            Self::produce_map_member("SUBR", OpcodeFormat::Two, 0x94),
            Self::produce_map_member("SVC", OpcodeFormat::Two, 0xB0),
            Self::produce_map_member("TD", OpcodeFormat::Three, 0xE0),
            Self::produce_map_member("TIO", OpcodeFormat::One, 0xF8),
            Self::produce_map_member("TIX", OpcodeFormat::Three, 0x2C),
            Self::produce_map_member("TIXR", OpcodeFormat::Two, 0xB8),
            Self::produce_map_member("WD", OpcodeFormat::Three, 0xDC),
        ];
        codes.iter().for_each(|code| {
            map.opcode_map.insert(String::from(code.0), code.1);
        });
        let register_codes = vec![
            ("A", 0),
            ("X", 1),
            ("L", 2),
            ("PC", 8),
            ("SW", 9),
            ("B", 3),
            ("S", 4),
            ("T", 5),
            ("F", 6),
        ];
        register_codes.iter().for_each(|code| {
            map.register_map.insert(String::from(code.0), code.1);
        });
        map
    }
    fn produce_map_member(
        mnemonic: &'static str,
        format: OpcodeFormat,
        opcode: u8,
    ) -> (&'static str, OpcodeDetail) {
        (mnemonic, OpcodeDetail::new(format, opcode))
    }
    pub fn get_reg_value(&self, reg: impl Into<String>) -> i32 {
        let reg = reg.into();
        *self.register_map.get(&reg).unwrap_or_else(|| panic!("Invalid register {}", reg))
    }
    pub fn get_opcode_value(&self, opcode: impl Into<String>) -> OpcodeDetail {
        let opcode = opcode.into();
        *self.opcode_map.get(&opcode).unwrap_or_else(|| panic!("Invalid opcode {}", opcode))
    }
}

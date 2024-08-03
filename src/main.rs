use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::usize;
use std::string::String;

fn main() {
    let mut global_map = GlobalMap::init();
    let file = File::open("./src.sic").expect("Unable to open file");
    let lines = BufReader::new(file).lines();
    let mut loc = 0;
    let mut loc_inc = 0;
    let mut base = 0;
    let mut asm_lines: Vec<ASMLine> = Vec::new();

    //PASS 1
    for line in lines.into_iter().enumerate().map(|line| (line.0 + 1, line.1.expect("Couldn't read line"))) {
        let line_parts = line.1.split(' ').collect::<Vec<&str>>();
        let label = line_parts[0];
        let opcode = line_parts[1];
        let address = line_parts[2];

        println!("line {} :", line.0);
        let address_spec = if !address.is_empty() {
            parse_address(&global_map, address.to_string())
        } else {
            ParseAddressReturn::Address(0, AddressingModes::None)
        };
        let opcode_spec = if !opcode.is_empty() {
            parse_opcode(&global_map, opcode.to_string())
        } else {
            panic!("No opcode")
        };
        loc += loc_inc;
        println!("LOC: {}", i32_to_hex_string(loc as i32, 0));
        if line.0 == 1 {
            if let ParseOpcodeReturn::Directive(directive) = &opcode_spec {
                if directive == "START" {
                    if let ParseAddressReturn::Address(address, _) = &address_spec {
                        loc = *address;
                        if !label.is_empty() {
                            global_map.label_map.insert(label.to_string(), loc);
                        }
                    } else {
                        panic!("Invalid address for start")
                    }
                }
            } else {
                panic!("First line should be a START")
            }
        } else {
            loc_inc = get_loc_inc(&opcode_spec, &address_spec);
            if !label.is_empty() {
                global_map.label_map.insert(label.to_string(), loc);
                if let ParseAddressReturn::Constant(constant) = &address_spec {
                    global_map.constant_map.insert(label.to_string(), constant.clone());
                }
            }
        }
        let asm_line = ASMLine {
            pc: loc,
            opcode_spec: opcode_spec.clone(),
            address_spec: address_spec.clone(),
        };
        asm_lines.push(asm_line);
    }
    println!("{:#?}", global_map);

    //PASS 2
    for idx in 0..asm_lines.len() - 1 {
        let line = &asm_lines[idx];
        let next_line = &asm_lines[idx + 1];
        let next_pc = next_line.pc;
        let (object_code, new_base) = get_object_code(base, next_pc, &global_map, &line);
        base = new_base;
        match &object_code {
            None => {
                println!("line {} Object code: None", idx + 1);
            }
            Some(object_code) => {
                println!("line {} Object code: {}", idx + 1, object_code);
            }
        }
    }
}


fn get_object_code(base: usize, pc: usize, global_map: &GlobalMap, asm_line: &ASMLine) -> (Option<String>, usize) {
    let opcode_spec = &asm_line.opcode_spec;
    let address_spec = &asm_line.address_spec;
    let mut base = base;
    let mut nixbpe = Nixbpe::new();
    let mut opcode_code = String::new();
    let mut address_code = String::new();
    let (code_len, address_len) = match &opcode_spec {
        ParseOpcodeReturn::Directive(_) => {
            (0, 12)
        }
        ParseOpcodeReturn::Opcode(_, format) => {
            match format {
                OpcodeFormat::None => {
                    panic!("what?")
                }
                OpcodeFormat::One => {
                    (8, 0)
                }
                OpcodeFormat::Two => {
                    (16, 8)
                }
                OpcodeFormat::Three => {
                    (24, 12)
                }
                OpcodeFormat::Four => {
                    (32, 20)
                }
            }
        }
    };
    let is_extended = matches!(opcode_spec, ParseOpcodeReturn::Opcode(_, OpcodeFormat::Four));

    match address_spec {
        ParseAddressReturn::Address(address, addressing_mode) => {
            address_code = i32_to_bin_string(*address as i32, address_len);
            match addressing_mode {
                AddressingModes::None => {
                    nixbpe.set_direct();
                }
                AddressingModes::Immediate => {
                    nixbpe.set_immediate();
                }
                AddressingModes::Indirect => {
                    nixbpe.set_indirect();
                }
                AddressingModes::Indexed => {
                    nixbpe.set_indexed();
                }
                AddressingModes::Direct => {
                    nixbpe.set_direct();
                }
            }
        }
        ParseAddressReturn::Label(label, addressing_mode) => {
            let label_loc = global_map.label_map.get(label).unwrap_or_else(|| panic!("Invalid label {}", label));
            if !is_extended {
                let mut disp = *label_loc as i32 - pc as i32;
                if disp >= -2048 && disp <= 2047 {
                    nixbpe.set_pc_relative();
                    if disp < 0 {
                        disp = 4096 + disp;
                    }
                } else {
                    disp = *label_loc as i32 - base as i32;
                    if disp >= 0 && disp < 4096 {
                        nixbpe.set_base_relative();
                        disp = *label_loc as i32 - base as i32;
                    } else {
                        panic!("Displacement  out of bounds")
                    }
                }
                address_code = i32_to_bin_string(disp, 12);
            } else {
                address_code = i32_to_bin_string(*label_loc as i32, 20);
            }
            match addressing_mode {
                AddressingModes::None => {
                    nixbpe.set_direct();
                }
                AddressingModes::Immediate => {
                    nixbpe.set_immediate();
                }
                AddressingModes::Indirect => {
                    nixbpe.set_indirect();
                }
                AddressingModes::Indexed => {
                    nixbpe.set_direct();
                    nixbpe.set_indexed();
                }
                AddressingModes::Direct => {
                    nixbpe.set_direct();
                }
            }
        }
        ParseAddressReturn::Literal(literal) => {
            todo!("Add literal logic")
        }
        ParseAddressReturn::Constant(constant) => {
            address_code = match constant {
                Constant::String(string) => {
                    string.as_bytes().iter().fold(String::new(), |acc, e| acc + i32_to_hex_string(*e as i32, 2).as_str())
                }
                Constant::Hex(hex) => {
                    let t = i32_to_hex_string(*hex, 0);
                    if t.len() % 2 != 0 {
                        format!("0{}", t)
                    } else {
                        t
                    }
                }
            }
        }
    }

    match opcode_spec {
        ParseOpcodeReturn::Directive(directive) => {
            if directive == "BASE" {
                if let ParseAddressReturn::Label(label, _) = address_spec {
                    base = *global_map.label_map.get(label).unwrap_or_else(|| panic!("Invalid label {}", label));
                    return (None, base);
                } else if let ParseAddressReturn::Address(address, _) = address_spec {
                    base = *address;
                    return (None, base);
                }
                panic!("provide label or address for base")
            }
            if directive == "BYTE" {
                return (Some(address_code), base);
            }
            return (None, base);
        }
        ParseOpcodeReturn::Opcode(opcode, format) => {
            let opcode_detail = global_map.get_opcode_value(opcode);
            let opcode = opcode_detail.opcode;
            match format {
                OpcodeFormat::None => {
                    panic!("What?")
                }
                OpcodeFormat::Four => {
                    nixbpe.set_extended();
                    opcode_code = i32_to_bin_string((opcode >> 2) as i32, 6);
                }
                OpcodeFormat::Three => {
                    opcode_code = i32_to_bin_string((opcode >> 2) as i32, 6);
                }
                OpcodeFormat::Two => {
                    opcode_code = i32_to_bin_string(opcode as i32, 8);
                    return (Some((i32_to_hex_string(opcode as i32, 2)) + i32_to_hex_string(bin_string_to_i32(address_code), 2).as_str()), base);
                }
                OpcodeFormat::One => {
                    return (Some(i32_to_hex_string(opcode as i32, 2)), base)
                }
            }
        }
    }

    let object_code = i32_to_hex_string(bin_string_to_i32(opcode_code + nixbpe.as_bin_string().as_str() + &*address_code), code_len / 4);
    return (Some(object_code), base);
}


struct ASMLine {
    pc: usize,
    opcode_spec: ParseOpcodeReturn,
    address_spec: ParseAddressReturn,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum AddressingModes {
    None,
    Immediate,
    Indirect,
    Indexed,
    Direct,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum OpcodeFormat {
    None,
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum OpcodeModifier {
    Fourth,
    None,
}

#[derive(Clone, Copy, Debug)]
struct OpcodeDetail {
    opcode: u8,
    format: OpcodeFormat,
}
impl OpcodeDetail {
    fn new(format: OpcodeFormat, opcode: u8) -> Self {
        Self { opcode, format }
    }
}
#[derive(Clone, Debug, PartialEq)]
enum Constant {
    String(String),
    Hex(i32),
}


#[derive(Debug)]
struct GlobalMap {
    opcode_map: HashMap<String, OpcodeDetail>,
    register_map: HashMap<String, i32>,
    label_map: HashMap<String, usize>,
    constant_map: HashMap<String, Constant>,
}

impl GlobalMap {
    fn init() -> Self {
        let mut map = GlobalMap {
            opcode_map: HashMap::new(),
            register_map: HashMap::new(),
            label_map: HashMap::new(),
            constant_map: HashMap::new(),
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
    fn get_reg_value(&self, reg: impl Into<String>) -> i32 {
        let reg = reg.into();
        *self.register_map.get(&reg).unwrap_or_else(|| panic!("Invalid register {}", reg))
    }
    fn get_opcode_value(&self, opcode: impl Into<String>) -> OpcodeDetail {
        let opcode = opcode.into();
        *self.opcode_map.get(&opcode).unwrap_or_else(|| panic!("Invalid opcode {}", opcode))
    }
}


#[derive(Clone, Debug)]
enum ParseAddressReturn {
    Address(usize, AddressingModes),
    Label(String, AddressingModes),
    Literal(Constant),
    Constant(Constant),
}


#[derive(Clone, Debug)]
enum ParseOpcodeReturn {
    Directive(String),
    Opcode(String, OpcodeFormat),
}


struct Nixbpe {
    n: bool,
    i: bool,
    x: bool,
    b: bool,
    p: bool,
    e: bool,
}

impl Nixbpe {
    fn new() -> Self {
        Self {
            n: false,
            i: false,
            x: false,
            b: false,
            p: false,
            e: false,
        }
    }
    fn set_direct(&mut self) {
        self.n = true;
        self.i = true;
    }
    fn set_indirect(&mut self) {
        self.n = true;
        self.i = false;
    }
    fn set_immediate(&mut self) {
        self.n = false;
        self.i = true;
    }
    fn set_indexed(&mut self) {
        self.x = true;
    }
    fn set_base_relative(&mut self) {
        self.b = true;
    }
    fn set_pc_relative(&mut self) {
        self.p = true;
    }
    fn set_extended(&mut self) {
        self.e = true;
    }
    fn as_bin_string(&self) -> String {
        format!("{}{}{}{}{}{}", self.n as i32, self.i as i32, self.x as i32, self.b as i32, self.p as i32, self.e as i32)
    }
}


fn parse_opcode(global_map: &GlobalMap, opcode: String) -> ParseOpcodeReturn {
    let is_directive = matches!(opcode.as_str(), "BASE" | "START" | "RESW" | "RESB" | "WORD" | "BYTE" |"END");
    return if is_directive {
        ParseOpcodeReturn::Directive(opcode)
    } else {
        let mut opcode = opcode;
        let first_char = get_nth_char(&opcode, 1);
        if let Ok(first_char) = first_char {
            if first_char == '+' {
                let opcode_format = OpcodeFormat::Four;
                opcode = opcode.chars().skip(1).collect();
                return ParseOpcodeReturn::Opcode(opcode, opcode_format);
            }
        }
        let opcode_detail = global_map.get_opcode_value(&opcode);
        let opcode_format = opcode_detail.format;
        ParseOpcodeReturn::Opcode(opcode, opcode_format)
    };
}


fn get_loc_inc(opcode_spec: &ParseOpcodeReturn, address_specs: &ParseAddressReturn) -> usize {
    return match opcode_spec {
        ParseOpcodeReturn::Directive(directive) => {
            match directive.as_str() {
                "START" => {
                    match address_specs {
                        ParseAddressReturn::Address(address, _) => {
                            *address
                        }
                        _ => panic!("START needs a positive integer value")
                    }
                }
                "RESW" => {
                    match address_specs {
                        ParseAddressReturn::Address(address, _) => {
                            address * 3
                        }
                        _ => panic!("RESW needs a positive integer value")
                    }
                }
                "RESB" => {
                    match address_specs {
                        ParseAddressReturn::Address(address, _) => {
                            //we will increase loc as binary parse as hex when required
                            *address
                        }
                        _ => panic!("RESB needs a positive integer value")
                    }
                }
                "BASE" => 0,
                "WORD" => 3,
                "BYTE" => {
                    match address_specs {
                        ParseAddressReturn::Constant(constant) => {
                            match constant {
                                Constant::String(string) => {
                                    string.len()
                                }
                                Constant::Hex(val) => i32_to_hex_string(*val, 0).len().div_ceil(2)
                            }
                        }
                        _ => panic!("BYTE needs a string or hex value")
                    }
                }
                "END" => 0,
                _ => panic!("Unknown directive {}", directive)
            }
        }
        ParseOpcodeReturn::Opcode(_, format) => {
            match format {
                OpcodeFormat::None => {
                    panic!("Cannot parse none as opcode format");
                }
                OpcodeFormat::One => {
                    1
                }
                OpcodeFormat::Two => {
                    2
                }
                OpcodeFormat::Three => {
                    3
                }
                OpcodeFormat::Four => {
                    4
                }
            }
        }
    };
}

fn get_nth_char(word: impl Into<String>, n: usize) -> Result<char, String> {
    let as_vec = word.into().chars().collect::<Vec<char>>();
    if as_vec.is_empty() {
        Err(String::from("Empty string"))
    } else if as_vec.len() < n {
        Err(String::from("Out of bounds"))
    } else {
        Ok(as_vec[n - 1])
    }
}

fn parse_address(global_map: &GlobalMap, address: String) -> ParseAddressReturn {
    let mut addressing_mode = AddressingModes::Direct;
    let mut address = address;
    let comma_splitter_address: Vec<&str> = address.split(",").collect();
    if comma_splitter_address.len() == 2 {
        let r1 = comma_splitter_address[0];
        let r2 = comma_splitter_address[1];
        if global_map.register_map.contains_key(r1) {
            address = i32_to_hex_string(global_map.get_reg_value(r1), 0) + &*i32_to_hex_string(global_map.get_reg_value(r2), 0);
            return ParseAddressReturn::Address(string_to_usize(&address), AddressingModes::None);
        } else if r2 == "X" {
            addressing_mode = AddressingModes::Indexed;
            // address = address.chars().enumerate().filter(|x| { x.0 <= address.len() - 2 }).map(|x| x.1).collect();
            return label_or_address(global_map, &(r1.to_string()), AddressingModes::Indexed);
        } else {
            panic!("Invalid address {}", address)
        }
    } else if comma_splitter_address.len() == 1 {
        let address_as_vec: Vec<char> = comma_splitter_address[0].chars().collect();
        let first_char = address_as_vec[0];
        let second_char = if address_as_vec.len() > 1 { address_as_vec[1] } else { ' ' };
        let third_char = if address_as_vec.len() > 2 { address_as_vec[2] } else { ' ' };
        let last_char = address_as_vec[address_as_vec.len() - 1];
        let mut address = comma_splitter_address[0].to_string();
        //TODO logic for literals
        if first_char == '=' {
            if second_char != 'X' && first_char != 'C' {
                panic!("Invalid literal {}", address)
            }
            if third_char != '\'' && last_char != '\'' {
                panic!("Invalid literal {}", address)
            }
            let literal = parse_constant(second_char,
                                         address_as_vec.iter().enumerate().filter(|indexed_item| indexed_item.0 > 2 && indexed_item.0 < address_as_vec.len() - 1).map(|indexed_item| indexed_item.1).collect());
            return ParseAddressReturn::Literal(literal);
        }
        if (first_char == 'X' || first_char == 'C') && second_char == '\'' && last_char == '\'' {
            let constant = parse_constant(first_char, address_as_vec.iter().enumerate().filter(|indexed_item|
            indexed_item.0 > 1 && indexed_item.0 < address_as_vec.len() - 1
            ).map(|indexed_item| indexed_item.1).collect());
            return ParseAddressReturn::Constant(constant);
        }
        if first_char == '#' {
            addressing_mode = AddressingModes::Immediate;
            address = address_as_vec.iter().skip(1).collect();
        } else if first_char == '@' {
            addressing_mode = AddressingModes::Indirect;
            address = address_as_vec.iter().skip(1).collect();
        }
        return label_or_address(global_map, &address, addressing_mode);
    } else {
        panic!("Invalid address {}", address)
    }
}

fn label_or_address(global_map: &GlobalMap, address: &String, addressing_modes: AddressingModes) -> ParseAddressReturn {
    if is_valid_decimal_string(address) {
        ParseAddressReturn::Address(string_to_usize(address), addressing_modes)
    } else if global_map.register_map.contains_key(address) {
        let t = hex_string_to_i32(global_map.get_reg_value(address).to_string() + "0") as usize;
        ParseAddressReturn::Address(t, addressing_modes)
    } else {
        ParseAddressReturn::Label(address.to_string(), addressing_modes)
    }
}

fn parse_constant(qualifier: char, constant: String) -> Constant {
    match qualifier {
        'X' => Constant::Hex(hex_string_to_i32(constant)),
        'C' => Constant::String(constant),
        _ => panic!("Invalid constant {}", constant)
    }
}

fn string_to_usize(string: impl Into<String>) -> usize {
    let string = string.into();
    string.parse::<usize>().unwrap_or_else(|_| panic!("couldn't parse as usize, string {}", string))
}

fn is_valid_decimal_string(bin_string: impl Into<String>) -> bool {
    bin_string.into().parse::<i32>().is_ok()
}


fn hex_string_to_i32(hex_string: String) -> i32 {
    i32::from_str_radix(&hex_string, 16).unwrap_or_else(|_| panic!("Invalid hex string {}", hex_string))
}

fn i32_to_hex_string(val: i32, len: usize) -> String {
    let mut t = format!("{:X}", val);
    while t.len() < len {
        t = format!("0{}", t);
    };
    return t;
}

fn i32_to_bin_string(val: i32, len: usize) -> String {
    let mut t = format!("{:b}", val);
    while t.len() < len {
        t = format!("0{}", t);
    }
    return t;
}

fn bin_string_to_i32(bin_string: String) -> i32 {
    i32::from_str_radix(&bin_string, 2).unwrap_or_else(|_| panic!("Invalid binary string {}", bin_string))
}
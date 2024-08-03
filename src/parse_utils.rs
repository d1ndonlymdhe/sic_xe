use crate::global_map::{Constant, GlobalMap};
use crate::nixbpe::Nixbpe;
use crate::utils::{bin_string_to_i32, get_nth_char, hex_string_to_i32, i32_to_bin_string, i32_to_hex_string, is_valid_decimal_string, string_to_usize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddressingModes {
    None,
    Immediate,
    Indirect,
    Indexed,
    Direct,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpcodeFormat {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddressSpec {
    Address(usize, AddressingModes),
    Label(String, AddressingModes),
    Literal(Constant),
    Constant(Constant),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OpcodeSpec {
    Directive(String),
    Opcode(String, OpcodeFormat),
}

pub struct ASMLine {
    pub opcode_spec: OpcodeSpec,
    pub address_spec: AddressSpec,
}

pub fn parse_opcode(global_map: &GlobalMap, opcode: String) -> OpcodeSpec {
    let is_directive = matches!(opcode.as_str(), "BASE" | "START" | "RESW" | "RESB" | "WORD" | "BYTE" |"END");
    return if is_directive {
        OpcodeSpec::Directive(opcode)
    } else {
        let mut opcode = opcode;
        let first_char = get_nth_char(&opcode, 1);
        if let Ok(first_char) = first_char {
            if first_char == '+' {
                let opcode_format = OpcodeFormat::Four;
                opcode = opcode.chars().skip(1).collect();
                return OpcodeSpec::Opcode(opcode, opcode_format);
            }
        }
        let opcode_detail = global_map.get_opcode_value(&opcode);
        let opcode_format = opcode_detail.format;
        OpcodeSpec::Opcode(opcode, opcode_format)
    };
}

pub fn get_loc_inc(opcode_spec: &OpcodeSpec, address_specs: &AddressSpec) -> usize {
    return match opcode_spec {
        OpcodeSpec::Directive(directive) => {
            match directive.as_str() {
                "START" => {
                    match address_specs {
                        AddressSpec::Address(address, _) => {
                            *address
                        }
                        _ => panic!("START needs a positive integer value")
                    }
                }
                "RESW" => {
                    match address_specs {
                        AddressSpec::Address(address, _) => {
                            address * 3
                        }
                        _ => panic!("RESW needs a positive integer value")
                    }
                }
                "RESB" => {
                    match address_specs {
                        AddressSpec::Address(address, _) => {
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
                        AddressSpec::Constant(constant) => {
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
        OpcodeSpec::Opcode(_, format) => {
            match format {
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

pub fn parse_address(global_map: &GlobalMap, address: String) -> AddressSpec {
    let mut addressing_mode = AddressingModes::Direct;
    let mut address = address;
    let comma_splitter_address: Vec<&str> = address.split(',').collect();
    if comma_splitter_address.len() == 2 {
        let r1 = comma_splitter_address[0];
        let r2 = comma_splitter_address[1];
        if global_map.register_map.contains_key(r1) {
            address = i32_to_hex_string(global_map.get_reg_value(r1), 0) + &*i32_to_hex_string(global_map.get_reg_value(r2), 0);
            AddressSpec::Address(string_to_usize(&address), AddressingModes::None)
        } else if r2 == "X" {
            label_or_address(global_map, &(r1.to_string()), AddressingModes::Indexed)
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
            return AddressSpec::Literal(literal);
        }
        if (first_char == 'X' || first_char == 'C') && second_char == '\'' && last_char == '\'' {
            let constant = parse_constant(first_char, address_as_vec.iter().enumerate().filter(|indexed_item|
            indexed_item.0 > 1 && indexed_item.0 < address_as_vec.len() - 1
            ).map(|indexed_item| indexed_item.1).collect());
            return AddressSpec::Constant(constant);
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

pub fn parse_constant(qualifier: char, constant: String) -> Constant {
    match qualifier {
        'X' => Constant::Hex(hex_string_to_i32(constant)),
        'C' => Constant::String(constant),
        _ => panic!("Invalid constant {}", constant)
    }
}

fn label_or_address(global_map: &GlobalMap, address: &String, addressing_modes: AddressingModes) -> AddressSpec {
    if is_valid_decimal_string(address) {
        AddressSpec::Address(string_to_usize(address), addressing_modes)
    } else if global_map.register_map.contains_key(address) {
        let t = hex_string_to_i32(global_map.get_reg_value(address).to_string() + "0") as usize;
        AddressSpec::Address(t, addressing_modes)
    } else {
        AddressSpec::Label(address.to_string(), addressing_modes)
    }
}


pub fn get_object_code(base: usize, pc: usize, global_map: &GlobalMap, asm_line: &ASMLine) -> (Option<String>, usize, usize) {
    let opcode_spec = &asm_line.opcode_spec;
    let address_spec = &asm_line.address_spec;
    let pc = pc + get_loc_inc(opcode_spec, address_spec);
    let mut base = base;
    let mut nixbpe = Nixbpe::new();
    let opcode_code: String;
    let address_code: String;
    let (code_len, address_len) = match &opcode_spec {
        OpcodeSpec::Directive(_) => {
            (0, 12)
        }
        OpcodeSpec::Opcode(_, format) => {
            match format {
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
    let is_extended = matches!(opcode_spec, OpcodeSpec::Opcode(_, OpcodeFormat::Four));

    match address_spec {
        AddressSpec::Address(address, addressing_mode) => {
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
        AddressSpec::Label(label, addressing_mode) => {
            let label_loc = global_map.label_map.get(label).unwrap_or_else(|| panic!("Invalid label {}", label));
            if !is_extended {
                let mut disp = *label_loc as i32 - pc as i32;
                if (-2048..=2047).contains(&disp) {
                    nixbpe.set_pc_relative();
                    if disp < 0 {
                        disp += 4096;
                    }
                } else {
                    disp = *label_loc as i32 - base as i32;
                    if (0..4096).contains(&disp) {
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
        AddressSpec::Literal(_) => {
            todo!("Add literal logic")
        }
        AddressSpec::Constant(constant) => {
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

    return match opcode_spec {
        OpcodeSpec::Directive(directive) => {
            if directive == "BASE" {
                if let AddressSpec::Label(label, _) = address_spec {
                    base = *global_map.label_map.get(label).unwrap_or_else(|| panic!("Invalid label {}", label));
                    return (None, base, pc);
                } else if let AddressSpec::Address(address, _) = address_spec {
                    base = *address;
                    return (None, base, pc);
                }
                panic!("provide label or address for base")
            }
            if directive == "BYTE" {
                return (Some(address_code), base, pc);
            }
            if directive == "WORD" {
                return (Some(i32_to_hex_string(bin_string_to_i32(address_code), 6)), base, pc);
            }
            (None, base, pc)
        }
        OpcodeSpec::Opcode(opcode, format) => {
            let opcode_detail = global_map.get_opcode_value(opcode);
            let opcode = opcode_detail.opcode;
            match format {
                OpcodeFormat::Four => {
                    nixbpe.set_extended();
                    opcode_code = i32_to_bin_string((opcode >> 2) as i32, 6);
                    let object_code = i32_to_hex_string(bin_string_to_i32(opcode_code + nixbpe.as_bin_string().as_str() + &*address_code), code_len / 4);
                    (Some(object_code), base, pc)
                }
                OpcodeFormat::Three => {
                    opcode_code = i32_to_bin_string((opcode >> 2) as i32, 6);
                    let object_code = i32_to_hex_string(bin_string_to_i32(opcode_code + nixbpe.as_bin_string().as_str() + &*address_code), code_len / 4);
                    (Some(object_code), base, pc)
                }
                OpcodeFormat::Two => {
                    (Some((i32_to_hex_string(opcode as i32, 2)) + i32_to_hex_string(bin_string_to_i32(address_code), 2).as_str()), base, pc)
                }
                OpcodeFormat::One => {
                    (Some(i32_to_hex_string(opcode as i32, 2)), base, pc)
                }
            }
        }
    };
}


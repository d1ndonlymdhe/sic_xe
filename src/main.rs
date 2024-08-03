use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::global_map::*;
use crate::utils::*;
use crate::parse_utils::*;

mod utils;
mod global_map;
mod nixbpe;
mod parse_utils;

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
            AddressSpec::Address(0, AddressingModes::None)
        };
        let opcode_spec = if !opcode.is_empty() {
            parse_opcode(&global_map, opcode.to_string())
        } else {
            panic!("No opcode")
        };
        loc += loc_inc;
        println!("LOC: {}", i32_to_hex_string(loc as i32, 0));
        if line.0 == 1 {
            if let OpcodeSpec::Directive(directive) = &opcode_spec {
                if directive == "START" {
                    if let AddressSpec::Address(address, _) = &address_spec {
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
                if let AddressSpec::Constant(constant) = &address_spec {
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
        let (object_code, new_base) = get_object_code(base, next_pc, &global_map, line);
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






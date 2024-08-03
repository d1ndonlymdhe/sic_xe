use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;

use crate::global_map::*;
use crate::parse_utils::*;

mod utils;
mod global_map;
mod nixbpe;
mod parse_utils;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: ./sic_xe_assembler <filename>")
    }
    let filename = &args[1];
    let mut global_map = GlobalMap::init();
    let file = File::open(filename).expect("Unable to open file");
    let lines = BufReader::new(file).lines();
    let mut loc = 0;
    let mut loc_inc = 0;
    let mut asm_lines: Vec<ASMLine> = Vec::new();
    //PASS 1
    for line in lines.into_iter().enumerate().map(|line| (line.0 + 1, line.1.expect("Couldn't read line"))) {
        let line_parts = line.1.split(' ').collect::<Vec<&str>>();
        let label = line_parts[0];
        let opcode = line_parts[1];
        let address = line_parts[2];

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
            opcode_spec: opcode_spec.clone(),
            address_spec: address_spec.clone(),
        };
        asm_lines.push(asm_line);
    }
    //PASS 2
    let mut base = 0;
    let mut pc = 0;

    for line in asm_lines.iter().take(asm_lines.len() - 1).enumerate() {
        let idx = line.0;
        let line = line.1;
        let (object_code, new_base, new_pc) = get_object_code(base, pc, &global_map, line);
        base = new_base;
        pc = new_pc;
        match &object_code {
            None => {
                println!("Line : {} Object code: None", idx + 1);
            }
            Some(object_code) => {
                println!("Line : {} Object code: {}", idx + 1, object_code);
            }
        }
    }
}






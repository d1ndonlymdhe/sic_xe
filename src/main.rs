use std::{env, panic};
use crate::batch::batch_mode;
use crate::global_map::*;
use crate::interactive::interactive_mode;
use crate::parse_utils::*;

mod utils;
mod global_map;
mod nixbpe;
mod parse_utils;
mod interactive;
mod batch;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: ./sic_xe_assembler <filename>||-i")
    }
    let filename = &args[1];
    let lines = if filename == "-i" {
        interactive_mode()
    } else {
        batch_mode(filename)
    };

    let mut global_map = GlobalMap::init();
    let mut loc = 0;
    let mut loc_inc = 0;
    let mut asm_lines: Vec<ASMLine> = Vec::new();
    // let mut start: usize = 0;
    //PASS 1
    for line in lines.into_iter().enumerate().map(|line| (line.0 + 1, line.1)) {
        let line_parts = line.1.split(' ').collect::<Vec<&str>>();
        let label: &str;
        let opcode: &str;
        let address: &str;
        if line_parts.len() >= 3 {
            label = line_parts[0];
            opcode = line_parts[1];
            address = line_parts[2];
        } else if line_parts.len() == 2 {
            let t = line_parts[0];
            if panic::catch_unwind(|| parse_opcode(&global_map, t.to_string())).is_ok()
            {
                label = "";
                opcode = t;
                address = line_parts[1];
            } else {
                label = t;
                opcode = line_parts[1];
                address = "";
            }
        } else {
            label = "";
            opcode = line_parts[0];
            address = "";
        }
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
                        // start = *address;
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
            if let OpcodeSpec::Directive(directive) = &opcode_spec{
                if directive == "LTORG" {
                    global_map.literal_pool = Vec::new();
                    for lit in &global_map.literal_pool {
                        global_map.literal_map.insert(lit.clone(),loc);
                        loc += lit.get_len();
                    }
                }
            }
            if let AddressSpec::Literal(literal) = &address_spec{
                global_map.literal_pool.push(literal.clone());
            }

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

    for asm in asm_lines.clone(){
        println!("{:#?}",asm);
    }

    //PASS 2
    let mut base = 0;
    let mut pc = 0;

    for line in asm_lines.iter().take(asm_lines.len() - 1).enumerate() {
        let idx = line.0;
        let line = line.1;
        let ret = get_object_code(base, pc, &global_map, line);
        let (object_code, new_base, new_pc) = ret;
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






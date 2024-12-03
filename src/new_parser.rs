use crate::utils::{hex_string_to_i32, i32_to_hex_string, Stack};
use crate::global_map::GlobalMap;
use crate::utils::{get_nth_char, substring};
use std::cmp::Ordering;
use std::str::Chars;
use std::{panic};
use crate::parse_utils::OpcodeFormat;

#[derive(Debug)]
pub struct ASMLine {
    label: Label,
    opcode: Opcode,
    operand: Operand,
    size: i32,
}
#[derive(Debug)]
pub enum Label {
    None,
    Val(String),
}

#[derive(Debug)]
pub enum Opcode {
    Simple(String),
    Extended(String),
    Directive(String),
}

#[derive(Debug)]
pub enum Operand {
    None,
    Literal(StringOrNum),
    Constant(StringOrNum),
    Expression(Vec<ParsedExpressionPart>),
    Symbol(Symbol),
    Address(Address),
}
#[derive(Debug)]
pub enum Symbol {
    PC
}
#[derive(Debug)]
pub enum Address {
    Indexed(StringOrNum),
    Indirect(StringOrNum),
    Immediate(StringOrNum),
    Simple(StringOrNum),
    Register(RegisterAddress),
}


type Register = String;
#[derive(Debug)]
pub enum RegisterAddress {
    Single(Register),
    Double(Register, Register),
}

#[derive(Clone, Debug)]
pub enum StringOrNum {
    Str(String),
    Num(i32),
}

impl StringOrNum {
    pub fn new(string: impl Into<String>) -> StringOrNum {
        let string = string.into();
        let res = i32::from_str_radix(string.to_lowercase().as_str(), 16);
        match res {
            Ok(num) => StringOrNum::Num(num),
            Err(_) => StringOrNum::Str(string),
        }
    }
    pub fn get_size_in_bytes(&self) -> i32 {
        match self {
            StringOrNum::Str(str) => {
                str.len() as i32
            }
            StringOrNum::Num(num) => {
                (i32_to_hex_string(*num, 0).len() as f32 / 2f32).ceil() as i32
            }
        }
    }
}

#[derive(Debug)]
pub enum ParsedExpressionPart {
    Operand(StringOrNum),
    Operator(Operator),
}
#[derive(Clone, Debug)]
pub enum Operator {
    Val(String),
    None,
}
impl Operator {
    pub fn unwrap(&self) -> String {
        match self {
            Operator::Val(string) => String::from(string),
            Operator::None => {
                panic!("Tried to call unwrap on Operator::None")
            }
        }
    }
}

impl Operator {
    pub fn int_order(&self) -> i32 {
        match self {
            Operator::Val(operator) => match operator.as_str() {
                "+" => 1,
                "-" => 1,
                "%" => 2,
                "/" => 2,
                "^" => 3,
                "(" => 4,
                ")" => 5,
                _ => panic!("Invalid operator {}", operator),
            },
            Operator::None => 0,
        }
    }
}

impl PartialEq for Operator {
    fn eq(&self, other: &Self) -> bool {
        self.int_order() == other.int_order()
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let other_val = other.int_order();
        let self_val = self.int_order();
        Some(self_val.cmp(&other_val))
    }
}

pub enum ValidOpcode {
    Val(String),
    Extended(String),
}
pub enum ValidDirective {
    Val(String),
}


pub struct Parser {}
impl Parser {
    pub fn parse_asm_line(global_map: &mut GlobalMap, line: String) -> ASMLine {
        let line_parts = line.split(' ').collect::<Vec<&str>>();
        let label: &str;
        let opcode: &str;
        let address: &str;
        if line_parts.len() >= 3 {
            label = line_parts[0];
            opcode = line_parts[1];
            address = line_parts[2];
        } else if line_parts.len() == 2 {
            let t = line_parts[0];
            if Self::is_opcode(t, global_map) || Self::is_directive(t)
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
        let asm_label = Self::parse_label(label);
        let asm_opcode = if let Ok(opcode) = Self::parse_opcode(opcode) {
            opcode
        } else {
            panic!("Opcode is empty")
        };
        let asm_operand = if let Ok(operand) = Self::parse_operand(address) {
            operand
        } else {
            panic!("invalid operand")
        };
        let size;
        if let Operand::Literal(literal) = &asm_operand {
            global_map.new_lit_pool.push(literal.clone())
        }
        match &asm_opcode {
            Opcode::Simple(mnemonic) => {
                size = match global_map.get_opcode_value(mnemonic).format {
                    OpcodeFormat::One => { 1 }
                    OpcodeFormat::Two => { 2 }
                    OpcodeFormat::Three => { 3 }
                    OpcodeFormat::Four => { 3 }
                }
            }
            Opcode::Extended(_) => {
                size = 4
            }
            Opcode::Directive(directive) => {
                size = match directive.as_str() {
                    "LTORG" => {
                        let s = global_map.literal_pool.iter()
                            .fold(0, |acc, e| {
                                return acc + e.get_len() as i32;
                            });
                        global_map.literal_pool = Vec::new();
                        s
                    }
                    _ => { 0 }
                }
            }
        }

        println!("----------------");
        println!("{:?}", asm_label);
        println!("{:?}", asm_opcode);
        println!("{:?}", asm_operand);
        println!("----------------");
        ASMLine { label: asm_label, opcode: asm_opcode, operand: asm_operand, size }
    }
    pub fn parse_label(label: impl Into<String>) -> Label {
        let label = label.into();
        if label.is_empty() {
            Label::None
        } else {
            Label::Val(label)
        }
    }
    pub fn parse_opcode(opcode: impl Into<String>) -> Result<Opcode, ()> {
        let opcode = opcode.into();
        if opcode.is_empty() {
            Err(())
        } else if Self::is_directive(&opcode) {
            Ok(Opcode::Directive(opcode))
        } else {
            let first_char = get_nth_char(&opcode, 1).unwrap();
            if first_char == '+' {
                let rest = substring(&opcode, 1, opcode.len()).unwrap();
                Ok(Opcode::Extended(rest))
            } else {
                Ok(Opcode::Simple(opcode))
            }
        }
    }
    pub fn parse_operand(operand: impl Into<String>) -> Result<Operand, ()> {
        let operand = operand.into();
        if operand.is_empty() {
            Ok(Operand::None)
        } else if let Ok(symbol) = Self::parse_symbol(&operand) {
            Ok(Operand::Symbol(symbol))
        } else if let Ok(constant) = Self::parse_constant(&operand) {
            Ok(Operand::Constant(constant))
        } else if let Ok(literal) = Self::parse_literal(&operand) {
            Ok(Operand::Literal(literal))
        } else if let Ok(address) = Self::parse_address(&operand) {
            Ok(Operand::Address(address))
        } else if let Ok(expression) = Self::parse_expression(&operand) {
            Ok(Operand::Expression(expression))
        } else {
            Err(())
        }
    }
    pub fn parse_constant(constant: impl Into<String>) -> Result<StringOrNum, ()> {
        let constant = constant.into();
        if constant.len() < 4 {
            return Err(());
        }
        let first_char = get_nth_char(&constant, 1).unwrap();
        let second_char = get_nth_char(&constant, 2).unwrap();
        let last_char = get_nth_char(&constant, constant.len()).unwrap();
        if (first_char != 'X' && first_char != 'C') || second_char != '\'' || last_char != '\'' {
            return Err(());
        }
        let const_part = substring(&constant, 2, constant.len() - 1).unwrap();
        Ok(match first_char {
            'C' => StringOrNum::Str(const_part),
            'X' => StringOrNum::Num(hex_string_to_i32(const_part)),
            _ => panic!("How?")
        })
    }
    pub fn parse_literal(literal: impl Into<String>) -> Result<StringOrNum, ()> {
        let literal = literal.into();
        if literal.len() < 5 {
            return Err(());
        }
        let first_char = get_nth_char(&literal, 1).unwrap();
        if first_char != '=' {
            return Err(());
        }
        Self::parse_constant(substring(&literal, 1, literal.len()).unwrap())
    }

    pub fn parse_symbol(operand: impl Into<String>) -> Result<Symbol, ()> {
        let operand = operand.into();
        if operand == "*" {
            Ok(Symbol::PC)
        } else {
            Err(())
        }
    }

    pub fn parse_register_address(
        register_address: impl Into<String>,
    ) -> Result<RegisterAddress, ()> {
        let register_address = register_address.into();
        let parts: Vec<&str> = register_address.split(",").collect();
        if parts.len() > 2 {
            return Err(());
        }
        if parts.len() == 2 {
            return if parts
                .iter()
                .filter(|part| !matches!(**part, "A" | "X" | "B" | "S" | "T" | "L"))
                .count()
                > 0
            {
                Err(())
            } else {
                Ok(RegisterAddress::Double(
                    parts[0].to_string(),
                    parts[1].to_string(),
                ))
            };
        }
        if !parts[0].is_empty() && matches!(parts[0], "A" | "X" | "B" | "S" | "T" | "L") {
            return Ok(RegisterAddress::Single(parts[0].to_string()));
        }
        Err(())
    }
    pub fn parse_indexed_address(indexed_address: impl Into<String>) -> Result<Address, ()> {
        let indexed_address = indexed_address.into();
        let parts: Vec<&str> = indexed_address.split(",").collect();
        if parts.len() != 2 {
            return Err(());
        }
        if parts[1] != "X" {
            return Err(());
        }

        return Ok(Address::Indexed(StringOrNum::new(parts[0])));
        Err(())
    }
    pub fn parse_immediate_address(immediate_address: impl Into<String>) -> Result<Address, ()> {
        let immediate_address = immediate_address.into();
        if let Ok(hash) = get_nth_char(&immediate_address, 1) {
            if hash != '#' {
                return Err(());
            }
            return Ok(Address::Immediate(StringOrNum::new(immediate_address.chars().skip(1).collect::<String>())));
        }
        Err(())
    }
    pub fn parse_indirect_address(immediate_address: impl Into<String>) -> Result<Address, ()> {
        let immediate_address = immediate_address.into();
        if let Ok(at) = get_nth_char(&immediate_address, 1) {
            if at != '@' {
                return Err(());
            }

            return Ok(Address::Indirect(StringOrNum::new(immediate_address.chars().skip(1).collect::<String>())));
        }
        Err(())
    }
    pub fn parse_simple_address(simple_address: impl Into<String>) -> Result<Address, ()> {
        Ok(Address::Simple(StringOrNum::new(simple_address.into())))
    }
    pub fn parse_address(address: impl Into<String>) -> Result<Address, ()> {
        let address = address.into();

        if address.chars().any(Self::is_operator) {
            Err(())
        } else if let Ok(register) = Self::parse_register_address(&address) {
            Ok(Address::Register(register))
        } else if let Ok(immediate) = Self::parse_immediate_address(&address) {
            Ok(immediate)
        } else if let Ok(indexed) = Self::parse_indexed_address(&address) {
            Ok(indexed)
        } else if let Ok(indirect) = Self::parse_indirect_address(&address) {
            Ok(indirect)
        } else if let Ok(simple) = Self::parse_simple_address(&address) {
            Ok(simple)
        } else {
            Err(())
        }
    }
    pub fn parse_expression(expression: impl Into<String>) -> Result<Vec<ParsedExpressionPart>, ()> {
        let expression = expression.into();
        let mut expression_chars = expression.chars();
        let expression_chars = expression_chars.by_ref();
        let mut ret_vec = Vec::new();
        let mut operator_stack: Stack<Operator> = Stack::new();
        while let Some(start) = expression_chars.next() {
            let (operand, optional_operator) =
                Self::parse_expression_operand(start, expression_chars);
            if let Some(operand) = operand {
                ret_vec.push(ParsedExpressionPart::Operand(operand));
            }
            match optional_operator {
                Some(operator_char) => {
                    let operator_string = String::from(operator_char);
                    let operator = Operator::Val(operator_string);
                    if operator_char == '(' {
                        print!("WOW");
                        operator_stack.push(operator);
                        continue;
                    }
                    if operator_char == ')' {
                        // println!("{:?}", operator_stack);
                        loop {
                            let tos_operator = operator_stack.pop();
                            match tos_operator {
                                None => {
                                    return Err(());
                                }
                                Some(stack_operator) => {
                                    if stack_operator.unwrap().as_str() == "(" {
                                        break;
                                    } else {
                                        ret_vec.push(ParsedExpressionPart::Operator(stack_operator))
                                    }
                                }
                            };
                        }
                        continue;
                    }
                    loop {
                        if operator_stack.is_empty() {
                            break;
                        }
                        let tos_operator = operator_stack.peek().unwrap();
                        if tos_operator >= operator && tos_operator.unwrap().as_str() != "(" {
                            let tos_operator = operator_stack.pop().unwrap();
                            ret_vec.push(ParsedExpressionPart::Operator(tos_operator));
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(operator)
                }
                None => {
                    continue;
                }
            }
        }
        while let Some(operator) = operator_stack.pop() {
            ret_vec.push(ParsedExpressionPart::Operator(operator));
        }
        Ok(ret_vec)
    }

    pub fn parse_expression_operand(
        start: char,
        expression: &mut Chars,
    ) -> (Option<StringOrNum>, Option<char>) {
        let mut operand_vec = vec![];
        let mut operand = None;
        if Self::is_operator(start) {
            return (None, Some(start));
        } else {
            operand_vec.push(start);
        }
        for c in expression {
            if Self::is_operator(c) {
                operand = Some(c);
                break;
            }
            operand_vec.push(c);
        }
        (
            Some(StringOrNum::new(operand_vec.iter().collect::<String>())),
            operand,
        )
    }

    pub fn is_operator(character: char) -> bool {
        matches!(character, '-' | '+' | '/' | '%' | '(' | ')' | '^')
    }

    pub fn is_directive(directive: impl Into<String>) -> bool
    {
        matches!(directive.into().as_str(), "RESB"|"RESW"|"WORD"|"BYTE"|"EQU"|"LTORG"|"USE"|"CSECT"|"ORG"|"START"|"END")
    }
    pub fn is_opcode(opcode: impl Into<String>, global_map: &GlobalMap) -> bool {
        let opcode = opcode.into();
        let opcode = opcode.trim_start_matches("+");
        global_map.opcode_map.keys().any(|mnemonic| {
            mnemonic.as_str() == opcode
        })
    }
}

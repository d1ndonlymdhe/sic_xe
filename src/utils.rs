pub fn string_to_usize(string: impl Into<String>) -> usize {
    let string = string.into();
    string.parse::<usize>().unwrap_or_else(|_| panic!("couldn't parse as usize, string {}", string))
}

pub fn is_valid_decimal_string(bin_string: impl Into<String>) -> bool {
    bin_string.into().parse::<i32>().is_ok()
}

pub fn hex_string_to_i32(hex_string: String) -> i32 {
    i32::from_str_radix(&hex_string, 16).unwrap_or_else(|_| panic!("Invalid hex string {}", hex_string))
}

pub fn i32_to_hex_string(val: i32, len: usize) -> String {
    let mut t = format!("{:X}", val);
    while t.len() < len {
        t = format!("0{}", t);
    };
    t
}

pub fn i32_to_bin_string(val: i32, len: usize) -> String {
    let mut t = format!("{:b}", val);
    while t.len() < len {
        t = format!("0{}", t);
    }
    t
}

pub fn bin_string_to_i32(bin_string: String) -> i32 {
    i32::from_str_radix(&bin_string, 2).unwrap_or_else(|_| panic!("Invalid binary string {}", bin_string))
}


pub fn get_nth_char(word: impl Into<String>, n: usize) -> Result<char, String> {
    let as_vec = word.into().chars().collect::<Vec<char>>();
    if as_vec.is_empty() {
        Err(String::from("Empty string"))
    } else if as_vec.len() < n {
        Err(String::from("Out of bounds"))
    } else {
        Ok(as_vec[n - 1])
    }
}
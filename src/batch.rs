use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn batch_mode(filename: &str) ->Vec<String>{
    let file = File::open(filename).expect("Unable to open file");
    let lines = BufReader::new(file).lines();
    let lines =  lines.into_iter().map(|line| line.expect("Couldn't read line")).collect();
    lines
}
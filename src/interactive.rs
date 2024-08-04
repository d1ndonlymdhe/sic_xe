pub fn interactive_mode()->Vec<String> {
    print_help();
    let mut lines = Vec::new();
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();
        if input == "exit" {
            break;
        }
        if input == "undo" {
            lines.pop();
            continue;
        }
        if input == "help" {
            print_help();
            continue;
        }
        lines.push(input.to_string());
    }
    lines 
}
fn print_help() {
    println!("INTERACTIVE MODE:");
    println!("Enter 'exit' to exit");
    println!("Enter 'undo' to undo the last line");
    println!("Enter 'help' to see this message again");
}
use std::io;

pub fn safe_number_input(question: &str) -> i32 {
    loop {
        let mut input: String = String::new();
        println!("{}", question);
        io::stdin().read_line(&mut input);
        let input: &str = input.trim();

        if let Ok(zahl) = input.parse::<i32>() {
            return zahl;
        }
    }
}

pub fn safe_string_input(question: &str) -> String {
    loop {
        let mut input: String = String::new();
        println!("{}", question);
        io::stdin().read_line(&mut input);
        let input: String = input.trim().to_string();

        return input;
    }
}

pub fn safe_option_input(question: &str, options: Option<&[&str]>) -> String {
    loop {
        let mut input: String = String::new();
        println!("{}", question);
        io::stdin().read_line(&mut input);
        let input: &str = input.trim();

        if let Some(options) = options {
            if options.contains(&input) {
                return input.to_string();
            }
        }
    }
}
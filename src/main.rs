use toml_edit::{DocumentMut, Item, Table, value};
use std::io;
use std::fs;

fn main() {

    //UI
    //Toml lesen und schreiben können
    //Datenstruktur mit Übungen die dann zufällig gewählt werden

    let PATH_TO_CONFIG: &str = "config.toml";
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    doc["floor"] = value(600);
    doc["ceil"] = value(1200);
    doc["doubles"] = value(true);
    doc["last_exercise"] = value("Pullup");
    let mut exercises_table: Table = Table::new();

    let mut pullup_table: Table = Table::new();
    let mut pullup_profiles: Table = Table::new();
    pullup_profiles["normal"] = value(true);
    pullup_table["profiles"] = Item::Table(pullup_profiles);
    pullup_table["type"] = value("reps");
    exercises_table["Pullup"] = Item::Table(pullup_table);

    let mut pushup_table: Table = Table::new();
    let mut pushup_profiles: Table = Table::new();
    pushup_profiles["normal"] = value(true);
    pushup_table["profiles"] = Item::Table(pushup_profiles);
    pushup_table["type"] = value("reps");
    exercises_table["Pushup"] = Item::Table(pushup_table);

    let mut situp_table: Table = Table::new();
    let mut situp_profiles: Table = Table::new();
    situp_profiles["normal"] = value(true);
    situp_table["profiles"] = Item::Table(situp_profiles);
    situp_table["type"] = value("reps");
    exercises_table["Situp"] = Item::Table(situp_table);
    doc["exercises"] = Item::Table(exercises_table);

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
    println!("Erfolgreich geschrieben!");


    let mut exercises: Vec<String> = Vec::new();
    exercises.push(String::from("Pullup"));
    exercises.push(String::from("Pushup"));
    exercises.push(String::from("Situp"));
    println!("{}", pick_random_exercise(exercises))
}

fn change_interval(path_to_config: String) {
    let toml_str: String = fs::read_to_string(path_to_config).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let floor:String  = safe_string_input("From: ");
    let ceil:String  = safe_string_input("To: ");
    doc["floor"] = value(floor);
    doc["ceil"] = value(ceil);

}

fn safe_number_input(question: &str) -> i32 {
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

fn safe_string_input(question: &str) -> String {
    loop {
        let mut input: String = String::new();
        println!("{}", question);
        io::stdin().read_line(&mut input);
        let input: String = input.trim().to_string();

        return input;
    }
}

fn safe_option_input(question: &str, options: Option<&[&str]>) -> String {
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

fn pick_random_exercise(exercises: Vec<String>) -> String {
    let index: usize = rand::random_range(0..exercises.len());
    exercises[index].clone()
}
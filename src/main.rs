use toml_edit::{DocumentMut, Item, Table, value};
use std::fs;
mod funcs;

fn change_exercise_status(path: &str, exercise: &str) {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut status: bool = true;

    if let Some(exercises_table) = doc.get("exercises") {
        if let Some(exercise_table) = exercises_table.get(exercise) {
            if let Some(profiles_table) = exercise_table.get("profiles") {
                if let Some(profile_value) = profiles_table.get("normal") {
                    status = profile_value.as_bool().unwrap();
                }
            }
        }
    }

    doc["exercises"][exercise]["profiles"]["normal"] = value(!status);

    fs::write(path, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn add_exercise(path: &str, exercise: &str) {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");
    
    doc[exercise]["reps"] = value(0);
    doc[exercise]["amount"] = value(0);
    doc[exercise]["max"] = value(0);
    
    fs::write(path, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn add_reps(path: &str, exercise: &str, reps: i32) {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    if let Some(exercise_value) = doc.get(exercise) {
        let reps: i64 = reps as i64;
        let mut all_reps: i64 = 0;
        let mut amount: i64 = 0;
        let mut max: i64 = 0;
        
        if let Some(reps_value) = exercise_value.get("reps") {
            all_reps = reps_value.as_integer().unwrap();
        }
        if let Some(amount_value) = exercise_value.get("amount") {
            amount = amount_value.as_integer().unwrap();
        }
        if let Some(max_value) = exercise_value.get("max") {
            max = max_value.as_integer().unwrap();
        }
        
        if reps > 0 {
            if reps > max {
                max = reps;
            }
            amount += 1;
            all_reps += reps;
            
            doc[exercise]["max"] = value(max);
            doc[exercise]["amount"] = value(amount);
            doc[exercise]["reps"] = value(all_reps);
            
            fs::write(path, doc.to_string()).expect("Fehler beim Schreiben!");
        }
    }
}

fn change_interval(path: &str) {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let floor:String  = funcs::safe_string_input("From: ");
    let ceil:String  = funcs::safe_string_input("To: ");
    doc["floor"] = value(floor);
    doc["ceil"] = value(ceil);
    
    fs::write(path, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn pick_random_exercise(exercises: Vec<String>) -> String {
    let index: usize = rand::random_range(0..exercises.len());
    exercises[index].clone()
}

fn get_exercises(path: &str) -> Vec<String> {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut exercises: Vec<String> = Vec::new();

    if let Some(exercises_table) = doc.get("exercises").and_then(|t| t.as_table()) {
        for (exercise_name, exercise_data) in exercises_table.iter() {
            if let Some(status) = exercise_data.get("profiles").unwrap().get("normal") {
                if status.as_bool().unwrap() == true {
                    exercises.push(String::from(exercise_name));
                }
            }
        }
    }

    exercises
}

fn main() {

    //profil ändern
    //profil hinzufügen/löschen
    //UI

    const PATH_TO_CONFIG: &str = "config.toml";
    const PATH_TO_DATA: &str = "exercise_data.toml";
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


    change_interval(PATH_TO_CONFIG);
}

use toml_edit::{DocumentMut, Item, Table, value};
use std::fs;
use std::time::Instant;
use std::process::Command;
use slint::SharedString;
mod funcs;

slint::slint! {
    export { MainWindow } from "src/ui.slint";
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
    }
}

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

fn change_interval(path: &str, floor: SharedString, ceil: SharedString) -> i32 {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut problem_number: i32 = 0;

    let floor_value: &str  = floor.as_str();
    let ceil_value: &str   = ceil.as_str();
    if let Ok(floor) = floor_value.parse::<i64>() {
        doc["floor"] = value(floor);
    } else {problem_number += 1}
    if let Ok(ceil) = ceil_value.parse::<i64>() {
        doc["ceil"] = value(ceil);
    } else {problem_number += 2}

    if problem_number == 0 {
        fs::write(path, doc.to_string()).expect("Fehler beim Schreiben!");
    }
    problem_number    
    //TODO nachgucken ob wirklich zahlen, wenn nicht dann slint sagen er hat fehler
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

fn start_timer(path: &str) {
    let toml_str: String = fs::read_to_string(path).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut floor: i64 = 0;
    let mut ceil: i64 = 0;

    if let Some(floor_value) = doc.get("floor") {
        floor = floor_value.as_integer().unwrap();
    }
    if let Some(ceil_value) = doc.get("ceil") {
        ceil = ceil_value.as_integer().unwrap();
    }

    let time: f64 = rand::random_range(floor..ceil) as f64;
    let chosen_exercise: String = pick_random_exercise(get_exercises(path));
    let start_time = Instant::now();
    let mut duration = start_time.elapsed().as_secs_f64();

    loop {
        duration = start_time.elapsed().as_secs_f64();
        clear_screen();
        println!("{}", duration.round());
        if duration >= time {break}
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("Übung: {}", chosen_exercise);
}

fn main() {
    //TODO(UI)
    //TODO(profiles)

    const PATH_TO_CONFIG: &str = "config.toml";
    const PATH_TO_DATA: &str = "exercise_data.toml";

    let ui = MainWindow::new().unwrap();
    let ui_handle = ui.as_weak();
    ui.on_start_pressed(move |floor: slint::SharedString, ceil: slint::SharedString| {
        println!("Floor: {}\n Ceil: {}", floor, ceil);

        let problem_number: i32 = change_interval(PATH_TO_CONFIG, floor, ceil);
        println!("Problems: {}", problem_number);
        match problem_number {
            0 => {},
            1 => if let Some(handle) = ui_handle.upgrade() {
                    handle.set_floor_input_invalid(true);
                    handle.invoke_text_input_flash();
                },
            2 => if let Some(handle) = ui_handle.upgrade() {
                    handle.set_ceil_input_invalid(true);
                    handle.invoke_text_input_flash();
                },
            3 => if let Some(handle) = ui_handle.upgrade() {
                    handle.set_floor_input_invalid(true);
                    handle.set_ceil_input_invalid(true);
                    handle.invoke_text_input_flash();
                },
            _ => {},
        }
    });
    ui.run().unwrap();
    
}

//TODO direkt am anfang die ersten werte für floor und ceil aus dem config übergeben
//TODO knopf farbe und aufschrift ändern
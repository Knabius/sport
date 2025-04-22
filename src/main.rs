use toml_edit::{DocumentMut, Item, Table, value};
use std::fs;
use std::time::Instant;
use std::process::Command;
use slint::{SharedString, Timer, TimerMode};
use std::rc::Rc;
use std::cell::RefCell;
mod funcs;

const PATH_TO_CONFIG: &str = "config.toml";
const PATH_TO_DATA: &str = "exercise_data.toml";

slint::slint! {
    export { MainWindow } from "src/ui.slint";
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "cls"]).status().unwrap();
    }
}

fn change_exercise_status(exercise: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
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

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn add_exercise(exercise: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_DATA).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");
    //TODO auch die config verändern
    
    doc[exercise]["reps"] = value(0);
    doc[exercise]["amount"] = value(0);
    doc[exercise]["max"] = value(0);
    
    fs::write(PATH_TO_DATA, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn add_reps(exercise: &str, reps: i32) {
    let toml_str: String = fs::read_to_string(PATH_TO_DATA).expect("Fehler beim lesen!");
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
            
            fs::write(PATH_TO_DATA, doc.to_string()).expect("Fehler beim Schreiben!");
        }
    }
}

fn change_interval(floor: SharedString, ceil: SharedString) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut problem_number: i32 = 0;

    let floor_value: &str  = floor.as_str();
    let ceil_value: &str   = ceil.as_str();
    if let Ok(floor) = floor_value.parse::<i64>() {
        doc["floor"] = value(floor);
    }
    if let Ok(ceil) = ceil_value.parse::<i64>() {
        doc["ceil"] = value(ceil);
    }

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn pick_random_exercise(exercises: Vec<String>) -> String {
    let index: usize = rand::random_range(0..exercises.len());
    exercises[index].clone()
}

fn get_exercises() -> Vec<String> {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
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

fn start_timer() {
    let interval: (i64, i64) = get_interval();

    let time: f64 = rand::random_range(interval.0..interval.1) as f64;
    let chosen_exercise: String = pick_random_exercise(get_exercises());
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

fn get_interval() -> (i64, i64) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    if let Some(floor) = doc.get("floor") {
        if let Some(ceil) = doc.get("ceil") {
            let tup: (i64, i64) = (floor.as_integer().unwrap(), ceil.as_integer().unwrap());
            return tup;
        }
    }
    (10,20)
}

fn set_last_exercise(exercise: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    doc["last_exercise"] = value(exercise);

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn main() {
    let ui = MainWindow::new().unwrap();
    let ui_handle = ui.as_weak();
    if let Some(handle) = ui_handle.upgrade() {
        let interval: (i64, i64) = get_interval();
        handle.set_initial_floor_value(interval.0 as i32);
        handle.set_initial_ceil_value(interval.1 as i32);
    }

    // true => time-loop inaktiv
    // false => time-loop aktiv
    let mut start_button_status: bool = true;

    let is_running = Rc::new(RefCell::new(true));
    let is_running_clone = is_running.clone();
    let start_button_status = Rc::new(RefCell::new(true));
    let start_button_status_clone = start_button_status.clone();

    let ui_handle_save_reps = ui_handle.clone();

    let chosen_exercise:Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let chosen_exercise_rep_clone = chosen_exercise.clone();


    ui.on_start_pressed(move |floor: slint::SharedString, ceil: slint::SharedString| {

        if *start_button_status_clone.borrow() == true {

            change_interval(floor, ceil);

            if let Some(handle) = ui_handle.upgrade() {
                *start_button_status_clone.borrow_mut() = false;
                handle.set_start_button_status(false);

                let interval: (i64, i64) = get_interval();
                let time: f64 = rand::random_range(interval.0..interval.1) as f64;
                *chosen_exercise.borrow_mut() = pick_random_exercise(get_exercises());
                let chosen_exercise_clone = chosen_exercise.clone();
                let start_time = Instant::now();
                
                let timer = Rc::new(Timer::default());
                let timer_clone = timer.clone();
                *is_running_clone.borrow_mut() = true;
                let is_running_deep = is_running_clone.clone();
                let start_button_status_deep = start_button_status_clone.clone();
                let ui_handle_deep = ui_handle.clone();

                timer.start(TimerMode::Repeated, std::time::Duration::from_millis(500), move || {
                    let duration: f64 = start_time.elapsed().as_secs_f64();

                    if duration >= time {
                        timer_clone.stop();
                        println!("{}", *chosen_exercise_clone.borrow());
                        *start_button_status_deep.borrow_mut() = true;
                        if let Some(handle) = ui_handle_deep.upgrade() {
                            handle.set_start_button_status(true);
                            handle.set_chosen_exercise(chosen_exercise_clone.borrow().as_str().into());
                            handle.set_current_view(CurrentView::RepInput);
                        }
                        set_last_exercise(chosen_exercise_clone.borrow().as_str());
                    } else if !*is_running_deep.borrow() {
                        timer_clone.stop();
                        *start_button_status_deep.borrow_mut() = true;
                        if let Some(handle) = ui_handle_deep.upgrade() {
                            handle.set_start_button_status(true);
                        }
                    } else {
                        clear_screen();
                        println!("{}", duration.round());
                        if let Some(handle) = ui_handle_deep.upgrade() {
                            handle.set_passed_time(duration as i32);
                        }
                    }
                });
            }
        } else if *start_button_status_clone.borrow() == false {
            *is_running_clone.borrow_mut() = false;
            if let Some(handle) = ui_handle.upgrade() {
                *start_button_status_clone.borrow_mut() = true;
                handle.set_start_button_status(true);
            }
        }
    });

    ui.on_save_reps(move |reps: slint::SharedString| {
        if let Some(handle) = ui_handle_save_reps.upgrade() {
            handle.set_current_view(CurrentView::BasicButton);
        }
        add_reps(chosen_exercise_rep_clone.borrow().as_str(), reps.parse::<i32>().unwrap());
    });
    ui.run().unwrap();
    
}

//TODO profiles
//TODO Menü mit knopf für +Übung, -Übung, welche übungen an sind, andere optionen

//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use toml_edit::{value, DocumentMut, Item, Table, TableLike, Entry};
use std::fs;
use std::thread::current;
use std::time::Instant;
use slint::{SharedString, Timer, TimerMode, ToSharedString, ModelRc, VecModel, Weak};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use rodio::{Decoder, OutputStream, Sink, Source};
use chrono::Local;

const PATH_TO_CONFIG: &str = "config.toml";
const PATH_TO_DATA: &str =   "exercise_data.toml";
const PATH_TO_CHRONO: &str = "chronological_data.txt";
const VOLUME: f32 = 0.3;

slint::slint! {export { MainWindow } from "src/ui.slint";}

fn add_exercise(exercise: &str, exercise_type: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_DATA).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");
    let mut table: Table = Table::new();

    table["reps"] = value(0);
    table["amount"] = value(0);
    table["max"] = value(0);

    doc[exercise] = Item::Table(table);
    
    fs::write(PATH_TO_DATA, doc.to_string()).expect("Fehler beim Schreiben!");

    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");
    
    doc["exercises"][exercise]["type"] = value(exercise_type);
    let current_profile: String = doc["current_profile"].as_str().unwrap().to_string();
    doc["exercises"][exercise]["profiles"][current_profile] = value(true);
    
    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
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

            let mut chrono: File = OpenOptions::new().write(true).append(true).create(true).open(PATH_TO_CHRONO).unwrap();
            writeln!(chrono, "[{}]:{}:{}", Local::now().format("%Y-%m-%d").to_string(),exercise, reps);
        }
    }
}

fn change_interval(floor: SharedString, ceil: SharedString) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

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
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut exercises: Vec<String> = exercises;
    let doubles: bool = doc["doubles"].as_bool().unwrap();
    let last_exercise: String = doc["last_exercise"].as_str().unwrap().to_string();

    if !doubles && exercises.len() > 1 {
        if let Some(index) = exercises.iter().position(|x| *x == last_exercise) {
            exercises.remove(index);
        }
        let index: usize = rand::random_range(0..exercises.len());
        exercises[index].clone()
    } else {
        let index: usize = rand::random_range(0..exercises.len());
        exercises[index].clone()
    }
}

fn get_exercises() -> Vec<String> {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut exercises: Vec<String> = Vec::new();
    let current_profile: &str = &doc["current_profile"].as_str().unwrap();
    
    
    if let Some(exercises_table) = doc.get("exercises").and_then(|t| t.as_table()) {
        for (exercise_name, exercise_data) in exercises_table.iter() {
            if let Some(status) = exercise_data.get("profiles").unwrap().get(current_profile) {
                if status.as_bool().unwrap() == true {
                    exercises.push(String::from(exercise_name));
                }
            }
        }
    }

    exercises
}

fn get_interval() -> (i64, i64) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

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

fn get_general_settings() -> bool {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let setting_doubles: bool = doc["doubles"].as_bool().unwrap();
    setting_doubles
}

fn set_general_settings(setting_doubles: bool) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    doc["doubles"] = value(setting_doubles);

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn get_exercise_settings() -> Vec<Exercise> {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut exercises: Vec<String>  = Vec::new();
    let mut activations: Vec<bool> = Vec::new();
    let current_profile: &str = doc["current_profile"].as_str().unwrap();

    if let Some(exercises_table) = doc.get("exercises").and_then(|t| t.as_table()) {
        for (exercise_name, exercise_data) in exercises_table.iter() {
            if let Some(status) = exercise_data.get("profiles").unwrap().get(current_profile).unwrap().as_bool() {
                exercises.push(String::from(exercise_name));
                activations.push(status);
            }
        }
    }

    let items: Vec<Exercise> = exercises.into_iter().zip(activations.into_iter()).map(|(exercise, activation)| Exercise {
        name: exercise.to_shared_string(),
        activation_status: activation,}).collect();

    items
}

fn set_exercise_activation(exercise_name: SharedString) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");
    let current_profile: String = doc["current_profile"].as_str().unwrap().to_string();

    let val: bool = doc["exercises"][exercise_name.as_str()]["profiles"][&current_profile].as_bool().unwrap();
    doc["exercises"][exercise_name.as_str()]["profiles"][&current_profile] = value(!val);

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn remove_exercise(exercise: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_DATA).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    doc.remove(exercise);

    fs::write(PATH_TO_DATA, doc.to_string()).expect("Fehler beim Schreiben!");

    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    if let Some(exercises) = doc.get_mut("exercises").and_then(|item| item.as_table_like_mut()) {
        exercises.remove(exercise);
    }

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn change_profile(profile: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    doc["current_profile"] = value(profile);

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn add_profile(profile: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    if let Some(exercises) = doc.get_mut("exercises").and_then(|e| e.as_table_mut()) {
        for (_name, settings) in exercises.iter_mut() {
            if let Some(profiles_table) = settings.get_mut("profiles").and_then(|p| p.as_table_mut()) {
                profiles_table.entry(profile).or_insert(value(false));
            }
        }
    }

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn remove_profile(profile: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    if let Some(exercises) = doc.get_mut("exercises").and_then(|e| e.as_table_mut()) {
        for (_name, settings) in exercises.iter_mut() {
            if let Some(profiles_table) = settings.get_mut("profiles").and_then(|p| p.as_table_mut()) {
                match profiles_table.entry(profile) {
                    Entry::Occupied(key) => {key.remove();}
                    Entry::Vacant(key) => {}
                }
            }
        }
    }

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn get_profile_names() -> Option<Vec<SharedString>> {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).ok()?;
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().ok()?;

    let exercises: &Table = doc.get("exercises")?.as_table()?;
    let profile_inline: &toml_edit::InlineTable = exercises.get_values().iter().next()?.1.as_inline_table()?.get("profiles")?.as_inline_table()?;
    let profile_names = profile_inline.iter().map(|x| x.0.into()).collect::<Vec<SharedString>>();

    Some(profile_names)
}

fn main() {
    //Slint
    let ui = MainWindow::new().unwrap();
    let ui_handle: Weak<MainWindow> = ui.as_weak();

    //Audio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink: Rc<RefCell<Sink>> = Rc::new(RefCell::new(Sink::try_new(&stream_handle).unwrap()));

    //Initial Setup
    if let Some(handle) = ui_handle.upgrade() {
        let interval: (i64, i64) = get_interval();
        handle.set_initial_floor_value(interval.0 as i32);
        handle.set_initial_ceil_value(interval.1 as i32);
        handle.set_setting_general_doubles(get_general_settings());

        let exercise_structs: Vec<Exercise> = get_exercise_settings();
        let exercise_structs_clone: Vec<Exercise> = exercise_structs.clone();
        handle.set_exercises(ModelRc::new(VecModel::from(exercise_structs)));
        let exercise_names: Vec<SharedString> = exercise_structs_clone.iter().map(|item| item.name.clone()).collect();
        handle.set_exercise_names(ModelRc::new(VecModel::from(exercise_names)));
        handle.set_profile_names(ModelRc::new(VecModel::from(get_profile_names().unwrap())));
    }

    // true => time-loop inaktiv
    // false => time-loop aktiv

    let is_running: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));
    let is_running_clone: Rc<RefCell<bool>> = is_running.clone();
    let start_button_status: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));
    let start_button_status_clone: Rc<RefCell<bool>> = start_button_status.clone();

    let ui_handle_save_reps: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_add_exercise: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_remove_exercise: Weak<MainWindow> = ui_handle.clone();


    let chosen_exercise:Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let chosen_exercise_rep_clone: Rc<RefCell<String>> = chosen_exercise.clone();

    let sink_clone: Rc<RefCell<Sink>> = sink.clone();

    ui.on_start_pressed(move |floor: SharedString, ceil: SharedString| {

        if !get_exercises().is_empty() {
            if *start_button_status_clone.borrow() == true {

                change_interval(floor, ceil);

                if let Some(handle) = ui_handle.upgrade() {
                    *start_button_status_clone.borrow_mut() = false;
                    handle.set_start_button_status(false);

                    let interval: (i64, i64) = get_interval();
                    let time: f64 = rand::random_range(interval.0..interval.1) as f64;
                    *chosen_exercise.borrow_mut() = pick_random_exercise(get_exercises());
                    let chosen_exercise_clone: Rc<RefCell<String>> = chosen_exercise.clone();
                    let start_time: Instant = Instant::now();
                    
                    let timer: Rc<Timer> = Rc::new(Timer::default());
                    let timer_clone: Rc<Timer> = timer.clone();
                    *is_running_clone.borrow_mut() = true;
                    let is_running_deep: Rc<RefCell<bool>> = is_running_clone.clone();
                    let start_button_status_deep: Rc<RefCell<bool>> = start_button_status_clone.clone();
                    let ui_handle_deep: Weak<MainWindow> = ui_handle.clone();

                    let sink: Rc<RefCell<Sink>> = sink.clone();

                    timer.start(TimerMode::Repeated, std::time::Duration::from_millis(500), move || {
                        let duration: f64 = start_time.elapsed().as_secs_f64();

                        if duration >= time {
                            set_last_exercise(chosen_exercise_clone.borrow().as_str());
                            timer_clone.stop();
                            *start_button_status_deep.borrow_mut() = true;
                            if let Some(handle) = ui_handle_deep.upgrade() {
                                handle.set_start_button_status(true);
                                handle.set_chosen_exercise(chosen_exercise_clone.borrow().as_str().into());
                                handle.set_current_view(CurrentView::RepInput);
                            }

                            //Audio
                            let boing: rodio::source::Amplify<Decoder<BufReader<File>>> = Decoder::new(BufReader::new(File::open("resources/Sound.mp3").unwrap())).unwrap().amplify(VOLUME);
                            sink.borrow_mut().append(boing);
                            sink.borrow_mut().play()

                        } else if !*is_running_deep.borrow() {
                            timer_clone.stop();
                            *start_button_status_deep.borrow_mut() = true;
                            if let Some(handle) = ui_handle_deep.upgrade() {
                                handle.set_start_button_status(true);
                            }

                        } else {
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
        }
    });

    ui.on_save_reps(move |reps: SharedString| {
        if let Some(handle) = ui_handle_save_reps.upgrade() {
            handle.set_current_view(CurrentView::BasicButton);
        }
        add_reps(chosen_exercise_rep_clone.borrow().as_str(), reps.parse::<i32>().unwrap());
        sink_clone.borrow_mut().clear();
    });

    ui.on_changed_general_settings(move |setting_doubles: bool| {
        set_general_settings(setting_doubles);
    });

    ui.on_changed_activation_settings(move |name: SharedString| {
        set_exercise_activation(name);
    });

    ui.on_add_exercise(move |name:SharedString, exercise_type:SharedString| {
        add_exercise(name.as_str(), &exercise_type.as_str());
        
        if let Some(handle) = ui_handle_add_exercise.upgrade() {
            let exercise_structs: Vec<Exercise> = get_exercise_settings();
            let exercise_structs_clone = exercise_structs.clone();
            handle.set_exercises(ModelRc::new(VecModel::from(exercise_structs)));
            let exercise_names: Vec<SharedString> = exercise_structs_clone.iter().map(|item| item.name.clone()).collect();
            handle.set_exercise_names(ModelRc::new(VecModel::from(exercise_names)));
        }
    });

    ui.on_remove_exercise(move |name: SharedString| {
        remove_exercise(name.as_str());

        if let Some(handle) = ui_handle_remove_exercise.upgrade() {
            handle.set_exercises(ModelRc::new(VecModel::from(get_exercise_settings())));
        }
    });

    ui.on_add_reps(move |name: SharedString, reps: SharedString| {
        add_reps(name.as_str(), reps.as_str().parse::<i32>().unwrap());
    });

    ui.run().unwrap();
}

//TODO profiles
//TODO Main Button mit Enter auslösen
//TODO daten einsehen können
//TODO daten in diagrammen sehen können
//TODO prioritize


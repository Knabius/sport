//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_locc)]
#![allow(clippy::too_many_lines)]

use toml_edit::{value, DocumentMut, Entry, InlineEntry, Item, Table, TableLike};
use std::fs;
use std::hash::Hash;
use std::os::windows::io::AsHandle;
use std::time::{Instant};
use slint::{SharedString, Timer, TimerMode, ToSharedString, ModelRc, VecModel, Weak};
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use rodio::{Decoder, OutputStream, Sink, Source};
use chrono::{Datelike, Local, NaiveDate, Duration};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use rand::*;

//REMINDER src/...
const PATH_TO_CONFIG: &str = "src/resources/config.toml";
const PATH_TO_DATA: &str =   "src/resources/exercise_data.toml";
const PATH_TO_CHRONO: &str = "src/resources/chronological_data.txt";
const PATH_TO_SOUNDS: &str = "src/resources/sounds/";
static mut VOLUME: f32 = 0.3;
static VISUALISATION_DATA: Lazy<Mutex<(HashMap<String,(i32,i32,i32)>, HashMap<String, Vec<(NaiveDate, i32)>>)>> = Lazy::new(|| Mutex::new((HashMap::new(), HashMap::new())));

slint::slint! {export { MainWindow, GeneralPart, ChronoElement, VisData } from "src/main.slint";}

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
    
    let shit: Vec<SharedString> = get_profile_names();
    let profiles: Vec<&str> = shit.iter().map(|x: &SharedString| x.as_str()).collect();
    

    doc["exercises"][exercise]["type"] = value(exercise_type);
    let current_profile: String = doc["current_profile"].as_str().unwrap().to_string();
    for profile in profiles {
        doc["exercises"][exercise]["profiles"][profile] = value(false);
    }
    
    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn add_reps(exercise: &str, reps: i32) {
    let toml_str: String = fs::read_to_string(PATH_TO_DATA).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    if let Some(exercise_value) = doc.get(exercise) {
        let reps: i64 = i64::from(reps);
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

            let mut chrono: File = OpenOptions::new().append(true).create(true).open(PATH_TO_CHRONO).unwrap();
            writeln!(chrono, "[{}]:{}:{}", Local::now().format("%Y-%m-%d"),exercise, reps);
        }
    }
}

fn change_interval(floor: &SharedString, ceil: &SharedString) {
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
    let current_profile: &str = doc["current_profile"].as_str().unwrap();
    
    
    if let Some(exercises_table) = doc.get("exercises").and_then(|t| t.as_table()) {
        for (exercise_name, exercise_data) in exercises_table {
            if let Some(status) = exercise_data.get("profiles").unwrap().get(current_profile) {
                if status.as_bool().unwrap() {
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
        for (exercise_name, exercise_data) in exercises_table {
            if let Some(status) = exercise_data.get("profiles").unwrap().get(current_profile).unwrap().as_bool() {
                exercises.push(String::from(exercise_name));
                activations.push(status);
            }
        }
    }

    let items: Vec<Exercise> = exercises.into_iter().zip(activations).map(|(exercise, activation)| Exercise {
        name: exercise.to_shared_string(),
        activation_status: activation,}).collect();

    items
}

fn set_exercise_activation(exercise_name: &SharedString) {
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
            let settings_table = settings.as_inline_table_mut().unwrap();
            if let Some(profiles_table) = settings_table.get_mut("profiles").and_then(|p| p.as_inline_table_mut()) {
                profiles_table.entry(profile).or_insert(false.into());
            }
        }
    }

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn remove_profile(profile: &str) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let current_profile: &str = doc["current_profile"].as_str().unwrap();
    if current_profile == profile {
        return;
    }
    
    if let Some(exercises) = doc.get_mut("exercises").and_then(|e| e.as_table_mut()) {
        for (_name, settings) in exercises.iter_mut() {
            let settings_table = settings.as_inline_table_mut().unwrap();
            if let Some(profiles_table) = settings_table.get_mut("profiles").and_then(|p| p.as_inline_table_mut()) {
                match profiles_table.entry(profile) {
                    InlineEntry::Occupied(key) => {key.remove();}
                    InlineEntry::Vacant(key) => {}
                }
            }
        }
    }

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn get_profile_names() -> Vec<SharedString> {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).unwrap();
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().unwrap();

    let exercises: &Table = doc.get("exercises").unwrap().as_table().unwrap();
    if let Some(profile_inline) = exercises.get_values().first() {
        let profile_names1 = profile_inline.1.as_inline_table().unwrap().get("profiles").unwrap().as_inline_table().unwrap();
        let profile_names2: Vec<SharedString> = profile_names1.iter().map(|x| x.0.into()).collect::<Vec<SharedString>>();
        profile_names2
    } else {
        Vec::new()
    }
}

fn get_current_profile() -> String {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).unwrap();
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().unwrap();

    return doc["current_profile"].as_str().unwrap().to_string();
}

fn change_volume(volume: i32) {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    doc["volume"] = value(volume as f64);

    fs::write(PATH_TO_CONFIG, doc.to_string()).expect("Fehler beim Schreiben!");
}

fn get_volume() -> i32 {
    let toml_str: String = fs::read_to_string(PATH_TO_CONFIG).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    return doc["volume"].as_float().unwrap() as i32;
}

fn create_visualisation_data() {
    let toml_str: String = fs::read_to_string(PATH_TO_DATA).expect("Fehler beim lesen!");
    let mut doc: DocumentMut = toml_str.parse::<DocumentMut>().expect("Fehler beim parsen!");

    let mut general_map = HashMap::new();
    for (exercise_name, exercise_data) in doc.as_table() {
        if let Some(exercise_data_table) = exercise_data.as_table() {
            let tuple = (
                exercise_data_table.get("reps").or_else(|| {exercise_data_table.get("time")}).unwrap().as_integer().unwrap() as i32,
                exercise_data_table["amount"].as_integer().unwrap() as i32, 
                exercise_data_table["max"].as_integer().unwrap() as i32);
            general_map.insert(exercise_name.to_string(), tuple);
        }
    }

    let mut chrono_map: HashMap<String, Vec<(NaiveDate, i32)>> = HashMap::new();
    let chrono: BufReader<File> = BufReader::new(File::open(PATH_TO_CHRONO).unwrap());
    for line in chrono.lines() {
        match line {
            Ok(raw_line) => {
                let exercise_name = raw_line.split(':').collect::<Vec<&str>>()[1].to_string();
                let exercise_data = (NaiveDate::parse_from_str(raw_line.chars().skip(1).take(10).collect::<String>().as_str(), "%Y-%m-%d").unwrap(), raw_line.split(":").collect::<Vec<&str>>()[2].parse::<i32>().unwrap());
                chrono_map.entry(exercise_name).or_default().push(exercise_data);
            },
            Err(e) => {}
        }
    }

    {
        let mut vis = VISUALISATION_DATA.lock().unwrap();
        vis.0 = general_map;
        vis.1 = chrono_map;
    }
}

fn get_visualisable_exercises() -> Vec<SharedString> {
    let vis_data = VISUALISATION_DATA.lock().unwrap();
    let general = vis_data.0.clone();
    let chrono = vis_data.1.clone();

    let mut visualisable_exercise_names = vec![];
    let all_exercises = get_exercise_settings().iter().map(|item| item.name.clone().to_string()).collect::<Vec<String>>();
    for exercise in all_exercises {
        if chrono.contains_key(&exercise) {
            visualisable_exercise_names.push(exercise.clone());
        }
    }
    visualisable_exercise_names.iter().map(|x| x.into()).collect()
}

fn visualisation_helper(exercise: &str, interval: &str) -> VisData {
    let vis_data = VISUALISATION_DATA.lock().unwrap();
    let general = vis_data.0.clone();
    let chrono = vis_data.1.clone();

    let general_extracted = general[exercise];

    let mut chrono_extracted = chrono[exercise].clone();
    let first_entry = chrono_extracted[0].0;
    let mut interval_start: NaiveDate = match interval {
        "year" => {
            first_entry.with_month(1).unwrap().with_day(1).unwrap()
        },
        "month" => {
            first_entry.with_day(1).unwrap()
        },
        "week" => {
            first_entry - Duration::days(i64::from(first_entry.weekday().num_days_from_monday()))
        },
        _ => {
            first_entry
        }
    };
    let now = chrono::Local::now().date_naive();


    let mut chrono_grouped: Vec<(NaiveDate, i32)> = vec![];
    while interval_start < now && !chrono_extracted.is_empty() {
        let interval_stepped = match interval {
            "year" => {
                interval_start.with_year(interval_start.year() + 1).unwrap()
            },
            "month" => {
                let mut month = interval_start.month() + 1;
                let mut year = interval_start.year();
                if month > 12 {
                    month = 1;
                    year += 1;
                }
                interval_start.with_year(year).and_then(|d| d.with_month(month)).unwrap()
            },
            "week" => {
                interval_start + Duration::days(7)
            },
            _ => {
                interval_start + Duration::days(1)
            }
        };

        let mut interval_sum: i32 = chrono_extracted.iter().filter(|(date, _)| *date>=interval_start && *date<interval_stepped).map(|(_, reps)| *reps).sum();
        chrono_grouped.push((interval_start, interval_sum));
        interval_start = interval_stepped;
    }

    let general: GeneralPart = GeneralPart {
        reps: general_extracted.0,
        amount: general_extracted.1,
        max: general_extracted.2,
    };
    let mut chrono_set = ModelRc::new(VecModel::from(chrono_grouped.iter().map(|(date,value)| ChronoElement {date: (date.to_string().into()), value: (*value)}).collect::<Vec<ChronoElement>>()));
    let mut vis_data: VisData = VisData { chrono: (chrono_set), general: (general), highest_value: (chrono_grouped.iter().map(|x| x.1).max().unwrap()) };
    return vis_data;
}

fn find_mp3_files(dir_path: &str) -> Vec<String> {    
    let entries = fs::read_dir(dir_path)
        .expect(&format!("Konnte den Ordner nicht lesen: {}", dir_path));
    
    let mut mp3_files = Vec::new();

    for entry_result in entries {
        let entry = entry_result.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(path_str) = path.to_str() {
                if path_str.to_lowercase().ends_with(".mp3") {
                    mp3_files.push(path_str.to_string());
                }
            }
        }
    }

    mp3_files
}

fn get_random_sound() -> String {
    let all_paths = find_mp3_files(PATH_TO_SOUNDS);

    if all_paths.is_empty() {
        panic!("Keine MP3-Dateien im Ordner gefunden: {}", PATH_TO_SOUNDS);
    }
    
    //let mut rng = rand::rng();
    let index = rand::random_range(0..all_paths.len());
    
    println!("{:?}", all_paths[index]);
    all_paths[index].clone()
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
        handle.set_visualisable_exercise_names(ModelRc::new(VecModel::from(get_visualisable_exercises())));
        handle.set_profile_names(ModelRc::new(VecModel::from(get_profile_names())));
        handle.set_current_profile(get_current_profile().into());
        handle.set_volume(get_volume());
    }

    let sink_for_start = sink.clone();
    let sink_for_reps = sink.clone();

    // true => time-loop inaktiv
    // false => time-loop aktiv

    let is_running: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));
    let is_running_clone: Rc<RefCell<bool>> = is_running.clone();
    let start_button_status: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));
    let start_button_status_clone: Rc<RefCell<bool>> = start_button_status.clone();

    let ui_handle_save_reps: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_add_exercise: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_remove_exercise: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_add_profile: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_remove_profile: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_change_profile: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_vis: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_changed_vis_exercise: Weak<MainWindow> = ui_handle.clone();
    let ui_handle_changed_vis_interval: Weak<MainWindow> = ui_handle.clone();

    let chosen_exercise:Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let chosen_exercise_rep_clone: Rc<RefCell<String>> = chosen_exercise.clone();

    ui.on_start_pressed(move |floor: SharedString, ceil: SharedString| {

        if !get_exercises().is_empty() {
            if *start_button_status_clone.borrow() {

                change_interval(&floor, &ceil);

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

                    let sink_for_timer: Rc<RefCell<Sink>> = sink_for_start.clone();

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
                            let mut sink = sink_for_timer.borrow_mut();
                            sink.clear();

                            let file_path = get_random_sound();
                            println!(">>> SPIELE NEUEN SOUND: {} <<<", file_path);
                            let file = File::open(file_path).expect("Konnte Sounddatei nicht öffnen");
                            let source = Decoder::new(BufReader::new(file)).expect("Konnte Sound nicht dekodieren");
                            
                            let current_volume = unsafe { VOLUME };
                            let boing = source.amplify(current_volume / 100.0);

                            sink.append(boing);
                            sink.play();
                        
                        } else if !*is_running_deep.borrow() {
                            timer_clone.stop();
                            *start_button_status_deep.borrow_mut() = true;
                            if let Some(handle) = ui_handle_deep.upgrade() {
                                handle.set_start_button_status(true);
                            }

                        } else if let Some(handle) = ui_handle_deep.upgrade() {
                            handle.set_passed_time(duration as i32);
                        }
                    });
                }
            } else if !(*start_button_status_clone.borrow()) {
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
        sink_for_reps.borrow_mut().clear();
    });

    ui.on_changed_general_settings(move |setting_doubles: bool| {
        set_general_settings(setting_doubles);
    });

    ui.on_changed_activation_settings(move |name: SharedString| {
        set_exercise_activation(&name);
    });

    ui.on_add_exercise(move |name:SharedString, exercise_type:SharedString| {
        add_exercise(name.as_str(), exercise_type.as_str());
        
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

    ui.on_add_profile(move |name: SharedString| {
        add_profile(name.as_str());

        if let Some(handle) = ui_handle_add_profile.upgrade() {
            handle.set_profile_names(ModelRc::new(VecModel::from(get_profile_names())));
        }
    });

    ui.on_remove_profile(move |name: SharedString| {
        remove_profile(name.as_str());

        if let Some(handle) = ui_handle_remove_profile.upgrade() {
            handle.set_profile_names(ModelRc::new(VecModel::from(get_profile_names())));
        }
    });

    ui.on_change_profile(move |name: SharedString| {
        change_profile(name.as_str());

        if let Some(handle) = ui_handle_change_profile.upgrade() {
            let exercise_structs: Vec<Exercise> = get_exercise_settings();
            let exercise_structs_clone = exercise_structs.clone();
            handle.set_exercises(ModelRc::new(VecModel::from(exercise_structs)));
            handle.set_current_profile(get_current_profile().into());
        }
    });

    ui.on_changed_volume(move |volume: i32| {
        unsafe {VOLUME = volume as f32;}
        change_volume(volume);
    });

    ui.on_load_visualisation_data(move || {
        create_visualisation_data();
        
        if let Some(handle) = ui_handle_vis.upgrade() {
            let data = visualisation_helper(get_visualisable_exercises().first().unwrap().as_str(), "day");
            handle.set_visualisable_exercise_names(ModelRc::new(VecModel::from(get_visualisable_exercises())));
            handle.set_visualisation_data(data);
        }
    });

    ui.on_changed_visualisation_exercise(move |exercise: SharedString| {
        if let Some(handle) = ui_handle_changed_vis_exercise.upgrade() {
            let data = visualisation_helper(exercise.as_str(), "day");
            handle.set_visualisation_data(data);
        }
    });

    ui.on_changed_visualisation_interval(move |exercise: SharedString, interval: SharedString| {
        if let Some(handle) = ui_handle_changed_vis_interval.upgrade() {
            let data = visualisation_helper(exercise.as_str(), interval.as_str());
            handle.set_visualisation_data(data);
        }
    });

    ui.run().unwrap();
}

//REMINDER vllt einstellen wieviele daten angezeigt werden 
//TODO daten in diagrammen sehen können
//TODO mehrere lieder
//TODO prioritize
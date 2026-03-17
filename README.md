# Sport - Interval Exercise Tracker

A desktop interval timer and tracker for sports activities written in Rust. The graphical user interface is based on the [Slint](https://slint.dev/) framework. The app reminds the user to perform exercises at random intervals, alerts them with audio notifications, and cleanly logs progress across different profiles. This method of training in frequent, random bursts rather than long workouts focuses more on ZNS connectivity than muscle hypertrophy and is called Grease-the-Groove.

## Features

- **Interval Timer:** Set a minimum and maximum time interval. The app selects a random time within this interval and notifies you as soon as it's time for an exercise.
- **Audio Notifications:** Plays randomly selected MP3 sounds from a local directory when the timer expires.
- **Workout Tracking & Statistics:** Records completed repetitions, execution frequency (number of sets), and personal records (Max) for each exercise.
- **Data Visualization:** Evaluate your training data chronologically. View histories and progress flexibly filtered by year, month, week, or day.
- **User Profiles:** Supports multiple profiles. Each profile can have different active exercises.
- **Local Data Storage:** All logs, configurations, and statistics are stored locally and offline in customizable TOML and text files within the `resources/` folder.

## Screenshots
<img width="1202" height="832" alt="Image" src="https://github.com/user-attachments/assets/8564063c-c5ea-4e21-a161-ed659708b942" />

<img width="1202" height="832" alt="Image" src="https://github.com/user-attachments/assets/50bc32dd-fed4-4d3c-973b-9c81151ed977" />

## Technologies

- **Programming Language:** Rust
- **GUI Framework:** [Slint](https://slint.dev/) (Uses `.slint` files for flexible UI design)
- **Audio:** `rodio` (For playing MP3 files)
- **Data Management:** `toml`, `toml_edit`, `chrono`
- **Randomness:** `rand`

## Installation & Prerequisites

### Prerequisites

Since the project is developed in Rust, you need **Rust & Cargo** (see [rustup.rs](https://rustup.rs/)).

### Directory Structure (`resources`)

For the application to start correctly, a folder named `resources` must be located next to the executable (or in the project root if executed via `cargo run`). It should have the following structure:

```text
sport/
├── src/
│   └── ...                 # Rust and Slint source code
├── resources/
│   ├── config.toml         # Basic settings, volume, profiles, and active exercises
│   ├── exercise_data.toml  # Cumulative statistics (Reps, Sets, Max)
│   ├── chronological_data.txt # Chronological log of each performed exercise
│   └── sounds/             # Folder for MP3 notification sounds (e.g., a gong)
├── Cargo.toml
└── README.md
```

*Note: Before starting the app for the first time, make sure there is at least one `.mp3` file in the `resources/sounds/` folder, otherwise the program might crash during playback.*

### Compile and Run

To start the project in development mode:

```bash
cargo run
```

For maximum performance and subsystem integration (`#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]` hides the console on Windows):

```bash
cargo run --release
```

## Usage

1. **Customize Exercises & Profiles:** Go to the settings tab. There you can create/switch user profiles and add new exercises or toggle them for the current profile.
2. **Start Timer:** Define your interval on the main view. Click on Start.
3. **Complete Exercise:** After the timer expires, you will hear a sound and an exercise will be assigned to you. Enter the number of completed repetitions and confirm.
4. **Visualization (Statistics):** Use the visualization section to view your progress in various exercises over defined periods (like the last 30 days) using dropdowns.

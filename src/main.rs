fn main() {
    //UI
    //Toml lesen und schreiben können
    //Datenstruktur mit Übungen die dann zufällig gewählt werden


    let mut exercises: Vec<String> = Vec::new();
    exercises.push(String::from("Pullup"));
    exercises.push(String::from("Pushup"));
    exercises.push(String::from("Situp"));
    println!("{}", pick_random_exercise(exercises))
}

fn pick_random_exercise(exercises: Vec<String>) -> String {
    let index: usize = rand::random_range(0..exercises.len());
    exercises[index].clone()

}
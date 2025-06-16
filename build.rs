extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // Korrekter Pfad zur .rc Datei (relativ zum Projekt-Root, wo build.rs ausgeführt wird)
        res.set_resource_file("app_icon_path.rc");
        // WICHTIG: Ressourcen kompilieren und linken!
        match res.compile() {
            Ok(()) => {}
            Err(e) => {
                // Gib eine Fehlermeldung aus, damit du siehst, wenn etwas schiefgeht
                eprintln!("Fehler beim Kompilieren der Windows-Ressourcen: {e}");
                std::process::exit(1);
            }
        }
    }
}
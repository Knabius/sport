extern crate winres;

fn main() {
    //Icon
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_resource_file("../app_icon.rc");
    }
}
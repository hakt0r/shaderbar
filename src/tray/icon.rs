use crate::utils::global_init;
use glob::glob;
use std::collections::HashMap;

/*
 ██████╗ ███████╗███████╗ ██████╗ ██╗    ██╗   ██╗███████╗    ██╗ ██████╗ ██████╗ ███╗   ██╗
 ██╔══██╗██╔════╝██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝    ██║██╔════╝██╔═══██╗████╗  ██║
 ██████╔╝█████╗  ███████╗██║   ██║██║    ██║   ██║█████╗      ██║██║     ██║   ██║██╔██╗ ██║
 ██╔══██╗██╔══╝  ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝      ██║██║     ██║   ██║██║╚██╗██║
 ██║  ██║███████╗███████║╚██████╔╝███████╗╚████╔╝ ███████╗    ██║╚██████╗╚██████╔╝██║ ╚████║
 ╚═╝  ╚═╝╚══════╝╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝    ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝
*/

global_init!( icon_overrides, HashMap<String, String>, init_icon_overrides );

fn init_icon_overrides() -> HashMap<String, String> {
    let mut overrides = HashMap::new();
    overrides.insert(
        "audio-volume-muted".to_string(),
        "/usr/share/icons/gnome/16x16/status/audio-volume-muted.png".to_string(),
    );
    overrides.insert(
        "audio-volume-low".to_string(),
        "/usr/share/icons/gnome/16x16/status/audio-volume-low.png".to_string(),
    );
    overrides.insert(
        "audio-volume-medium".to_string(),
        "/usr/share/icons/gnome/16x16/status/audio-volume-medium.png".to_string(),
    );
    overrides.insert(
        "audio-volume-high".to_string(),
        "/usr/share/icons/gnome/16x16/status/audio-volume-high.png".to_string(),
    );
    return overrides;
}

pub fn resolve(name: String) -> String {
    eprintln!("Resolving icon: {}", name);
    let overrides = icon_overrides();
    let theme = "*";
    if overrides.contains_key(&name) {
        return overrides.get(&name).unwrap().clone();
    }
    let mut path = format!("/usr/share/icons/{}/16x16/**/", theme);
    path.push_str(&name);
    path.push_str(".png");
    let mut resolved = String::from("");
    for entry in glob(&path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                resolved = path.display().to_string();
                break;
            }
            Err(e) => println!("{:?}", e),
        }
    }
    return resolved;
}

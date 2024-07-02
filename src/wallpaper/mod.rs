use crate::config::Config;
use colored::Colorize;
use gtk4::{gdk::Monitor, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::collections::HashMap;

/*
 ██╗    ██╗ █████╗ ██╗     ██╗     ██████╗  █████╗ ██████╗ ███████╗██████╗
 ██║    ██║██╔══██╗██║     ██║     ██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗
 ██║ █╗ ██║███████║██║     ██║     ██████╔╝███████║██████╔╝█████╗  ██████╔╝
 ██║███╗██║██╔══██║██║     ██║     ██╔═══╝ ██╔══██║██╔═══╝ ██╔══╝  ██╔══██╗
 ╚███╔███╔╝██║  ██║███████╗███████╗██║     ██║  ██║██║     ███████╗██║  ██║
  ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝
*/

crate::utils::global!(wallpaper_windows, HashMap<gtk4::gdk::Monitor, gtk4::Window>, HashMap::new());

pub fn wallpaper_enabled(config: &Config) -> bool {
    let wallpaper = config.config["wallpaper"]
        .as_object()
        .expect("$config.wallpaper is not an object")
        .get("enabled")
        .expect("$config.wallpaper.enabled is not set")
        .as_bool()
        .expect("$config.wallpaper.enabled is not a boolean");
    println!(
        "[{}]: {}",
        "wallpaper".green(),
        match wallpaper {
            true => "enabled".yellow(),
            false => "disabled".yellow(),
        }
    );
    return wallpaper;
}

pub fn init_wallpaper(config: &Config) {
    if !wallpaper_enabled(config) {
        return;
    }
    let display_manager = gtk4::gdk::DisplayManager::get();
    display_manager.connect_display_opened(|_, _| diff_wallpaper_windows());
    let screen = gtk4::gdk::Display::default().unwrap();
    screen.connect_seat_added(|_, _| diff_wallpaper_windows());
    screen.connect_seat_removed(|_, _| diff_wallpaper_windows());
    screen.connect_setting_changed(|_, _| diff_wallpaper_windows());
    let monitors = screen.monitors();
    monitors.connect("items-changed", true, |_| {
        diff_wallpaper_windows();
        None
    });
    diff_wallpaper_windows();
}

fn diff_wallpaper_windows() {
    eprintln!(
        "[{}]: {}",
        "wallpaper".green(),
        "diff_wallpaper_windows".yellow()
    );

    let screen = gtk4::gdk::Display::default().unwrap();
    let monitor = screen.monitors();
    let windows = wallpaper_windows();

    for m in &monitor.clone() {
        let monitor = m.unwrap().downcast::<Monitor>().unwrap();
        let window = windows.get(&monitor);
        let found = window.is_some();
        if !found {
            create_wallpaper_for_monitor(&monitor);
        }
    }

    let windows = wallpaper_windows();

    for (m, w) in &windows.clone() {
        let found = monitor
            .clone()
            .iter::<Monitor>()
            .any(|existing_monitor| existing_monitor.unwrap().downcast::<Monitor>().unwrap() == *m);
        if !found {
            eprintln!(
                "[{}]: {}: {}({})",
                "wallpaper".green(),
                "diff_wallpaper_windows".yellow(),
                "destroy".red(),
                m.model().unwrap().magenta()
            );
            w.destroy();
            windows.remove(m);
        }
    }
}

fn create_wallpaper_for_monitor(monitor: &Monitor) {
    let workarea = monitor.geometry();
    let width = workarea.width();
    let height = workarea.height();

    eprintln!(
        "[{}]: {}: {} [{},{}]",
        "wallpaper".green(),
        "create_wallpaper_for_monitor".green(),
        monitor.model().unwrap(),
        width,
        height
    );

    let window = gtk4::Window::new();

    window.init_layer_shell();
    window.set_title(Some(
        format!("{} - wallpaper", env!("CARGO_PKG_NAME")).as_str(),
    ));

    window.set_monitor(&monitor);
    window.set_decorated(false);

    window.set_layer(gtk4_layer_shell::Layer::Background);
    window.set_namespace(env!("CARGO_PKG_NAME"));

    window.set_width_request(width);
    window.set_height_request(height);

    window.set_margin(gtk4_layer_shell::Edge::Top, 0);
    window.set_margin(gtk4_layer_shell::Edge::Right, 0);
    window.set_margin(gtk4_layer_shell::Edge::Bottom, 0);
    window.set_margin(gtk4_layer_shell::Edge::Left, 0);

    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);

    window.add_css_class("wallpaper");

    window.show();
    window.present();

    wallpaper_windows().insert(monitor.clone(), window);
}

use crate::sensors::Sensors;
use crate::state::state;
use crate::tray::init_tray_icons;
use crate::wallpaper::init_wallpaper;
use config::config;
use gl::*;
use glib::spawn_future_local;
use gtk4::{glib, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{borrow::BorrowMut, ptr, time::Duration};
use utils::global;

mod config;
mod gl;
mod sensors;
mod state;
mod tray;
mod utils;
mod wallpaper;

global!(
    application,
    gtk4::Application,
    gtk4::Application::builder()
        .application_id("de.hakt0r.shaderbar")
        .build()
);
global!(
    window,
    gtk4::ApplicationWindow,
    gtk4::ApplicationWindow::new(application())
);
global!(widget, GliumGLArea, GliumGLArea::default());
global!(is_ready, Option<bool>, Some(false));
global!(sensors, Sensors, Sensors::new());

#[tokio::main]
async fn main() -> glib::ExitCode {
    eprintln!("Starting shaderbar");

    let config = config().await;

    pre_init().await;

    let application: &mut gtk4::Application = application();

    application.connect_activate(move |app| {
        init_ui(app);
        post_init(&config);
    });

    return application.run();
}

async fn pre_init() {
    load_epoxy();
    state();
    sensors().read();
    read_sensors();
    read_sensors_lowfreq();
    render_timer();
}

fn init_ui(_: &gtk4::Application) {
    let window = window();

    window.init_layer_shell();
    window.set_title(Some(env!("CARGO_PKG_NAME")));
    window.set_layer(gtk4_layer_shell::Layer::Top);
    window.set_namespace(env!("CARGO_PKG_NAME"));

    window.auto_exclusive_zone_enable();
    window.set_width_request(1920);
    window.set_height_request(24);

    window.set_margin(gtk4_layer_shell::Edge::Top, 0);
    window.set_margin(gtk4_layer_shell::Edge::Right, 0);
    window.set_margin(gtk4_layer_shell::Edge::Bottom, 0);
    window.set_margin(gtk4_layer_shell::Edge::Left, 0);

    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);

    let container = gtk4::Fixed::new();

    window.set_child(Some(&container));

    let widget = widget();
    widget.set_width_request(1920);
    widget.set_height_request(24);
    container.put(widget, 0f64, 0f64);

    container.put(&tray::tray().widget, 16f64, 0f64);

    window.present();

    (*is_ready()).replace(true);
}

fn post_init(config: &config::Config) {
    init_stylesheet();
    init_wallpaper(config);
    init_tray_icons();
}

fn load_epoxy() {
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });
}

fn init_stylesheet() {
    let provider = gtk4::CssProvider::new();
    #[cfg(not(debug_assertions))]
    {
        let stylesheet_file = config().stylesheet_file.clone();
        provider.load_from_path(&stylesheet_file);
    }
    #[cfg(debug_assertions)]
    provider.load_from_path(std::path::Path::new("src/config/defaults.css"));
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

/*
 ████████╗██╗███╗   ███╗███████╗██████╗ ███████╗
 ╚══██╔══╝██║████╗ ████║██╔════╝██╔══██╗██╔════╝
    ██║   ██║██╔████╔██║█████╗  ██████╔╝███████╗
    ██║   ██║██║╚██╔╝██║██╔══╝  ██╔══██╗╚════██║
    ██║   ██║██║ ╚═╝ ██║███████╗██║  ██║███████║
    ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝
*/

fn render_timer() {
    spawn_future_local(async move {
        readyness().await;
        loop {
            glib::timeout_future(Duration::from_millis(1000 / 30)).await;
            widget().queue_render();
        }
    });
}

fn read_sensors() {
    spawn_future_local(async move {
        loop {
            sensors().read();
            glib::timeout_future(Duration::from_millis(1000 / 30)).await;
        }
    });
}

fn read_sensors_lowfreq() {
    spawn_future_local(async move {
        loop {
            glib::timeout_future(Duration::from_secs(1)).await;
            sensors().read_lowfreq();
        }
    });
}

async fn readyness() {
    while !is_ready().unwrap() {
        glib::timeout_future(Duration::from_millis(100)).await;
    }
}

pub const ERR_CHANNEL_SEND: &str = "Failed to send message to channel";

/*
      ██╗██╗   ██╗███╗   ██╗██╗  ██╗
      ██║██║   ██║████╗  ██║██║ ██╔╝
      ██║██║   ██║██╔██╗ ██║█████╔╝
 ██   ██║██║   ██║██║╚██╗██║██╔═██╗
 ╚█████╔╝╚██████╔╝██║ ╚████║██║  ██╗
  ╚════╝  ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝
*/

/* let screen = gdk::Screen::default().unwrap();
let monitor = screen.primary_monitor().unwrap();
let workarea = monitor.workarea();
let (x, y) = workarea.origin();
let (width, height) = workarea.size();

window.move_(x, y);
window.resize(width, height);

window.connect_destroy(|window| {
    let mut windows = wallpaper_windows();
    windows.retain(|w| w != window);
});

window.connect_screen_changed(|window, _| {
    let screen = gdk::Screen::default().unwrap();
    let monitor = screen.primary_monitor().unwrap();
    let workarea = monitor.workarea();
    let (x, y) = workarea.origin();
    let (width, height) = workarea.size();

    window.move_(x, y);
    window.resize(width, height);
});

window.connect_configure_event(|window, _| {
    let screen = gdk::Screen::default().unwrap();
    let monitor = screen.primary_monitor().unwrap();
    let workarea = monitor.workarea();
    let (x, y) = workarea.origin();
    let (width, height) = workarea.size();

    window.move_(x, y);
    window.resize(width, height);
    gtk4::Inhibit(false)
});

window.connect_draw(|window, context| {
    let screen = gdk::Screen::default().unwrap();
    let monitor = screen.primary_monitor().unwrap();
    let workarea = monitor.workarea();
    let (x, y) = workarea.origin();
    let (width, height) = workarea.size();

    context.set_source_rgb(0.0, 0.0, 0.0);
    context.rectangle(x as f64, y as f64, width as f64, height as f64);
    context.fill();

    gtk4::Inhibit(false)
});

window.connect_draw(|window, context| {
    let screen = gdk::Screen::default().unwrap();
    let monitor = screen.primary_monitor().unwrap();
    let workarea = monitor.workarea();
    let (x, y) = workarea.origin();
    let (width, height) = workarea.size();

    context.set_source_rgb(0.0, 0.0, 0.0);
    context.rectangle(x as f64, y as f64, width as f64, height as f64);
    context.fill();

    gtk4::Inhibit(false)
}); */

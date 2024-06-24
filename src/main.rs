use crate::sensors::Sensors;
use crate::state::state;
use crate::tray::create_tray;
use gl::*;
use gtk4::{glib, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{borrow::BorrowMut, ptr, time::Duration};
use utils::global;
mod gl;
mod sensors;
mod state;
mod tray;

mod utils {
    macro_rules! global {
        ($name:ident, $type:ty, $default:expr) => {
            pub fn $name() -> &'static mut $type {
                static mut VALUE: Option<$type> = None;
                unsafe { VALUE.get_or_insert_with(|| $default).borrow_mut() }
            }
        };
    }
    macro_rules! global_init {
        ($name:ident, $type:ty, $initializer:expr) => {
            pub fn $name() -> &'static mut $type {
                static mut VALUE: Option<$type> = None;
                unsafe { VALUE.get_or_insert_with($initializer) }
            }
        };
    }
    pub(crate) use global;
    pub(crate) use global_init;
}

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
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });

    state();
    sensors().read();

    glib::spawn_future_local(async move {
        read_sensors().await;
    });

    glib::spawn_future_local(async move {
        read_sensors_lowfreq().await;
    });

    glib::spawn_future_local(async move {
        render_timer().await;
    });

    glib::spawn_future_local(async move {
        create_tray().await;
    });

    let application = application();

    application.connect_activate(move |app| {
        let provider = gtk4::CssProvider::new();
        provider.load_from_path(std::path::Path::new("src/style.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        build_ui(app);
    });

    return application.run();
}

fn build_ui(_: &gtk4::Application) {
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

    init_wallpaper();

    (*is_ready()).replace(true);
}

/*
 ████████╗██╗███╗   ███╗███████╗██████╗ ███████╗
 ╚══██╔══╝██║████╗ ████║██╔════╝██╔══██╗██╔════╝
    ██║   ██║██╔████╔██║█████╗  ██████╔╝███████╗
    ██║   ██║██║╚██╔╝██║██╔══╝  ██╔══██╗╚════██║
    ██║   ██║██║ ╚═╝ ██║███████╗██║  ██║███████║
    ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝╚══════╝
*/

async fn render_timer() {
    readyness().await;
    loop {
        glib::timeout_future(Duration::from_millis(1000 / 30)).await;
        widget().queue_render();
    }
}

async fn read_sensors() {
    loop {
        sensors().read();
        glib::timeout_future(Duration::from_millis(1000 / 30)).await;
    }
}

async fn read_sensors_lowfreq() {
    loop {
        glib::timeout_future(Duration::from_secs(1)).await;
        sensors().read_lowfreq();
    }
}

async fn readyness() {
    while !is_ready().unwrap() {
        glib::timeout_future(Duration::from_millis(100)).await;
    }
}

pub const ERR_CHANNEL_SEND: &str = "Failed to send message to channel";

/*
 ██╗    ██╗ █████╗ ██╗     ██╗     ██████╗  █████╗ ██████╗ ███████╗██████╗
 ██║    ██║██╔══██╗██║     ██║     ██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗
 ██║ █╗ ██║███████║██║     ██║     ██████╔╝███████║██████╔╝█████╗  ██████╔╝
 ██║███╗██║██╔══██║██║     ██║     ██╔═══╝ ██╔══██║██╔═══╝ ██╔══╝  ██╔══██╗
 ╚███╔███╔╝██║  ██║███████╗███████╗██║     ██║  ██║██║     ███████╗██║  ██║
  ╚══╝╚══╝ ╚═╝  ╚═╝╚══════╝╚══════╝╚═╝     ╚═╝  ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝
*/

use gtk4::gdk::Monitor;
use std::collections::HashMap;

global!(wallpaper_windows, HashMap<gtk4::gdk::Monitor, gtk4::Window>, HashMap::new());

fn init_wallpaper() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_path(std::path::Path::new("src/wallpaper.css"));
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let screen = gtk4::gdk::Display::default().unwrap();
    screen.connect_seat_added(|_, _| diff_wallpaper_windows());
    screen.connect_seat_removed(|_, _| diff_wallpaper_windows());
    diff_wallpaper_windows();
}

fn diff_wallpaper_windows() {
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

    let mut windows = wallpaper_windows();

    for (m, w) in &windows.clone() {
        let found = monitor
            .clone()
            .iter::<Monitor>()
            .any(|existing_monitor| existing_monitor.unwrap().downcast::<Monitor>().unwrap() == *m);
        if !found {
            w.destroy();
            windows.remove(m);
        }
    }
}

fn create_wallpaper_for_monitor(monitor: &Monitor) {
    let workarea = monitor.geometry();
    let width = workarea.width();
    let height = workarea.height();

    let window = gtk4::Window::new();

    window.init_layer_shell();
    window.set_title(Some(
        format!("{} - wallpaper", env!("CARGO_PKG_NAME")).as_str(),
    ));
    eprintln!("wallpaper: {}x{}", width, height);
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

mod config;
mod gl;
mod sensors;
mod tray;
mod utils;
mod wallpaper;

use crate::tray::tray;
use crate::wallpaper::init_wallpaper;
use config::config;
use gl::*;
use glib::spawn_future_local;
use gtk4::{glib, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{ptr, time::Duration};
use utils::global;

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
global!(
    widgets,
    gtk4::Grid,
    gtk4::Grid::builder()
        .row_spacing(0)
        .column_spacing(0)
        .build()
);
global!(is_ready, Option<bool>, Some(false));

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
    sensors::detect().await;
    sensors::spawn_read_sensors();
    sensors::spawn_read_sensors_lowfreq();
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

    let widgets = widgets();
    widgets.attach(&tray::tray().widget, 0, 0, 1, 1);
    container.put(widgets, 16f64, 0f64);
    base_widgets();
    window.present();

    (*is_ready()).replace(true);
}

fn base_widgets() {
    date_time_widget();
    user_host_widget();
    window_name_widget();
}

global!(
    user_host,
    (gtk4::Box, [gtk4::Label; 3]),
    (
        gtk4::Box::new(gtk4::Orientation::Horizontal, 0),
        [
            gtk4::Label::new(None),
            gtk4::Label::new(None),
            gtk4::Label::new(None)
        ]
    )
);

fn user_host_widget() {
    fn update_user_host() {
        let (_, [user, _, host]) = user_host();
        user.set_text(whoami::username().as_str());
        host.set_text(
            whoami::fallible::hostname()
                .unwrap_or("shaderbar".to_string())
                .as_str(),
        );
    }
    spawn_future_local(async move {
        let widgets = widgets();
        let (container, [user, at, host]) = user_host();
        container.add_css_class("user-host");
        user.add_css_class("user");
        at.add_css_class("at");
        host.add_css_class("hostname");
        at.set_text("@");
        update_user_host();
        widgets.attach(container, 1, 0, 1, 1);
        loop {
            update_user_host();
            glib::timeout_future(Duration::from_millis(1000)).await;
        }
    });
}

global!(window_name, gtk4::Label, gtk4::Label::new(None));
fn window_name_widget() {
    spawn_future_local(async move {
        let widgets = widgets();
        let window_name = window_name();
        window_name.set_text("swayfx");
        window_name.add_css_class("window-name");
        widgets.attach(window_name, 4, 0, 1, 1);
        loop {
            {
                let window_script = "swaymsg -t get_tree | jq -r '.. | select(.type?) | select(.focused==true) | .window_properties.title '";
                let window = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(window_script)
                    .output()
                    .unwrap();
                let utf8 = window.stdout;
                let name = String::from_utf8(utf8)
                    .to_owned()
                    .unwrap()
                    .trim()
                    .to_string();
                let name = match name.as_str() {
                    "null" => "",
                    "" => "",
                    _ => name.as_str(),
                };
                window_name.set_text(name);
            }
            glib::timeout_future(Duration::from_millis(1000 / 3)).await;
        }
    });
}

global!(date_time, gtk4::Label, gtk4::Label::new(None));
fn date_time_widget() {
    spawn_future_local(async move {
        let widgets = widgets();
        let date_time = date_time();
        date_time.set_text("00:00:00.00 1970/01/01");
        date_time.add_css_class("date-time");
        widgets.attach(date_time, 3, 0, 1, 1);
        loop {
            glib::timeout_future(Duration::from_millis(1000 / 30)).await;
            let now = chrono::Local::now();
            date_time.set_text(now.format("%T %Y/%m/%d").to_string().as_str());
        }
    });
}

fn post_init(config: &config::Config) {
    spawn_future_local(init_stylesheet());
    init_wallpaper(config);
    tray();
}

fn load_epoxy() {
    let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();

    epoxy::load_with(|name| {
        unsafe { library.get::<_>(name.as_bytes()) }
            .map(|symbol| *symbol)
            .unwrap_or(ptr::null())
    });
}

async fn init_stylesheet() {
    let provider = gtk4::CssProvider::new();
    #[cfg(not(debug_assertions))]
    {
        // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD
        let stylesheet_file = config().await.stylesheet_file.clone();
        provider.load_from_path(&stylesheet_file);
    } // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD // PROD
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

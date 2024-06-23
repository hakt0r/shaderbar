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

    (*is_ready()).replace(true);
}

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

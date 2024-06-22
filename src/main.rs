use crate::sensors::Sensors;
use crate::state::state;
use crate::tray::create_tray;
use gl::*;
use gtk4::{glib, prelude::*};
use gtk4_layer_shell::LayerShell;
use std::{borrow::BorrowMut, ptr, time::Duration};
mod desktop_file;
mod error;
mod gl;
mod sensors;
mod state;
mod tray;

static mut READY: Option<bool> = None;

pub fn ready() -> bool {
    unsafe {
        return READY.is_some();
    }
}

pub fn set_ready() {
    unsafe {
        READY = Some(true);
    }
}

pub fn sensors() -> &'static mut Sensors {
    static mut SENSORS: Option<Sensors> = None;
    unsafe { return SENSORS.get_or_insert_with(|| Sensors::new()).borrow_mut() }
}

pub fn widget() -> &'static GliumGLArea {
    static mut WIDGET: Option<GliumGLArea> = None;
    unsafe { return WIDGET.get_or_insert_with(|| GliumGLArea::default()) }
}

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

    let application = gtk4::Application::builder()
        .application_id("de.hakt0r.shaderbar")
        .build();

    application.connect_activate(move |application| {
        build_ui(application);
    });

    return application.run();
}

fn build_ui(application: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::new(application);

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

    let container = gtk4::Grid::builder()
        .row_spacing(0)
        .column_spacing(0)
        .build();

    window.set_child(Some(&container));

    let widget = widget();
    widget.set_width_request(1920);
    widget.set_height_request(24);
    container.attach(widget, 0, 0, 1, 1);

    tray::tray();

    window.present();

    set_ready();
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
    while !ready() {
        glib::timeout_future(Duration::from_millis(100)).await;
    }
}

pub const ERR_CHANNEL_SEND: &str = "Failed to send message to channel";

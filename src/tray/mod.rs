use glib::spawn_future_local;
use gtk4::{glib, prelude::*};
use std::{borrow::BorrowMut, collections::HashMap};
use system_tray::{client::Client, item::StatusNotifierItem};
pub mod diff;
pub mod icon;
// use crate::utils::*;
// use diff::get_diffs;

pub struct Tray {
    pub client: Option<Client>,
    pub items: HashMap<String, TrayItem>,
    pub widget: gtk4::Box,
}

fn init_tray() -> Tray {
    return Tray {
        client: None,
        items: HashMap::new(),
        widget: gtk4::Box::builder()
            .spacing(0)
            .orientation(gtk4::Orientation::Horizontal)
            .build(),
    };
}

pub fn tray() -> &'static mut Tray {
    static mut TRAY: Option<Tray> = None;
    unsafe {
        return TRAY.get_or_insert_with(init_tray);
    }
}

pub async fn create_tray() {
    spawn_future_local(async move {
        let pid: u32 = std::process::id();
        let process_name = "shaderbar";
        let process_id = format!("{}-{}", process_name, pid);
        eprintln!("Creating tray for process: {}", process_id);
        let client: Client = Client::new(&process_id).await.unwrap();
        let rx = &mut client.subscribe();
        let tray = tray();
        let widget = tray.widget.clone();
        tray.client = Some(client);

        while let Ok(ev) = rx.recv().await {
            let items = tray.items.borrow_mut();
            match ev {
                system_tray::client::Event::Add(_id, item) => {
                    let resolved = icon::resolve(item.icon_name.clone().unwrap());
                    let clickable = gtk4::Button::builder().css_classes(["tray"]).build();
                    let icon = gtk4::Image::from_file(resolved);
                    clickable.set_child(Some(&icon));
                    widget.append(&clickable);
                    // handle_click(id.clone(), &icon);
                }
                system_tray::client::Event::Update(id, item) => {
                    eprintln!("Updating tray item: {}\n{:?}", id, item);
                }
                system_tray::client::Event::Remove(id) => {
                    items.remove(&id);
                }
            }
        }
    });
}

pub fn handle_click(id: String, parent: &impl IsA<gtk4::Widget>) {
    let tray = tray();
    let item = tray.items.get(&id).unwrap();
    let menu = item.menu.clone();
    menu.set_parent(parent);
    menu.add_child(
        &gtk4::Label::builder().label("Hello, world!").build(),
        "Hello, world!",
    );
    menu.activate();
    menu.show();
}

pub struct TrayItem {
    pub item: Option<StatusNotifierItem>,
    pub icon: gtk4::Image,
    pub menu: gtk4::PopoverMenu,
}

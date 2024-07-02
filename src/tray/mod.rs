mod menu;
mod menu_item;
mod section;
mod submenu;
mod tray_item;

use menu::RootMenu;
use tray_item::TrayIcon;

use colored::Colorize;
use glib::spawn_future_local;
use gtk4::{glib, prelude::*};
use std::{borrow::BorrowMut, collections::HashMap};
use system_tray::client::{Client, Event};

pub struct Tray {
    pub client: Option<Client>,
    pub items: HashMap<String, TrayIcon>,
    pub widget: gtk4::Box,
}

crate::utils::global_init!(tray, Tray, init_tray);

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

/*
  ██████╗██╗     ██╗███████╗███╗   ██╗████████╗
 ██╔════╝██║     ██║██╔════╝████╗  ██║╚══██╔══╝
 ██║     ██║     ██║█████╗  ██╔██╗ ██║   ██║
 ██║     ██║     ██║██╔══╝  ██║╚██╗██║   ██║
 ╚██████╗███████╗██║███████╗██║ ╚████║   ██║
  ╚═════╝╚══════╝╚═╝╚══════╝╚═╝  ╚═══╝   ╚═╝
*/

pub fn init_tray_icons() {
    spawn_future_local(async move {
        let pid: u32 = std::process::id();
        let process_name = "shaderbar";
        let process_id = format!("{}-{}", process_name, pid);
        let client: Client = Client::new(&process_id).await.unwrap();
        let rx = &mut client.subscribe();
        let tray = tray();
        let widget = tray.widget.clone();
        tray.client = Some(client);

        while let Ok(ev) = rx.recv().await {
            let items = tray.items.borrow_mut();
            match ev {
                Event::Add(id, item) => {
                    let existing_item = items.get(&id);
                    if existing_item.is_some() {
                        eprintln!("[{}]: {}({})", "tray".green(), "item_exists".yellow(), id);
                        continue;
                    }
                    let item = TrayIcon::new(&id, item.as_ref());
                    widget.append(&item.button);
                    items.insert(id, item);
                }
                Event::Update(id, item) => match item {
                    system_tray::client::UpdateEvent::Menu(menu) => {
                        let tray_item = items.get(&id);
                        if tray_item.is_none() {
                            continue;
                        }
                        tray_item.unwrap().update_menu(&menu);
                    }
                    _ => {
                        eprintln!(
                            "[{}]: {}({})",
                            "tray".green(),
                            "unhandled_update_event".yellow(),
                            id
                        );
                    }
                },
                Event::Remove(id) => {
                    eprintln!("[{}]: {}({})", "tray".green(), "remove_item".yellow(), id);
                    let item = items.get(&id);
                    if item.is_none() {
                        eprintln!("Item not found: {}", id);
                        continue;
                    }
                    let item = item.unwrap();
                    widget.remove(&item.button);
                    items.remove(&id);
                }
            }
        }
    });
}

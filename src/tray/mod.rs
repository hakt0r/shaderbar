mod menu;
mod menu_item;
mod section;
mod submenu;
mod tray_icon;

use menu::RootMenu;
use tray_icon::TrayIcon;

use colored::Colorize;
use glib::spawn_future_local;
use gtk4::{glib, prelude::*, Box};
use std::{borrow::BorrowMut, collections::HashMap, sync::Arc};
use system_tray::client::{Client, Event};

use crate::utils::global;

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

/*
 ████████╗ ██████╗ ██╗   ██╗ ██████╗██╗  ██╗███████╗██████╗     ██╗  ██╗███████╗██╗   ██╗███████╗
 ╚══██╔══╝██╔═══██╗██║   ██║██╔════╝██║  ██║██╔════╝██╔══██╗    ██║ ██╔╝██╔════╝╚██╗ ██╔╝██╔════╝
    ██║   ██║   ██║██║   ██║██║     ███████║█████╗  ██║  ██║    █████╔╝ █████╗   ╚████╔╝ ███████╗
    ██║   ██║   ██║██║   ██║██║     ██╔══██║██╔══╝  ██║  ██║    ██╔═██╗ ██╔══╝    ╚██╔╝  ╚════██║
    ██║   ╚██████╔╝╚██████╔╝╚██████╗██║  ██║███████╗██████╔╝    ██║  ██╗███████╗   ██║   ███████║
    ╚═╝    ╚═════╝  ╚═════╝  ╚═════╝╚═╝  ╚═╝╚══════╝╚═════╝     ╚═╝  ╚═╝╚══════╝   ╚═╝   ╚══════╝
*/

global!(touched_keys, Vec<String>, Vec::new());

/*
██████╗  ██████╗ ██╗  ██╗███████╗███████╗
██╔══██╗██╔═══██╗╚██╗██╔╝██╔════╝██╔════╝
██████╔╝██║   ██║ ╚███╔╝ █████╗  ███████╗
██╔══██╗██║   ██║ ██╔██╗ ██╔══╝  ╚════██║
██████╔╝╚██████╔╝██╔╝ ██╗███████╗███████║
╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝
*/

global!(tray_menu_widget, HashMap<String, Arc<Box>>, HashMap::new());

pub fn cached_box(id: &String) -> (Arc<Box>, bool) {
    match tray_menu_widget().get(id) {
        Some(widget) => (Arc::clone(widget), true),
        None => {
            let rows = Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .build();
            tray_menu_widget().insert(id.clone(), Arc::new(rows));
            (Arc::clone(tray_menu_widget().get(id).unwrap()), false)
        }
    }
}

pub fn touch_cached_box(cache_key: &String, alias: &String) -> (Arc<Box>, bool) {
    eprintln!(
        "[{}]: {}({}) @{}",
        "tray".green(),
        "cached_box".yellow(),
        cache_key.blue(),
        alias.magenta()
    );
    touched_keys().push(cache_key.to_string());
    cached_box(&cache_key)
}

pub fn touch_or_init_cached_box(
    cache_key: &String,
    alias: &String,
    init: impl FnOnce(Arc<Box>, &String),
    touch: impl FnOnce(Arc<Box>, &String) -> Arc<Box>,
) {
    let (widget, was_cached) = touch_cached_box(cache_key, alias);
    touch(Arc::clone(&widget), &cache_key);
    if !was_cached {
        init(Arc::clone(&widget), &cache_key);
    }
}
